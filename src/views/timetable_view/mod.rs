mod colors;
mod layout_engine;

use crate::{
    CrynContext,
    models::{CourseSpan, OrderedWeekday},
    views::{CoursesView, View},
    windows::{MainWindow, Window},
};
use egui::{Align, Label, Layout, RichText, Sense};
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
