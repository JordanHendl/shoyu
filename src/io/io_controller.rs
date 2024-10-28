use std::collections::HashMap;

use super::{EventCache, JoystickController};
use sdl2::{keyboard::Keycode, mouse::MouseButton};

#[derive(Clone, Copy, Debug)]
pub struct MousePosition {
    pub position: (f32, f32),
    pub delta: (f32, f32),
}

enum Action {
    Key(Keycode),
    MouseButton(MouseButton),
}

pub struct IOController {
    event_cache: EventCache,
    joystick_controller: Option<JoystickController>,
    action_map: HashMap<String, Vec<Action>>, // Maps action strings to keycodes
}

impl IOController {
    pub fn new(sdl_ctx: &mut sdl2::Sdl) -> Self {
        let event_pump = sdl_ctx.event_pump().unwrap();
        IOController {
            event_cache: EventCache::new(event_pump),
            joystick_controller: None,
            action_map: HashMap::new(),
        }
    }
    pub fn event_cache(&self) -> &EventCache {
        &self.event_cache
    }

    pub fn map_action_keys(&mut self, action: &str, keycodes: Vec<Keycode>) {
        let actions: Vec<Action> = keycodes.into_iter().map(|a| Action::Key(a)).collect();
        self.action_map.insert(action.to_string(), actions);
    }

    pub fn map_action_buttons(&mut self, action: &str, keycodes: Vec<MouseButton>) {
        let actions: Vec<Action> = keycodes.into_iter().map(|a| Action::MouseButton(a)).collect();
        self.action_map.insert(action.to_string(), actions);
    }


    pub fn is_action_active(&self, action: &str) -> bool {
        if let Some(actions) = self.action_map.get(action) {
            for action in actions {
                match action {
                    Action::Key(keycode) => {
                        if self.event_cache.is_key_held(*keycode) {
                            return true;
                        }
                    }
                    Action::MouseButton(button) => {
                        if self.event_cache.is_mouse_held(*button) {
                            return true;
                        }
                    }
                }
            }
        }

        if let Some(joystick) = &self.joystick_controller {
            if joystick.is_action_active(action) {
                return true;
            }
        }

        false
    }

    pub fn is_action_pressed(&self, action: &str) -> bool {
        if let Some(actions) = self.action_map.get(action) {
            for action in actions {
                match action {
                    Action::Key(keycode) => {
                        if self.event_cache.is_key_changed_to_pressed(*keycode) {
                            return true;
                        }
                    }
                    Action::MouseButton(button) => {
                        if self.event_cache.is_mouse_pressed(*button) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn is_action_released(&self, action: &str) -> bool {
        if let Some(actions) = self.action_map.get(action) {
            for action in actions {
                match action {
                    Action::Key(keycode) => {
                        if self.event_cache.is_key_changed_to_released(*keycode) {
                            return true;
                        }
                    }
                    Action::MouseButton(button) => {
                        if self.event_cache.is_mouse_released(*button) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn get_mouse_position(&self) -> MousePosition {
        let pos = self.event_cache.get_mouse_position();
        let last = self.event_cache.get_last_mouse_position();
        return MousePosition {
            position: (pos.0 as f32, pos.1 as f32),
            delta: ((pos.0 - last.0) as f32, (pos.1 - last.1) as f32),
        };
    }

    pub fn get_mouse_position_interp(&self, width: f32, height: f32) -> MousePosition {
        let pos = self.event_cache.get_mouse_position();
        let last = self.event_cache.get_last_mouse_position();
        return MousePosition {
            position: (pos.0 as f32 / width, pos.1 as f32 / height),
            delta: (
                (pos.0 - last.0) as f32 / width,
                (pos.1 - last.1) as f32 / height,
            ),
        };
    }

    pub fn update(&mut self) {
        self.event_cache.poll_events();

        //TODO make query at low low low rate
        //
        // Query system for new joysticks and add them if detected
        //        let sdl_context = sdl2::init().unwrap();
        //        let joystick_subsystem = sdl_context.joystick().unwrap();

        //        for i in 0..joystick_subsystem.num_joysticks().unwrap() {
        //            if self.joystick_controller.is_none() {
        //                if let Ok(joystick) = joystick_subsystem.open(i) {
        //                    self.joystick_controller = Some(JoystickController::new(joystick, None));
        //                }
        //            }
        //        }
    }

    pub fn joystick(&self) -> Option<&JoystickController> {
        self.joystick_controller.as_ref()
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::event::Event;
    use sdl2::joystick::Joystick;
    use sdl2::keyboard::Keycode;

    #[test]
    fn test_io_controller() {
        //        let sdl_context = sdl2::init().unwrap();
        //        let event_pump = sdl_context.event_pump().unwrap();
        //        let joystick = Joystick::open(0).unwrap();
        //        let joystick_controller = JoystickController::new(joystick, None);
        //        let mut event_cache = EventCache::new(event_pump);
        //        let mut io_controller = IOController::new(event_cache, Some(joystick_controller));
        //
        //        io_controller.map_action("kick", vec![Keycode::A]);
        //        io_controller.map_action("punch", vec![Keycode::B]);
        //
        //        // Simulate event
        //        io_controller.event_cache.poll_events();
        //        assert_eq!(io_controller.is_action_active("kick"), false);
        //        assert_eq!(io_controller.is_action_active("punch"), false);
    }
}
