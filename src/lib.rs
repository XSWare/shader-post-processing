mod post_processing;
mod scene;
mod shader_globals;
mod texture;

use chrono::{DateTime, Utc};
use post_processing::PostProcessing;
use scene::Scene;
use shader_globals::Globals;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let size = winit::dpi::PhysicalSize { width: 800, height: 220 };

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Post processing example")
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        let _ = window.request_inner_size(size);

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| match event {
            Event::Resumed => {
                log::debug!("Resumed");
            }
            Event::WindowEvent { ref event, window_id } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::MouseInput {
                            state: button_state,
                            button,
                            ..
                        } => {
                            if *button == MouseButton::Left && button_state.is_pressed() {
                                change_post_processing_effect(&mut state);
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let size = state.window.inner_size();
                            state.globals.cursor_x = (position.x as f32 / size.width as f32) * 2. - 1.;
                            state.globals.cursor_y = (position.y as f32 / size.height as f32) * 2. - 1.;
                        }
                        WindowEvent::Touch(touch) => match touch.phase {
                            TouchPhase::Started => state.last_touch_start = chrono::Utc::now(),
                            TouchPhase::Ended => {
                                if (chrono::Utc::now() - state.last_touch_start).num_milliseconds() < 500 {
                                    change_post_processing_effect(&mut state)
                                }
                            }
                            TouchPhase::Moved => {
                                let position = touch.location;
                                let size = state.window.inner_size();
                                state.globals.cursor_x = (position.x as f32 / size.width as f32) * 2. - 1.;
                                state.globals.cursor_y = (position.y as f32 / size.height as f32) * 2. - 1.;
                            }
                            _ => {}
                        },
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                            surface_configured = true;
                        }
                        WindowEvent::RedrawRequested => {
                            // This tells winit that we want another frame after this one
                            state.window().request_redraw();

                            if !surface_configured {
                                return;
                            }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when the a frame takes too long to present
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .unwrap();
}

fn change_post_processing_effect(state: &mut State) {
    state.current_post_processing_index = (state.current_post_processing_index + 1) % state.post_processing_effects.len();
}

use winit::window::Window;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    start_time: DateTime<Utc>,
    globals: Globals,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    scene: Scene,
    post_processing_effects: Vec<PostProcessing>,
    current_post_processing_index: usize,
    last_touch_start: DateTime<Utc>,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let start_time = chrono::Utc::now();
        let globals = Globals::new();

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group_layout = device.create_bind_group_layout(shader_globals::BIND_GROUP_LAYOUT);

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals bind group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let scene = Scene::new(&device, &queue, config.format, &globals_bind_group_layout);

        let mut post_processing_effects = Vec::new();

        let invert_color_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/post_processing_invert_color.wgsl"));
        post_processing_effects.push(PostProcessing::new(
            &device,
            config.format,
            &globals_bind_group_layout,
            invert_color_shader,
        ));

        let wave_distortion_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/post_processing_wave_distortion.wgsl"));
        post_processing_effects.push(PostProcessing::new(
            &device,
            config.format,
            &globals_bind_group_layout,
            wave_distortion_shader,
        ));

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            start_time,
            globals,
            globals_buffer,
            globals_bind_group,
            scene,
            post_processing_effects,
            current_post_processing_index: 0,
            last_touch_start: start_time,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        self.globals.time = (chrono::Utc::now() - self.start_time).num_milliseconds() as f32 / 1000.;
        self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&self.globals));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let screen = self.surface.get_current_texture()?;
        let screen_texture = &screen.texture;
        let screen_view = screen_texture.create_view(&wgpu::TextureViewDescriptor { ..Default::default() });

        // create a view that only lives in memory and is not displayed on the screen
        let in_memory_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("initial render pass canvas"),
            dimension: screen_texture.dimension(),
            format: screen_texture.format(),
            mip_level_count: screen_texture.mip_level_count(),
            sample_count: screen_texture.sample_count(),
            size: screen_texture.size(),
            usage: screen_texture.usage() | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let in_memory_view = in_memory_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // first render pass - create the scene
        self.scene.render_pass(&mut encoder, &in_memory_view, &self.globals_bind_group)?;

        // second render pass - apply post processing effects to the scene
        self.post_processing_effects[self.current_post_processing_index].render_pass(
            &self.device,
            &mut encoder,
            &in_memory_view,
            &screen_view,
            &self.globals_bind_group,
        )?;

        self.queue.submit(std::iter::once(encoder.finish()));
        screen.present();

        Ok(())
    }
}
