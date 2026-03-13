mod logger;
pub mod ui;

use std::cell::Cell;

pub use logger::*;

/// Simple boolean signal
#[derive(Default)]
pub struct Signal(Cell<bool>, u32);

impl Signal {
    /// Request the signal to be set to true after a given number of frames
    pub fn request_delayed(&mut self, frames: u32) {
        self.0.set(true);
        self.1 = frames;
    }

    pub fn request(&self) {
        self.0.set(true);
    }

    pub fn consume(&mut self) -> bool {
        if self.1 > 0 {
            self.1 -= 1;
            return false;
        }

        let value = self.0.get();
        self.0.set(false);
        value
    }
}
