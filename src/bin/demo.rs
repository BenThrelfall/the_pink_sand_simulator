use std::{rc::Rc, time::Instant};

use macroquad::prelude::*;

use the_pink_sand_simulator::{
    cells::{self, Cell, CellKind::*, Cells, cell},
    input::{Ina, InputManager},
    point::{Point, point},
};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 900;

#[macroquad::main("Cell World")]
async fn main() {
    let inputs = Rc::new(InputManager::new());
    set_default_filter_mode(FilterMode::Nearest);

    let render_target = render_target_ex(
        WIDTH,
        HEIGHT,
        RenderTargetParams {
            sample_count: 0,
            depth: false,
        },
    );
    render_target.texture.set_filter(FilterMode::Nearest);

    let camera = Camera2D {
        zoom: vec2(2. / 1600.0, 2. / 900.0),
        target: vec2(1600.0 / 2., 900.0 / 2.),
        render_target: Some(render_target),
        ..Default::default()
    };

    let mut cam_control = CameraControl {
        inputs: inputs.clone(),
        panning: None,
        camera_final_target: Vec2::ZERO,
        camera_smooth_target: Vec2::ZERO,
        zoom_level: 2.0,
        camera,
    };

    let mut cells = Cells::new();

    for y in 10..100 {
        for x in -50..50 {
            let offset = y % 2;
            cells.set_cell(point(2 * x + offset, y * -10), cell(Bedrock));
        }
    }

    let mut buffer = Box::new([0u8; WIDTH as usize * HEIGHT as usize * 4]);

    loop {
        trace!("!! New Frame !!");
        cam_control.direct_camera_control();
        cam_control.camera_zoom_control();

        set_camera(&cam_control.camera);
        clear_background(DARKGRAY);

        let timer = Instant::now();
        draw(
            &cells,
            buffer.as_mut(),
            point(
                (cam_control.camera_final_target.x - 800.0) as i32,
                -(cam_control.camera_final_target.y + 450.0) as i32,
            ),
        );
        trace!("fn draw: {}", timer.elapsed().as_micros());

        let timer = Instant::now();
        cam_control
            .camera
            .render_target
            .as_ref()
            .unwrap()
            .texture
            .update_from_bytes(WIDTH, HEIGHT, buffer.as_ref());
        trace!("update from bytes: {}", timer.elapsed().as_micros());

        let mouse = cam_control.world_mouse_pos();
        if inputs.down(Ina::PrimaryTarget) {
            cells.set_cell(
                point(mouse.x as i32, -mouse.y as i32),
                Cell::new(cells::CellKind::Water),
            );
        }

        set_default_camera();
        cam_control.draw_world_to_screen();

        draw_text(
            &cam_control.camera_final_target.to_string(),
            30.,
            30.,
            36.,
            WHITE,
        );
        draw_text(&mouse.to_string(), 30., 130., 36., WHITE);
        draw_fps();

        for x in -1..=1 {
            cells.set_cell(point(x, 0), cell(Water));
        }

        let timer = Instant::now();
        cells.update_all();
        trace!("cells update_all: {}", timer.elapsed().as_micros());

        next_frame().await;
    }
}

fn draw(cells: &Cells, frame: &mut [u8], screen_pos: Point) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = i % WIDTH as usize;
        let y = HEIGHT as usize - (i / WIDTH as usize) - 1;

        let rgba = cells
            .cell_at(point(x as i32, y as i32) + screen_pos)
            .colour();

        pixel.copy_from_slice(&rgba);
    }
}

struct CameraControl {
    pub inputs: Rc<InputManager>,
    pub panning: Option<Vec2>,
    pub camera_final_target: Vec2,
    pub camera_smooth_target: Vec2,
    pub zoom_level: f32,
    pub camera: Camera2D,
}

impl CameraControl {
    fn camera_zoom_control(&mut self) {
        if self.inputs.pressed(Ina::CameraZoomIn) {
            self.zoom_level += 1.;
        }
        if self.inputs.pressed(Ina::CameraZoomOut) && self.zoom_level > 1. {
            self.zoom_level -= 1.;
        }
    }

    fn direct_camera_control(&mut self) {
        let move_amount = (600. - self.zoom_level * 60.).max(60.) * get_frame_time();

        if self.inputs.down(Ina::MoveUp) {
            self.camera_smooth_target += vec2(0., -move_amount);
        }
        if self.inputs.down(Ina::MoveLeft) {
            self.camera_smooth_target += vec2(-move_amount, 0.);
        }
        if self.inputs.down(Ina::MoveDown) {
            self.camera_smooth_target += vec2(0., move_amount);
        }
        if self.inputs.down(Ina::MoveRight) {
            self.camera_smooth_target += vec2(move_amount, 0.);
        }

        let mouse_pos = self.world_mouse_pos();
        let pan_input = self.inputs.down(Ina::Pan);

        //Handling Panning
        match (self.panning, pan_input) {
            (None, true) => {
                self.panning = Some(mouse_pos);
            }
            (Some(origin), true) => {
                let delta = origin - mouse_pos;
                self.camera_smooth_target += delta;
            }
            (Some(_), false) => {
                self.panning = None;
            }
            (None, false) => (),
        }

        self.camera_final_target = self.camera_smooth_target;
        self.camera.target = self.camera_final_target.floor();
    }

    fn draw_world_to_screen(&mut self) {
        let timer = Instant::now();
        let subpixel_adjust = self.camera_final_target.fract_gl();

        assert!(subpixel_adjust.x < 1. && subpixel_adjust.y < 1.);

        let final_size = vec2(1600., 900.) * self.zoom_level;

        let x_draw_offset =
            -subpixel_adjust.x * self.zoom_level + (screen_width() - final_size.x) / 2.;
        let y_draw_offset =
            -subpixel_adjust.y * self.zoom_level + (screen_height() - final_size.y) / 2.;

        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            x_draw_offset.floor(),
            y_draw_offset.floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(final_size),
                ..Default::default()
            },
        );
        trace!("Draw World Texture: {}", timer.elapsed().as_micros());
    }

    pub fn world_mouse_pos(&self) -> Vec2 {
        let screen_pos: Vec2 = mouse_position().into();
        let centred = screen_pos - vec2(screen_width(), screen_height()) / 2.;
        self.camera_final_target + centred / self.zoom_level
    }
}
