use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Timer {
    start_time: Option<Instant>,
    elapsed: Duration,
    is_paused: bool,
}

impl Timer {
    // Create a new timer instance
    pub fn new() -> Timer {
        Timer {
            start_time: None,
            elapsed: Duration::new(0, 0),
            is_paused: false,
        }
    }

    // Start the timer
    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        } else if self.is_paused {
            // Resume from where it was paused
            self.start_time = Some(Instant::now() - self.elapsed);
            self.is_paused = false;
        }
    }

    // Stop the timer
    pub fn stop(&mut self) {
        if let Some(start_time) = self.start_time {
            self.elapsed = start_time.elapsed();
            self.start_time = None;
            self.is_paused = false;
        }
    }

    // Pause the timer
    pub fn pause(&mut self) {
        if let Some(start_time) = self.start_time {
            self.elapsed = start_time.elapsed();
            self.is_paused = true;
            self.start_time = None;
        }
    }

    // Reset the timer
    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::new(0, 0);
        self.is_paused = false;
    }

    // Get the current elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        if let Some(start_time) = self.start_time {
            if self.is_paused {
                self.elapsed.as_millis()
            } else {
                start_time.elapsed().as_millis()
            }
        } else {
            self.elapsed.as_millis()
        }
    }
}

#[test]
fn test_time() {
    let mut timer = Timer::new();
    
    timer.start();
    // Do some work here...
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    println!("Elapsed time: {} ms", timer.elapsed_ms());
    
    timer.pause();
    println!("Paused time: {} ms", timer.elapsed_ms());
    
    std::thread::sleep(std::time::Duration::from_millis(200)); // Simulate time passing during pause
    
    timer.start(); // Resume
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    println!("Elapsed time after resume: {} ms", timer.elapsed_ms());
    
    timer.stop();
    println!("Final time: {} ms", timer.elapsed_ms());
    
    timer.reset();
    println!("After reset: {} ms", timer.elapsed_ms());
}

