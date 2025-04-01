use std::{sync::Arc, time::Instant};

use cells::cells::{
    Cell,
    CellKind::{self, *},
    Cells,
};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::Window,
};

const WIDTH: u32 = 480;
const HEIGHT: u32 = 270;

fn main() {
    // Create an event loop.
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut cells = Cells::new(WIDTH as usize, HEIGHT as usize);

    cells.set_cell(21, 22, Cell::new(Water));
    cells.set_cell(20, 21, Cell::new(Water));
    cells.set_cell(21, 21, Cell::new(Water));
    cells.set_cell(21, 20, Cell::new(Water));
    cells.set_cell(22, 20, Cell::new(Water));

    let mut app = App {
        window: Default::default(),
        pixels: None,
        timer: Instant::now(),
        logical_time: 0.0,
        cells,
        cursor_pos: PhysicalPosition::new(0, 0),
        mouse_down: (false, false),
        place_cell: Sand,
    };

    event_loop.run_app(&mut app).unwrap();
}

#[derive(Debug)]
struct App<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    timer: Instant,
    logical_time: f64,
    cells: Cells,
    cursor_pos: PhysicalPosition<u32>,
    mouse_down: (bool, bool),
    place_cell: CellKind,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(3 * WIDTH, 3 * HEIGHT)),
                )
                .unwrap()
                .into(),
        );

        self.pixels = {
            let window_size = self.window.as_ref().unwrap().inner_size();
            let this = self.window.as_ref().unwrap().to_owned();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, this);
            Pixels::new(WIDTH, HEIGHT, surface_texture).ok()
        };

        self.timer = Instant::now();
        self.logical_time = 0.0;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let _ = window_id;
        let pixels = self.pixels.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();

        match event {
            WindowEvent::RedrawRequested => {
                let mut updates = 0;

                while self.logical_time < self.timer.elapsed().as_secs_f64() {
                    updates += 1;

                    let cursor = self
                        .cursor_pos
                        .to_logical::<u32>(window.inner_size().width as f64 / WIDTH as f64);

                    match self.mouse_down {
                        (true, _) => self.cells.set_cell(
                            cursor.x as usize,
                            cursor.y as usize,
                            Cell::new(self.place_cell.clone()),
                        ),
                        (false, _) => (),
                    }

                    self.cells.update_all();

                    self.logical_time += 1. / 15.;
                }

                if updates > 1 {
                    println!("catch up {}!", updates);
                }

                draw(&self.cells, pixels.frame_mut());
                pixels.render().unwrap();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CloseRequested => {
                //io::stdout().flush().unwrap();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => match (event.physical_key, event.state) {
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Escape),
                    winit::event::ElementState::Pressed,
                ) => event_loop.exit(),
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit1),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = Sand,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit2),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = Water,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit3),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = Honey,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit4),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = PinkSand,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit5),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = BlueSand,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit6),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = PurpleSand,
                _ => (),
            },
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match (state, button) {
                (winit::event::ElementState::Pressed, winit::event::MouseButton::Left) => {
                    self.mouse_down.0 = true;
                }
                (winit::event::ElementState::Released, winit::event::MouseButton::Left) => {
                    self.mouse_down.0 = false;
                }
                (winit::event::ElementState::Pressed, winit::event::MouseButton::Right) => {
                    self.mouse_down.1 = true;
                }
                (winit::event::ElementState::Released, winit::event::MouseButton::Right) => {
                    self.mouse_down.1 = false;
                }
                _ => (),
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_pos = position.cast();
                let cursor = self
                    .cursor_pos
                    .to_logical::<u32>(window.inner_size().width as f64 / WIDTH as f64);

                match self.mouse_down {
                    (true, _) => {
                        self.cells.set_cell(
                            cursor.x as usize,
                            cursor.y as usize,
                            Cell::new(self.place_cell.clone()),
                        );
                    }
                    _ => (),
                }
            }
            WindowEvent::Resized(size) => {
                pixels.resize_surface(size.width, size.height).unwrap();
            }
            _ => {}
        }
    }
}

fn draw(cells: &Cells, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % WIDTH as usize;
        let y = i / WIDTH as usize;

        let rgba = cells.cell_at(x, y).colour();

        pixel.copy_from_slice(&rgba);
    }
}
