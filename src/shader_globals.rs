#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub time: f32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    // WebGL needs 16 byte alignment
    padding: f32,
}

pub const BIND_GROUP_LAYOUT: &wgpu::BindGroupLayoutDescriptor = &wgpu::BindGroupLayoutDescriptor {
    label: Some("globals bind group layout"),
    entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        count: None,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
    }],
};

impl Globals {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            padding: 0.0,
        }
    }
}
