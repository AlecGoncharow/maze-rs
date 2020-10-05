use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(dead_code)]
mod grid;
mod renderer;
use renderer::{GraphicsContext, Vertex};

pub struct State {
    gfx_ctx: GraphicsContext,
}

fn make_grid(rows: usize, cols: usize) -> Vec<Vertex> {
    let mut grid = Vec::new();
    let sq_width = 2.0 / cols as f32;
    let sq_height = 2.0 / rows as f32;

    for row in 0..rows {
        for col in 0..cols {
            let low_x = ((col as f32 / cols as f32) * 2.0) - 1.0;
            let low_y = ((row as f32 / rows as f32) * 2.0) - 1.0;

            let verts: &[Vertex] = &[
                // lower left triangle
                Vertex {
                    position: [low_x, low_y, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [low_x + sq_width, low_y, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [low_x, low_y + sq_height, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
                // upper right triangle
                Vertex {
                    position: [low_x + sq_width, low_y + sq_height, 0.0],
                    color: [1.0, 1.0, 0.0],
                },
                Vertex {
                    position: [low_x + sq_width, low_y, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [low_x, low_y + sq_height, 0.0],
                    color: [0.0, 1.0, 1.0],
                },
            ];

            grid.append(&mut Vec::from(verts));
        }
    }

    grid
}

impl State {
    // returns false if event hasn't been fully processed
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.gfx_ctx.clear_color = wgpu::Color {
                    r: position.x as f64 / self.gfx_ctx.size.width as f64,
                    g: position.y as f64 / self.gfx_ctx.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        self.gfx_ctx.start();

        let verts = make_grid(5, 5);

        self.gfx_ctx.draw(&verts);

        self.gfx_ctx.render();
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Since main can't be async, we're going to need to block
    let gfx_ctx = block_on(GraphicsContext::new(&window));

    let mut state = State { gfx_ctx };

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            state.render();
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                // UPDATED!
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.gfx_ctx.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.gfx_ctx.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
