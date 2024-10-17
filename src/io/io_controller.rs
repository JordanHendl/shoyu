use std::collections::HashMap;

use sdl2::keyboard::Keycode;

use super::{EventCache, JoystickController};

pub struct IOController {
    event_cache: EventCache,
    joystick_controller: Option<JoystickController>,
    action_map: HashMap<String, Vec<Keycode>>, // Maps action strings to keycodes
}

impl IOController {
    pub fn new(sdl_ctx: &mut sdl2::Sdl) -> Self {
        IOController {
            event_cache: EventCache::new(sdl_ctx.event_pump().unwrap()),
            joystick_controller: None,
            action_map: HashMap::new(),
        }
    }
    pub fn event_cache(&self) -> &EventCache {
        &self.event_cache
    }

    pub fn map_action(&mut self, action: &str, keycodes: Vec<Keycode>) {
        self.action_map.insert(action.to_string(), keycodes);
    }

    pub fn is_action_active(&self, action: &str) -> bool {
        if let Some(keycodes) = self.action_map.get(action) {
            for keycode in keycodes {
                if self.event_cache.is_key_held(*keycode) {
                    return true;
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
        if let Some(keycodes) = self.action_map.get(action) {
            for keycode in keycodes {
                if self.event_cache.is_key_changed_to_pressed(*keycode) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_action_released(&self, action: &str) -> bool {
        if let Some(keycodes) = self.action_map.get(action) {
            for keycode in keycodes {
                if self.event_cache.is_key_changed_to_released(*keycode) {
                    return true;
                }
            }
        }
        false
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
