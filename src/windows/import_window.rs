use egui::{Id, Modal};

use crate::{CrynContext, windows::Window};

#[derive(Default)]
pub struct ImportWindow {}

impl Window for ImportWindow {
    fn render(&mut self, ctx: &egui::Context, app_ctx: &CrynContext) {
        Modal::new(Id::new("import_window")).show(ctx, |ui| {
            ui.heading("Import Window");
            if ui.button("Close").clicked() {
                app_ctx.dispose_import_window();
            }
        });
    }
}
