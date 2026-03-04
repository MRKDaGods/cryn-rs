mod import_window;
pub mod main_window;

pub use import_window::ImportWindow;
pub use main_window::MainWindow;
pub use main_window::NavbarInterface;

use crate::CrynContext;
use crate::traits::AsAny;

pub trait Window: AsAny {
    fn initialize(&mut self, _app_ctx: &CrynContext) {}
    fn render(&mut self, ctx: &egui::Context, app_ctx: &CrynContext);
    fn on_dispose(&mut self, _app_ctx: &CrynContext) {}
}
