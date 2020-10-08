use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use imgui::im_str;

#[allow(dead_code)]
mod grid;
use grid::{Grid, GridKind, SolverKind};

#[allow(dead_code)]
mod renderer;
use renderer::GraphicsContext;

pub struct State {
    pub gfx_ctx: GraphicsContext,
    pub grid: Grid,

    pub last_x: f32,
    pub last_y: f32,
}

impl State {
    // returns false if event hasn't been fully processed
    fn input(&mut self, event: &WindowEvent, kind: GridKind) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.last_x = position.x as f32 / self.gfx_ctx.size.width as f32;
                self.last_y = position.y as f32 / self.gfx_ctx.size.height as f32;
                /*
                self.gfx_ctx.clear_color = wgpu::Color {
                    r: self.last_x as f64,
                    g: self.last_y as f64,
                    b: 0.6,
                    a: 1.0,
                };
                */
                true
            }
            WindowEvent::MouseInput { state, .. } => {
                if state == &ElementState::Pressed {
                    self.grid
                        .handle_click((self.last_x, self.last_y), self.gfx_ctx.size, kind);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.gfx_ctx.start(view, device, queue);

        let verts = self.grid.render(self);

        self.gfx_ctx.draw(&verts, view, device);

        self.gfx_ctx.render(queue);
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let hidpi_factor = window.scale_factor();
    // Since main can't be async, we're going to need to block
    let grid = Grid::new();

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let size = window.inner_size();

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, mut queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        },
        None, // Trace path
    ))
    .unwrap();

    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Set up dear imgui
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

    let gfx_ctx = block_on(GraphicsContext::new(&window, &device, &sc_desc));

    #[cfg(not(feature = "glsl-to-spirv"))]
    let mut renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &mut queue, sc_desc.format);

    #[cfg(feature = "glsl-to-spirv")]
    let mut renderer =
        imgui_wgpu::Renderer::new_glsl(&mut imgui, &device, &mut queue, sc_desc.format);

    let mut state = State {
        gfx_ctx,
        grid,
        last_x: 0.0,
        last_y: 0.0,
    };

    let mut last_frame = std::time::Instant::now();
    let mut last_cursor = None;
    let mut show_demo = false;
    let mut grid_kind = GridKind::Wall;
    let mut expanded_solve_running = false;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = window.inner_size();

                let sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Fifo,
                };

                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                state.gfx_ctx.resize(size);
            }
            Event::RedrawRequested(_) => {
                let delta_s = last_frame.elapsed();
                let now = std::time::Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();

                {
                    let window = imgui::Window::new(im_str!("Hello world"));
                    window
                        .size([500.0, 200.0], imgui::Condition::FirstUseEver)
                        .build(&ui, || {
                            ui.text(im_str!("Frametime: {:?}", delta_s));
                            ui.separator();
                            let mouse_pos = ui.io().mouse_pos;
                            ui.text(im_str!(
                                "Mouse Position: ({:.1},{:.1})",
                                mouse_pos[0],
                                mouse_pos[1]
                            ));

                            ui.separator();
                            if ui.button(im_str!("Toggle Demo"), [100., 20.]) {
                                show_demo = !show_demo
                            }

                            ui.separator();

                            if ui.radio_button(im_str!("Wall"), &mut grid_kind, GridKind::Wall) {
                                grid_kind = GridKind::Wall;
                            }
                            if ui.radio_button(im_str!("Start"), &mut grid_kind, GridKind::Start) {
                                grid_kind = GridKind::Start;
                            }
                            if ui.radio_button(im_str!("Goal"), &mut grid_kind, GridKind::Goal) {
                                grid_kind = GridKind::Goal;
                            }

                            ui.separator();

                            if ui.radio_button(
                                im_str!("BFS"),
                                &mut state.grid.solver_kind,
                                SolverKind::BFS,
                            ) {
                                state.grid.solver_kind = SolverKind::BFS;
                            }
                            if ui.radio_button(
                                im_str!("DFS"),
                                &mut state.grid.solver_kind,
                                SolverKind::DFS,
                            ) {
                                state.grid.solver_kind = SolverKind::DFS;
                            }
                            if ui.radio_button(
                                im_str!("A Star"),
                                &mut state.grid.solver_kind,
                                SolverKind::AStar,
                            ) {
                                state.grid.solver_kind = SolverKind::AStar;
                            }

                            ui.separator();

                            if ui.button(im_str!("Solve!"), [100., 20.]) {
                                state.grid.solve_path();
                            }
                            ui.separator();
                            if ui.button(im_str!("Step Solver"), [100., 20.]) {
                                state.grid.step_solve_path();
                            }
                            ui.separator();
                            if ui.button(im_str!("Expanded Solve"), [100., 20.]) {
                                state.grid.graph = None;
                                expanded_solve_running = !expanded_solve_running;
                            }
                        });

                    if show_demo {
                        ui.show_demo_window(&mut false);
                    }
                }

                if expanded_solve_running {
                    expanded_solve_running = state.grid.step_solve_path();
                }

                state.update();
                state.render(&frame.output.view, &device, &queue);

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    platform.prepare_render(&ui, &window);
                }

                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                renderer
                    .render(ui.render(), &queue, &device, &mut rpass)
                    .expect("Rendering failed");

                drop(rpass);

                queue.submit(Some(encoder.finish()));
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
                if !state.input(event, grid_kind) {
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
        }
        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
