use std::{
    mem::swap,
    sync::Arc,
    time::Instant,
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

    cells.set_cell(21, 22, true);
    cells.set_cell(20, 21, true);
    cells.set_cell(21, 21, true);
    cells.set_cell(21, 20, true);
    cells.set_cell(22, 20, true);

    let mut app = App {
        window: Default::default(),
        pixels: None,
        timer: Instant::now(),
        logical_time: 0.0,
        cells,
        cursor_pos: PhysicalPosition::new(0, 0),
        mouse_down: false,
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
    mouse_down: bool,
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

                    //let func_time = Instant::now();
                    self.cells.update();

                    if self.logical_time % 2.0 < 0.1 {
                        //   println!("{:.2}", func_time.elapsed().as_secs_f64() * 1000.0);
                    }

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
                _ => (),
            },
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match (state, button) {
                (winit::event::ElementState::Pressed, winit::event::MouseButton::Left) => {
                    self.mouse_down = true;
                }
                (winit::event::ElementState::Released, winit::event::MouseButton::Left) => {
                    self.mouse_down = false;
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
                if self.mouse_down {
                    self.cells
                        .set_cell(cursor.x as usize, cursor.y as usize, true);
                }
            }
            WindowEvent::Resized(size) => {
                pixels.resize_surface(size.width, size.height).unwrap();
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
struct Cells {
    x_size: usize,
    y_size: usize,
    data: Vec<bool>,
    data_two: Vec<bool>,
}

fn draw(cells: &Cells, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % WIDTH as usize;
        let y = i / WIDTH as usize;

        let rgba = if cells.cell_at(x, y) {
            [0xaf, 0xaf, 0xaf, 0xff]
        } else {
            [0x48, 0x48, 0x48, 0xff]
        };

        pixel.copy_from_slice(&rgba);
    }
}

impl Cells {
    pub fn new(x_size: usize, y_size: usize) -> Cells {
        Cells {
            data: vec![false; x_size * y_size],
            x_size,
            y_size,
            data_two: vec![false; x_size * y_size],
        }
    }

    #[inline]
    pub fn cell_at(&self, x: usize, y: usize) -> bool {
        self.data[x * self.y_size + y]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.x_size && y < self.y_size {
            self.data[x * self.y_size + y] = alive;
        }
    }

    fn set_cell_next(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.x_size && y < self.y_size {
            self.data_two[x * self.y_size + y] = alive;
        }
    }

    pub fn neighbours(&self, x: usize, y: usize) -> usize {
        self.cell_at(x - 1, y - 1) as usize
            + self.cell_at(x, y - 1) as usize
            + self.cell_at(x + 1, y - 1) as usize
            + self.cell_at(x - 1, y) as usize
            + self.cell_at(x + 1, y) as usize
            + self.cell_at(x - 1, y + 1) as usize
            + self.cell_at(x, y + 1) as usize
            + self.cell_at(x + 1, y + 1) as usize
    }

    pub fn update(&mut self) {
        for x in 1..self.x_size - 1 {
            for y in 1..self.y_size - 1 {
                let neighbours = self.neighbours(x, y);

                if self.cell_at(x, y) {
                    match neighbours {
                        ..2 | 4.. => self.set_cell_next(x, y, false),
                        2..=3 => self.set_cell_next(x, y, true),
                    }
                } else {
                    match neighbours {
                        3 => self.set_cell_next(x, y, true),
                        _ => self.set_cell_next(x, y, false),
                    }
                }
            }
        }

        swap(&mut self.data, &mut self.data_two);
    }
}
