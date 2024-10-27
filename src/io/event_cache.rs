extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::collections::HashSet;

pub struct EventCache {
    event_pump: EventPump,
    quit: bool,
    key_pressed: HashSet<Keycode>,
    key_released: HashSet<Keycode>,
    key_held: HashSet<Keycode>,
    key_changed_to_pressed: HashSet<Keycode>,
    key_changed_to_released: HashSet<Keycode>,
}

impl EventCache {
    pub fn new(event_pump: EventPump) -> Self {
        EventCache {
            event_pump,
            quit: false,
            key_pressed: HashSet::new(),
            key_released: HashSet::new(),
            key_held: HashSet::new(),
            key_changed_to_pressed: HashSet::new(),
            key_changed_to_released: HashSet::new(),
        }
    }

    pub fn poll_events(&mut self) {
        // Clear previous frame's pressed and released keys
        self.key_pressed.clear();
        self.key_released.clear();
        self.key_changed_to_pressed.clear();
        self.key_changed_to_released.clear();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.quit = true;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        if !self.key_held.contains(&keycode) {
                            self.key_changed_to_pressed.insert(keycode);
                        }
                        self.key_pressed.insert(keycode);
                        self.key_held.insert(keycode);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if self.key_held.contains(&keycode) {
                        self.key_changed_to_released.insert(keycode);
                    }
                    self.key_released.insert(keycode);
                    self.key_held.remove(&keycode);
                }
                _ => {}
            }
        }
    }

    pub fn is_quit(&self) -> bool {
        self.quit
    }

    pub fn is_key_pressed(&self, keycode: Keycode) -> bool {
        self.key_pressed.contains(&keycode)
    }

    pub fn is_key_released(&self, keycode: Keycode) -> bool {
        self.key_released.contains(&keycode)
    }

    pub fn is_key_held(&self, keycode: Keycode) -> bool {
        self.key_held.contains(&keycode)
    }

    pub fn is_key_changed_to_pressed(&self, keycode: Keycode) -> bool {
        self.key_changed_to_pressed.contains(&keycode)
    }

    pub fn is_key_changed_to_released(&self, keycode: Keycode) -> bool {
        self.key_changed_to_released.contains(&keycode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use std::collections::VecDeque;

    #[allow(dead_code)]
    struct MockEventPump {
        events: VecDeque<Event>,
    }

    impl MockEventPump {
        #[allow(dead_code)]
        fn new(events: Vec<Event>) -> Self {
            MockEventPump {
                events: events.into_iter().collect(),
            }
        }
    }

    impl Iterator for MockEventPump {
        type Item = Event;

        fn next(&mut self) -> Option<Self::Item> {
            self.events.pop_front()
        }
    }

    fn mock_event_pump(_events: Vec<Event>) -> EventPump {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        event_pump
    }

    #[test]
    fn test_is_quit() {
        let events = vec![Event::Quit { timestamp: 0 }];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_quit(), true);
    }

    #[test]
    fn test_key_pressed() {
        let events = vec![Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Escape),
            scancode: None,
            keymod: sdl2::keyboard::Mod::empty(),
            repeat: false,
        }];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_key_pressed(Keycode::Escape), true);
    }

    #[test]
    fn test_key_released() {
        let events = vec![Event::KeyUp {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Escape),
            scancode: None,
            keymod: sdl2::keyboard::Mod::empty(),
                repeat: false,
        }];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_key_released(Keycode::Escape), true);
    }

    #[test]
    fn test_key_held() {
        let events = vec![Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::Space),
            scancode: None,
            keymod: sdl2::keyboard::Mod::empty(),
            repeat: false,
        }];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_key_held(Keycode::Space), true);
    }

    #[test]
    fn test_key_changed_to_pressed() {
        let events = vec![Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::A),
            scancode: None,
            keymod: sdl2::keyboard::Mod::empty(),
            repeat: false,
        }];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_key_changed_to_pressed(Keycode::A), true);
    }

    #[test]
    fn test_key_changed_to_released() {
        let events = vec![
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::A),
                scancode: None,
                keymod: sdl2::keyboard::Mod::empty(),
                repeat: false,
            },
            Event::KeyUp {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::A),
                scancode: None,
                keymod: sdl2::keyboard::Mod::empty(),
                repeat: false,
            },
        ];
        let mut event_cache = EventCache::new(mock_event_pump(events));
        event_cache.poll_events();
        assert_eq!(event_cache.is_key_changed_to_released(Keycode::A), true);
    }
}
