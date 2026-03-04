use super::TITLEBAR_HEIGHT;
use crate::views::MainWindowView;

const TITLEBAR_PADDING_H: f32 = 12.0;

pub fn render_title_bar(ctx: &egui::Context, current_view: Option<&dyn MainWindowView>) {
    egui::TopBottomPanel::top("titlebar")
        .frame(
            egui::Frame::new()
                .inner_margin(egui::Margin::ZERO)
                .fill(ctx.style().visuals.window_fill),
        )
        .exact_height(TITLEBAR_HEIGHT)
        .show_separator_line(true)
        .show(ctx, |ui| {
            // Title bar events on desktop
            #[cfg(not(target_arch = "wasm32"))]
            super::desktop::handle_title_bar_events(ctx, ui);

            // Main titlebar pass
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Center)
                    .with_cross_align(egui::Align::Center),
                |ui| {
                    // Title
                    ui.add_space(TITLEBAR_PADDING_H);
                    ui.label("Cryn - Ammar Magnus");

                    // View name
                    if let Some(current_view) = current_view {
                        let title_width = ui
                            .painter()
                            .layout_no_wrap(
                                current_view.name().to_owned(),
                                egui::TextStyle::Body.resolve(ui.style()),
                                ui.visuals().text_color(),
                            )
                            .size()
                            .x;

                        // Centered on desktop
                        #[cfg(not(target_arch = "wasm32"))]
                        ui.add_space(
                            (-ui.cursor().left() + ui.available_width()) * 0.5 - title_width * 0.5,
                        );

                        // Far right on web
                        #[cfg(target_arch = "wasm32")]
                        ui.add_space(ui.available_width() - title_width - TITLEBAR_PADDING_H);

                        ui.label(current_view.name());
                    }

                    // Window controls on desktop
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        super::desktop::render_window_controls(ctx, ui);
                    });
                },
            );
        });
}
