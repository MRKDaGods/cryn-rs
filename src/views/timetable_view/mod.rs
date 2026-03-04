mod colors;
mod layout_engine;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use egui::{Align, Color32, FontId, Label, Layout, RichText, Sense, Ui, Vec2};

use crate::models::{
    CourseDefinition, CourseEvent, CourseEventListener, CourseRecord, CourseRecordType, CourseSpan,
    CourseSummary, OrderedWeekday,
};
use crate::utils::ui::get_trunacted_text;
use crate::views::timetable_view::colors::{COURSE_COLORS_LEC, COURSE_COLORS_TUT};
use crate::views::{CoursesView, MainWindowView, View};
use crate::windows::{MainWindow, NavbarInterface, Window};
use crate::{CrynContext, icons};

pub struct TimeTableView {
    span_map: BTreeMap<OrderedWeekday, CourseSpan>,
    layout_context: layout_engine::LayoutContext,
    listener_state: Rc<RefCell<TimeTableListenerState>>,
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

impl TimeTableView {
    pub fn new() -> Self {
        Self::default()
    }
}

impl View for TimeTableView {
    fn name(&self) -> &str {
        "Time Table"
    }

    fn on_show(&mut self, app_ctx: &CrynContext) {
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
        // If no courses, show placeholder
        if self.span_map.is_empty() {
            render_landing(ui, app_ctx, window);
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
    fn on_navbar_gui(&mut self, ui: &mut Ui, app_ctx: &CrynContext, _interface: &NavbarInterface) {
        let listener_state = self.listener_state.borrow();
        if listener_state.course_summaries.is_empty() {
            return;
        }

        // Build inline summary text
        let summary_text = listener_state
            .course_summaries
            .iter()
            .map(|s| {
                format!(
                    "{} ({}/{})",
                    &s.name,
                    s.selected_lec.map_or("NA".to_owned(), |g| g.to_string()),
                    s.selected_tut.map_or("NA".to_owned(), |g| g.to_string()),
                )
            })
            .collect::<Vec<_>>()
            .join(format!(" {} ", icons::SEPARATOR).as_str());

        // Use previous frame hover state
        let hover_id = ui.id().with("summary_hover");
        let is_hovered = ui.data(|d| d.get_temp::<bool>(hover_id).unwrap_or(false));

        let response = ui
            .allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let width = ui.available_width();
                    let summary_font_size = 13.0;
                    let text = get_trunacted_text(ui, &summary_text, summary_font_size, width, 1);

                    let mut rich_text = RichText::new(text).size(summary_font_size);
                    if !is_hovered {
                        rich_text = rich_text.weak();
                    }

                    ui.add(Label::new(rich_text).extend().sense(Sense::hover()))
                },
            )
            .inner;

        // Persist hover state for next frame
        ui.data_mut(|d| d.insert_temp(hover_id, response.hovered()));

        // Hover popup
        response.on_hover_ui(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_width(600.0);
            ui.spacing_mut().item_spacing.y = 2.0;

            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icons::LIBRARY)
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.strong(format!(
                    "{} courses selected",
                    listener_state.course_summaries.len()
                ));
            });

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            // Course list
            let mut to_deselect: Vec<*const RefCell<CourseDefinition>> = Vec::new();

            for s in &listener_state.course_summaries {
                ui.horizontal(|ui| {
                    // Code
                    ui.label(
                        RichText::new(&s.code)
                            .strong()
                            .color(ui.visuals().strong_text_color()),
                    );

                    ui.label(
                        RichText::new(icons::SEPARATOR)
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );

                    // Name
                    ui.label(RichText::new(&s.name));

                    // Lec/Tut groups
                    let lec = s.selected_lec.map_or("NA".to_owned(), |g| g.to_string());
                    let tut = s.selected_tut.map_or("NA".to_owned(), |g| g.to_string());

                    ui.label(
                        RichText::new(format!("(Lec {} / Tut {})", lec, tut))
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );

                    // Remove button
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(icons::CLOSE)
                                        .size(10.0)
                                        .color(ui.visuals().weak_text_color()),
                                )
                                .frame(false),
                            )
                            .on_hover_text("Remove course")
                            .clicked()
                        {
                            to_deselect.push(s.definition);
                        }
                    });
                });
            }

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            // Action buttons row
            ui.columns(3, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(summary_text);
                    }
                });
                cols[1].vertical_centered_justified(|ui| {
                    if ui.button("Clear All").clicked() {
                        to_deselect = listener_state
                            .course_summaries
                            .iter()
                            .map(|s| s.definition)
                            .collect();
                    }
                });
                cols[2].vertical_centered_justified(|ui| {
                    if ui.button("Import").clicked() {
                        app_ctx.show_import_window();
                        ui.close_kind(egui::UiKind::Tooltip);
                    }
                });
            });

            // Apply deselections
            // Release old borrow since notify_listeners will borrow listener_state again
            drop(listener_state);

            if !to_deselect.is_empty() {
                let mut course_manager = app_ctx.course_manager.borrow_mut();
                for (idx, def_ptr) in to_deselect.iter().enumerate() {
                    let is_batch = idx < to_deselect.len() - 1;
                    course_manager.deselect_course_records(*def_ptr, is_batch);
                }
            }
        });
    }
}

fn render_landing(ui: &mut Ui, app_ctx: &CrynContext, window: &mut dyn Window) {
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

fn render_legend(ui: &mut Ui) {
    // Top padding
    ui.add_space(4.0);

    let height = 28.0;
    let size = Vec2::new(ui.available_width(), height);

    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        let lec_colors = &*COURSE_COLORS_LEC;

        let items = [
            ("Lecture", lec_colors.default.normal.bg),
            ("Tutorial", COURSE_COLORS_TUT.default.normal.bg),
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

#[derive(Default)]
struct TimeTableListenerState {
    course_summaries: Vec<CourseSummary>,
}

impl CourseEventListener for TimeTableListenerState {
    fn on_course_event(&mut self, event: &CourseEvent) {
        match event {
            CourseEvent::SelectionChanged(selected_records) => {
                // Rebuild summaries
                self.course_summaries.clear();

                if selected_records.is_empty() {
                    return;
                }

                // Group selected records by course def
                let mut course_record_map = HashMap::<
                    *const RefCell<CourseDefinition>,
                    Vec<Rc<RefCell<CourseRecord>>>,
                >::new();

                selected_records.iter().for_each(|r| {
                    let record = r.borrow();
                    course_record_map
                        .entry(Rc::as_ptr(&record.course_definition))
                        .or_default()
                        .push(Rc::clone(r));
                });

                // Map into CourseSummary
                let mut summaries = course_record_map
                    .iter()
                    .map(|(def_ptr, record)| {
                        let def = unsafe { &**def_ptr }.borrow();

                        let mut summary = CourseSummary {
                            code: def.code.clone(),
                            name: def.name.clone(),
                            definition: *def_ptr,
                            ..Default::default()
                        };

                        record.iter().for_each(|r| {
                            let record = r.borrow();
                            match record.record_type {
                                CourseRecordType::Lecture => {
                                    summary.selected_lec = Some(record.group)
                                }
                                CourseRecordType::Tutorial => {
                                    summary.selected_tut = Some(record.group)
                                }
                                CourseRecordType::None => (),
                            }
                        });

                        summary
                    })
                    .collect::<Vec<_>>();

                // Sort by code
                summaries.sort_by(|a, b| a.code.cmp(&b.code));

                // Update summaries
                self.course_summaries = summaries;
            }
        }
    }
}
