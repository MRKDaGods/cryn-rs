mod logger;
pub mod ui;

use std::cell::Cell;

pub use logger::*;

/// Simple boolean signal
#[derive(Default)]
pub struct Signal(Cell<bool>);

impl Signal {
    pub fn request(&self) {
        self.0.set(true);
    }

    pub fn consume(&self) -> bool {
        let value = self.0.get();
        self.0.set(false);
        value
    }
}
