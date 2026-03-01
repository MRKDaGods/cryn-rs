use super::View;
use crate::CrynContext;
use crate::windows::Window;

pub struct PlaceholderView;

impl View for PlaceholderView {
    fn name(&self) -> &str {
        "Placeholder"
    }

    fn on_show(&mut self, _app_ctx: &CrynContext) {
        println!("Placeholder::hello")
    }

    fn on_hide(&mut self, _app_ctx: &CrynContext) {
        println!("Placeholder::bye")
    }

    fn on_gui(&mut self, ui: &mut egui::Ui, _app_ctx: &CrynContext, _window: &mut dyn Window) {
        ui.heading("Placeholder View");
    }
}
