pub mod main_window;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub use main_window::MainWindow;
pub use main_window::NavbarInterface;

use crate::views::View;

pub trait Window {
    fn views(&self) -> &HashMap<TypeId, Box<dyn View>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
