# Post processing shader example

A small example to show how to do screen shaders / post processing with rust / wgpu.

Based on this [wgpu tutorial](https://sotrh.github.io/learn-wgpu).

## How it works
During the render step, two render passes are executed.
The first render pass works with an "in-memory" view that is not yet put on the screen.
The second render pass applies the post processing effect and outputs it to the screen view.
For details check the `render` function in [lib.rs](src/lib.rs#270) and the `render_pass` function in [post_processing.rs](src/post_processing.rs#101)

## Run the project

run natively: `cargo run`

## Host the project as a website

build javascript/wasm: `wasm-pack build --release --target web`  
host with http server: `python3 -m http.server`