use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(dead_code)]
mod grid;
#[allow(dead_code)]
mod renderer;
use renderer::{GraphicsContext, Vertex};

const GRID_SCALE: f32 = 1.5;

pub struct State {
    gfx_ctx: GraphicsContext,
}

fn make_grid(state: &State, rows: usize, cols: usize) -> Vec<Vertex> {
    let mut grid = Vec::new();
    let ratio = state.gfx_ctx.size.width as f32 / state.gfx_ctx.size.height as f32;
    let (sq_width, sq_height) = if ratio >= 1.0 {
        (GRID_SCALE / cols as f32 / ratio, GRID_SCALE / rows as f32)
    } else {
        (GRID_SCALE / cols as f32, GRID_SCALE / rows as f32 * ratio)
    };

    //@TODO factor in GRID_SCALE somehow so that the grid has a margin from the border of the
    //screen, make boxes touch?
    for row in 0..rows {
        for col in 0..cols {
            let low_x = ((col as f32 / cols as f32) * 2.0) - 1.0;
            let low_y = ((row as f32 / rows as f32) * 2.0) - 1.0;

            let up_x = low_x + sq_width;
            let up_y = low_y + sq_height;

            /*
            println!(
                "low_x {} low_y {} high_x {} high_y {}",
                low_x, low_y, up_x, up_y
            );
            */

            let verts: &[Vertex] = &[
                // lower left triangle
                Vertex {
                    position: [low_x, low_y, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [up_x, low_y, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [low_x, up_y, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
                // upper right triangle
                Vertex {
                    position: [low_x, up_y, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [up_x, low_y, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [up_x, up_y, 0.0],
                    color: [0.0, 0.0, 1.0],
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
                    b: 0.0,
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

        let verts = make_grid(self, 5, 5);

        /*
        let verts: &[Vertex] = &[
            // lower left triangle
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            // upper right triangle
            Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let verts: &[Vertex] = &[
            // lower left triangle
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            // upper right triangle
            Vertex {
                position: [0.0, 0.6, 0.0],
                color: [1.0, 0.0, 0.0],
            },
        ];

        let indices: &[u16] = &[1, 2, 3];

        self.gfx_ctx.draw_indexed(&verts, indices);
        */

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
