mod colors;
mod landing;
mod layout_engine;
mod listener;
mod navbar;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{Align, Color32, FontId, Layout, RichText, Sense, Ui, Vec2};
use listener::TimeTableListenerState;

use crate::CrynContext;
use crate::models::{CourseEventListener, CourseSpan, OrderedWeekday};
use crate::views::timetable_view::colors::{
    COURSE_COLORS_LEC, COURSE_COLORS_TUT, COURSE_COLORS_UNK,
};
use crate::views::{MainWindowView, View};
use crate::windows::{NavbarInterface, Window};

pub struct TimeTableView {
    span_map: BTreeMap<OrderedWeekday, CourseSpan>,
    layout_context: layout_engine::LayoutContext,
    listener_state: Rc<RefCell<TimeTableListenerState>>,
}

impl TimeTableView {
    /// Rebuilds the span map and recomputes the layout context
    fn rebuild(&mut self, app_ctx: &CrynContext) {
        // Build span map
        let available_records = app_ctx
            .course_manager
            .borrow()
            .get_available_course_records();

        self.span_map.clear();
        available_records.iter().for_each(|record| {
            self.span_map
                .entry(record.borrow().day.into())
                .or_default()
                .insert(record);
        });

        // Rebuild spans
        self.span_map.values_mut().for_each(|span| span.rebuild());

        // Recompute everything
        self.layout_context.invalidate();
    }
}

impl Default for TimeTableView {
    fn default() -> Self {
        Self {
            span_map: BTreeMap::new(),
            layout_context: layout_engine::LayoutContext::default(),
            listener_state: Rc::new(RefCell::new(TimeTableListenerState::default())),
        }
    }
}

impl View for TimeTableView {
    fn name(&self) -> &str {
        "Time Table"
    }

    fn on_show(&mut self, app_ctx: &CrynContext) {
        // Rebuild span map and layout context
        self.rebuild(app_ctx);

        // Register listener
        app_ctx.course_manager.borrow_mut().register_listener(
            Rc::clone(&self.listener_state) as Rc<RefCell<dyn CourseEventListener>>
        );
    }

    fn on_hide(&mut self, app_ctx: &CrynContext) {
        // Unregister listener
        app_ctx.course_manager.borrow_mut().unregister_listener(
            Rc::clone(&self.listener_state) as Rc<RefCell<dyn CourseEventListener>>
        );
    }

    fn on_gui(&mut self, ui: &mut egui::Ui, app_ctx: &CrynContext, window: &mut dyn Window) {
        // Check for rebuild requests
        let rebuild_required = self.listener_state.borrow().rebuild_required.consume();
        if rebuild_required {
            self.rebuild(app_ctx);
        }

        // If no courses, show placeholder
        if self.span_map.is_empty() {
            landing::render_landing(ui, app_ctx, window);
            return;
        }

        // Main view: legend + timetable
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
                        self.layout_context.begin(ui);

                        for (day, span) in &self.span_map {
                            self.layout_context.render_day(app_ctx, ui, day, span);
                        }

                        self.layout_context.end(ui);
                    });
                });
        });
    }
}

impl MainWindowView for TimeTableView {
    fn navbar_padding(&self) -> Option<f32> {
        Some(0.0)
    }

    fn on_navbar_gui(&mut self, ui: &mut Ui, app_ctx: &CrynContext, _interface: &NavbarInterface) {
        navbar::render_navbar(&self.listener_state, ui, app_ctx, _interface);
    }
}

fn render_legend(ui: &mut Ui) {
    // Top padding
    ui.add_space(4.0);

    let height = 28.0;
    let size = Vec2::new(ui.available_width(), height);

    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        let items = [
            ("Lecture", COURSE_COLORS_LEC.default.normal.bg),
            ("Tutorial", COURSE_COLORS_TUT.default.normal.bg),
            ("Selected", COURSE_COLORS_UNK.selected.normal.bg),
            ("Clashing", COURSE_COLORS_UNK.clashing.normal.bg),
            ("Closed", COURSE_COLORS_UNK.closed.normal.bg),
            ("Same Group", COURSE_COLORS_UNK.group_match.normal.bg),
            ("Diff Group", COURSE_COLORS_UNK.group_mismatch.normal.bg),
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
