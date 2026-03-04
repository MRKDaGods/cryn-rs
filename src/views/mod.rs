mod courses_view;
mod placeholder_view;
mod timetable_view;

pub use courses_view::*;
use egui::epaint::MarginF32;
pub use placeholder_view::*;
pub use timetable_view::*;

use crate::CrynContext;
use crate::traits::AsAny;
use crate::windows::Window;
use crate::windows::main_window::NavbarInterface;

pub trait View: AsAny {
    /// View name
    fn name(&self) -> &str;

    /// Should we pad the view?
    fn padding(&self) -> Option<MarginF32> {
        None
    }

    /// Shown callback
    fn on_show(&mut self, app_ctx: &CrynContext);

    /// Hidden callback
    fn on_hide(&mut self, app_ctx: &CrynContext);

    /// Can we hide this view?
    fn can_hide(&self, _app_ctx: &CrynContext) -> bool {
        true
    }

    /// Called every frame when the view is active
    fn on_gui(&mut self, ui: &mut egui::Ui, app_ctx: &CrynContext, window: &mut dyn Window);
}

pub trait MainWindowView: View {
    /// For custom navbar padding
    fn navbar_padding(&self) -> Option<f32> {
        None
    }

    /// Make use of the empty space in the navbar
    fn on_navbar_gui(
        &mut self,
        _ui: &mut egui::Ui,
        _app_ctx: &CrynContext,
        _interface: &NavbarInterface,
    ) {
    }
}
