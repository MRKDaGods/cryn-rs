use egui::{Color32, FontId, RichText, Ui};

use crate::views::CoursesView;
use crate::windows::{MainWindow, Window};
use crate::{CrynContext, icons};

pub(super) fn render_landing(ui: &mut Ui, app_ctx: &CrynContext, window: &mut dyn Window) {
    let button_width = 240.0;
    let button_height = 38.0;
    let button_corner_radius = 6.0;

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.28);

        // Title
        ui.label(
            RichText::new(format!("{} Cryn", icons::CALENDAR))
                .size(42.0)
                .strong(),
        );

        ui.add_space(6.0);

        // Acronym
        let accent = ui.visuals().strong_text_color();
        let muted = ui.visuals().weak_text_color();
        let font_size = 16.0;

        ui.horizontal(|ui| {
            // Measure and center
            let display_text = "CUFE  Dry  Run";
            let total_width = ui.fonts_mut(|f| {
                f.layout_no_wrap(
                    display_text.to_string(),
                    FontId::proportional(font_size),
                    Color32::WHITE,
                )
                .size()
                .x
            });
            let available = ui.available_width();
            if total_width < available {
                ui.add_space((available - total_width) / 2.0);
            }

            ui.spacing_mut().item_spacing.x = 0.0;

            // "C" accented, "UFE" muted
            ui.label(RichText::new("C").size(font_size).color(accent).strong());
            ui.label(RichText::new("UFE").size(font_size).color(muted));

            ui.add_space(8.0);

            // "D" muted, "r" accented, "y" accented
            ui.label(RichText::new("D").size(font_size).color(muted));
            ui.label(RichText::new("r").size(font_size).color(accent).strong());
            ui.label(RichText::new("y").size(font_size).color(accent).strong());

            ui.add_space(8.0);

            // "Ru" muted, "n" accented
            ui.label(RichText::new("Ru").size(font_size).color(muted));
            ui.label(RichText::new("n").size(font_size).color(accent).strong());
        });

        ui.add_space(32.0);

        // Primary: Select Courses
        let select_btn = ui.add_sized(
            [button_width, button_height],
            egui::Button::new(
                RichText::new(format!("{}  Select Courses", icons::LIBRARY)).size(15.0),
            )
            .corner_radius(button_corner_radius),
        );

        if select_btn.clicked()
            && let Some(window) = window.as_any_mut().downcast_mut::<MainWindow>()
        {
            window.switch_to_view::<CoursesView>(app_ctx);
        }

        ui.add_space(10.0);

        // Divider
        ui.label(
            RichText::new("or")
                .size(13.0)
                .color(Color32::from_gray(120)),
        );

        ui.add_space(10.0);

        // Secondary: Import
        let import_btn = ui.add_sized(
            [button_width, button_height],
            egui::Button::new(
                RichText::new(format!("{}  Import Timetable", icons::IMPORT))
                    .size(15.0)
                    .color(Color32::from_gray(200)),
            )
            .corner_radius(button_corner_radius)
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(100)))
            .fill(Color32::from_gray(30)),
        );

        if import_btn.clicked() {
            app_ctx.show_import_window();
        }
    });
}
