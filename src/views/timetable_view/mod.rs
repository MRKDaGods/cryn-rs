mod colors;
mod layout_engine;

use crate::{
    CrynContext,
    models::{CourseSpan, OrderedWeekday},
    views::{
        CoursesView, View,
        timetable_view::colors::{COURSE_COLORS_LEC, COURSE_COLORS_TUT},
    },
    windows::{MainWindow, Window},
};
use egui::{Align, Color32, FontId, Label, Layout, RichText, Sense, Ui, Vec2};
use std::collections::BTreeMap;

pub struct TimeTableView {
    span_map: BTreeMap<OrderedWeekday, CourseSpan>,
    layout_context: layout_engine::LayoutContext,
}

impl TimeTableView {
    pub fn new() -> Self {
        Self {
            span_map: BTreeMap::new(),
            layout_context: layout_engine::LayoutContext::new(),
        }
    }
}

impl View for TimeTableView {
    fn name(&self) -> &str {
        "Time Table"
    }

    fn on_show(&mut self, app_ctx: &CrynContext) {
        // Build span map
        let available_records = &app_ctx
            .course_manager
            .borrow()
            .get_available_course_records();

        self.span_map.clear();
        available_records.iter().for_each(|record| {
            self.span_map
                .entry(record.borrow().day.into())
                .or_insert(CourseSpan::new())
                .insert(record);
        });

        // Rebuild spans
        self.span_map.values_mut().for_each(|span| span.rebuild());

        self.span_map.iter().for_each(|(day, span)| {
            println!("{}: {} periods", day.to_string(), span.period_count());
        });

        // Recompute everything
        layout_engine::invalidate_layout(&mut self.layout_context);
    }

    fn on_hide(&mut self, _app_ctx: &CrynContext) {}

    fn on_gui(&mut self, ui: &mut egui::Ui, app_ctx: &CrynContext, window: &mut dyn Window) {
        // hmmm
        // elnas 3yza eh
        // elnas bt3ml eh
        // 3en safra wltanya 7amra
        // ololy a3ml eh
        // 3en safra wltanya khadra ;)
        // ololy a3ml ehhhhhh

        // If no courses, show placeholder
        if self.span_map.is_empty() {
            ui.centered_and_justified(|ui| {
                if ui
                    .add(
                        Label::new(RichText::new("Select courses to start").heading())
                            .sense(Sense::click()),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    // Go to courses view
                    if let Some(main_window) = window.as_any_mut().downcast_mut::<MainWindow>() {
                        main_window.switch_to_view::<CoursesView>(app_ctx);
                    }
                }
            });
            return;
        }

        // Render legend
        render_legend(ui);

        // Render timetable
        ui.scope(|ui| {
            let style = ui.style_mut();
            style.spacing.item_spacing = egui::vec2(0.0, 0.0);

            // Clip to timetable area
            let rect = ui.available_rect_before_wrap();
            ui.set_clip_rect(rect);

            // Render days
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        layout_engine::begin(&mut self.layout_context, ui);

                        for (day, span) in &self.span_map {
                            layout_engine::render_day(
                                &mut self.layout_context,
                                app_ctx,
                                ui,
                                day,
                                span,
                            );
                        }

                        layout_engine::end(&mut self.layout_context, ui);
                    });
                });
        });
    }
}

fn render_legend(ui: &mut Ui) {
    // Top padding
    ui.add_space(4.0);

    let height = 28.0;
    let size = Vec2::new(ui.available_width(), height);

    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        let lec_colors = &*COURSE_COLORS_LEC;

        let items = [
            ("Lecture", lec_colors.default.normal.bg),
            ("Tutorial", (&*COURSE_COLORS_TUT).default.normal.bg),
            ("Selected", lec_colors.selected.normal.bg),
            ("Clashing", lec_colors.clashing.normal.bg),
            ("Closed", lec_colors.closed.normal.bg),
            ("Same Group", lec_colors.group_match.normal.bg),
            ("Diff Group", lec_colors.group_mismatch.normal.bg),
        ];

        // Measure total width first
        let swatch_size = 12.0;
        let inner_spacing = 6.0; // space between swatch and label
        let item_spacing = 16.0;
        let total_width: f32 = items
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                let label_width = ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        label.to_string(),
                        FontId::proportional(11.0),
                        Color32::WHITE,
                    )
                    .size()
                    .x
                });
                let spacing = if i > 0 { item_spacing } else { 0.0 };
                spacing + swatch_size + inner_spacing + label_width
            })
            .sum();

        // Add left padding
        let available = ui.available_width();
        if total_width < available {
            ui.add_space((available - total_width) / 2.0);
        }

        ui.spacing_mut().item_spacing.x = inner_spacing;

        for (i, (label, color)) in items.iter().enumerate() {
            if i > 0 {
                ui.add_space(item_spacing - inner_spacing);
            }

            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(swatch_size, swatch_size), Sense::hover());
            ui.painter().rect_filled(rect, 2.0, *color);
            ui.label(RichText::new(*label).size(11.0));
        }
    });

    ui.separator();
}
