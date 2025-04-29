use std::{
    ops::{Add, Div, Sub},
    sync::Arc,
    time::Instant,
};

use cells::{
    cells::{
        Cell,
        CellKind::{self, *},
        Cells,
    },
    point::{point, Point, DOWN, LEFT, RIGHT, UP},
};
use pixels::{wgpu::Color, Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, PhysicalPosition, PhysicalSize},
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

    let cells = Cells::new();

    let mut app = App {
        window: Default::default(),
        pixels: None,
        timer: Instant::now(),
        logical_time: 0.0,
        cells,
        cursor_pos: PhysicalPosition::new(0, 0),
        mouse_down: (false, false),
        place_cell: Sand,
        place_size: 0,
        screen_pos: point(0, 0),
        view: RenderMode::Normal,
        frame_adjust: 0.0,
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
    place_size: i32,
    screen_pos: Point,
    frame_adjust: f64,
    view: RenderMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Normal,
    Thermal,
    Updates,
}

impl RenderMode {
    fn next(&mut self) {
        use RenderMode::*;

        *self = match self {
            Normal => Thermal,
            Thermal => Updates,
            Updates => Normal,
        }
    }
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
                        (true, _) => {
                            let mid = screen_to_world(cursor);
                            let size = self.place_size;

                            for n in -size..=size {
                                for m in -size..=size {
                                    self.cells.set_cell(
                                        self.screen_pos + mid + n * UP + m * LEFT,
                                        Cell::new(self.place_cell.clone()),
                                    );
                                }
                            }
                        }
                        (false, _) => (),
                    }

                    self.cells.update_all();

                    self.logical_time += 1. / (15. - self.frame_adjust);
                }

                if updates > 1 {
                    println!("catch up {}!", updates);
                }

                draw(&self.cells, pixels.frame_mut(), self.screen_pos, self.view);

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
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit7),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = Bedrock,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit8),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = Hydrogen,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Digit9),
                    winit::event::ElementState::Pressed,
                ) => self.place_cell = LiquidNitrogen,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyJ),
                    winit::event::ElementState::Pressed,
                ) => self.place_size += 1,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyK),
                    winit::event::ElementState::Pressed,
                ) => self.place_size = 0.max(self.place_size - 1),
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyW),
                    winit::event::ElementState::Pressed,
                ) => self.screen_pos = self.screen_pos + UP,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyS),
                    winit::event::ElementState::Pressed,
                ) => self.screen_pos = self.screen_pos + DOWN,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyA),
                    winit::event::ElementState::Pressed,
                ) => self.screen_pos = self.screen_pos + LEFT,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyD),
                    winit::event::ElementState::Pressed,
                ) => self.screen_pos = self.screen_pos + RIGHT,
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyT),
                    winit::event::ElementState::Pressed,
                ) => self.view.next(),
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Equal),
                    winit::event::ElementState::Pressed,
                ) => self.frame_adjust = self.frame_adjust.sub(1.0).max(0.0),
                (
                    winit::keyboard::PhysicalKey::Code(KeyCode::Minus),
                    winit::event::ElementState::Pressed,
                ) => self.frame_adjust = self.frame_adjust.add(1.0).min(14.0),
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
                        let mid = screen_to_world(cursor);
                        let size = self.place_size;

                        for n in -size..=size {
                            for m in -size..=size {
                                self.cells.set_cell(
                                    self.screen_pos + mid + n * UP + m * LEFT,
                                    Cell::new(self.place_cell.clone()),
                                );
                            }
                        }
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

fn draw(cells: &Cells, frame: &mut [u8], screen_pos: Point, render_mode: RenderMode) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % WIDTH as usize;
        let y = HEIGHT as usize - (i / WIDTH as usize) - 1;

        let cell_pos = point(x as i32, y as i32) + screen_pos;

        let rgba = match render_mode {
            RenderMode::Normal => cells.cell_at(cell_pos).colour(),
            RenderMode::Thermal => {
                let heat = cells.cell_at(cell_pos).heat.div(2.0) as u8;
                [heat, 0, 255 - heat, 255]
            }
            RenderMode::Updates => {
                if cells.was_update(cell_pos) {
                    [230, 230, 230, 255]
                } else {
                    [0, 0, 0, 255]
                }
            }
        };

        pixel.copy_from_slice(&rgba);
    }
}

fn screen_to_world(screen: LogicalPosition<u32>) -> Point {
    point(screen.x as i32, (HEIGHT - screen.y) as i32)
}
