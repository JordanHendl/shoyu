extern crate sdl2;

use sdl2::keyboard::Keycode;
use std::collections::HashMap;

use crate::io::EventCache;

use sdl2::controller::GameController;
use sdl2::joystick::{HatState, Joystick};

pub struct JoystickController {
    joystick: Joystick,
    controller: Option<GameController>,
    action_map: HashMap<String, HatState>, // Maps action strings to joystick hat states
}

pub struct JoystickAngles {
    pub angle: f32,
    pub magnitude: f32,
}
impl JoystickController {
    pub fn new(joystick: Joystick, controller: Option<GameController>) -> Self {
        JoystickController {
            joystick,
            controller,
            action_map: HashMap::new(),
        }
    }

    pub fn map_action(&mut self, action: &str, hat_state: HatState) {
        self.action_map.insert(action.to_string(), hat_state);
    }

    pub fn is_action_active(&self, action: &str) -> bool {
        if let Some(&hat_state) = self.action_map.get(action) {
            if let Ok(current_state) = self.joystick.hat(0) {
                return current_state == hat_state;
            }
        }
        false
    }

    pub fn get_joystick_info(&self) -> Option<JoystickAngles> {
        let axis_x = self.joystick.axis(0).ok()?;
        let axis_y = self.joystick.axis(1).ok()?;

        let x = axis_x as f32 / i16::MAX as f32;
        let y = axis_y as f32 / i16::MAX as f32;

        if x == 0.0 && y == 0.0 {
            return None;
        }

        let magnitude = (x * x + y * y).sqrt();

        let mut angle = y.atan2(x).to_degrees();
        angle = if angle < 0.0 { angle + 360.0 } else { angle };

        Some(JoystickAngles {
            angle,
            magnitude,
        })
    }
    
    pub fn joystick(&self) -> &Joystick {
        &self.joystick
    }
}
