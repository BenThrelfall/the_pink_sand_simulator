use macroquad::{
    input::{
        KeyCode, MouseButton, is_key_down, is_key_pressed, is_key_released, is_mouse_button_down,
        is_mouse_button_pressed, is_mouse_button_released,
    },
    time::get_time,
};
use std::collections::HashMap;

macro_rules! create_input_actions {
    {$(($name:ident, $defaults:expr, $mouse:expr)),+ $(,)?} => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Ina {
            $(
                $name
            ),+
        }

        impl InputManager {
            pub fn new() -> Self {
                let mappings = HashMap::from([
                    $(
                        (Ina::$name, ($defaults, $mouse))
                    ),+
                ]);

                Self { mappings, captured: false, hold_press: std::cell::Cell::new(0.0)}
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct InputManager {
    captured: bool,
    hold_press: std::cell::Cell<f64>,
    mappings: HashMap<Ina, (Vec<KeyCode>, Vec<MouseButton>)>,
}

impl InputManager {
    pub fn reset(&mut self) {
        self.captured = false;
    }

    pub fn capture(&mut self) {
        self.captured = true;
    }

    pub fn pressed(&self, action: Ina) -> bool {
        if self.captured {
            return false;
        }

        let (keys, mouse) = self.mappings.get(&action).unwrap();

        keys.iter().any(|&key| is_key_pressed(key))
            || mouse.iter().any(|&button| is_mouse_button_pressed(button))
    }

    pub fn hold_pressed(&self, action: Ina) -> bool {
        if self.captured {
            return false;
        }

        let (keys, mouse) = self.mappings.get(&action).unwrap();

        if keys.iter().any(|&key| is_key_pressed(key))
            || mouse.iter().any(|&button| is_mouse_button_pressed(button))
        {
            self.hold_press.set(get_time() + 1.0);
            return true;
        }

        if get_time() > self.hold_press.get() && keys.iter().any(|&key| is_key_down(key)) {
            self.hold_press.update(|n| n + 0.1);
            return true;
        }

        return false;
    }

    pub fn down(&self, action: Ina) -> bool {
        if self.captured {
            return false;
        }

        let (keys, mouse) = self.mappings.get(&action).unwrap();

        keys.iter().any(|&key| is_key_down(key))
            || mouse.iter().any(|&button| is_mouse_button_down(button))
    }

    pub fn released(&self, action: Ina) -> bool {
        if self.captured {
            return false;
        }

        let (keys, mouse) = self.mappings.get(&action).unwrap();

        keys.iter().any(|&key| is_key_released(key))
            || mouse.iter().any(|&button| is_mouse_button_released(button))
    }
}

use KeyCode::*;
use MouseButton::{Left, Middle, Right};
create_input_actions! {
    (Undo, vec![Backspace], vec![]),
    (LogicalShift, vec![LeftShift, RightShift], vec![]),
    (Dash, vec![Space], vec![]),
    (PrimaryTarget, vec![], vec![Left]),
    (SecondaryTarget, vec![], vec![Right]),
    (TimeMagicCast, vec![KeyCode::F], vec![]),
    (Pan, vec![], vec![Middle]),
    (MoveLeft, vec![KeyCode::A, KeyCode::Left], vec![]),
    (MoveRight, vec![KeyCode::D, KeyCode::Right], vec![]),
    (MoveUp, vec![KeyCode::W, KeyCode::Up], vec![]),
    (MoveDown, vec![KeyCode::S, KeyCode::Down], vec![]),
    (CameraZoomIn, vec![KeyCode::E, KeyCode::KpAdd], vec![]),
    (CameraZoomOut, vec![KeyCode::Q, KeyCode::KpSubtract], vec![]),
    (OpenDevConsole, vec![KeyCode::Semicolon], vec![]),
    (CloseDevConsole, vec![KeyCode::Escape], vec![]),
    (DevSubmitCommand, vec![KeyCode::Enter], vec![]),
    (DevStepForward, vec![KeyCode::Period], vec![]),
    (DevStepBackwards, vec![KeyCode::Comma], vec![]),
    (UIClick, vec![], vec![Left])
}
