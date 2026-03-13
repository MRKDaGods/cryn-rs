use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use egui::{
    Align, Color32, CursorIcon, FontId, Label, Layout, Rect, Response, RichText, Sense, Stroke,
    TextWrapMode, Ui, UiBuilder, Vec2,
};

use super::colors::*;
use crate::CrynContext;
use crate::models::{CourseRecord, CourseRecordType, CourseSpan, OrderedWeekday};
use crate::utils::ui::get_trunacted_text;

const TIMETABLE_PADDING: Vec2 = Vec2::new(8.0, 8.0);
const TIMETABLE_SPACING: Vec2 = Vec2::new(12.0, 32.0);

const TIMESLOT_HEADER_WIDTH: f32 = 95.0;
const TIMESLOT_HEADER_HEIGHT: f32 = 43.0;

const DAY_HEADER_HEIGHT: f32 = 42.0;

const COURSE_SLOT_HEIGHT: f32 = 65.5;
const COURSE_SLOT_PADDING: f32 = 8.0;
const COURSE_FONT_SIZE: f32 = 12.5;

struct LayoutEdges {
    h_edges: Vec<Vec<bool>>,
    v_edges: Vec<Vec<bool>>,
}

/// Number of header rows before course rows begin in the edge grid
///
/// Row 0 = day header top, row 1 = timeslot header top, row 2 = timeslot header bottom = courses top
const EDGE_HEADER_ROWS: usize = 2;

impl LayoutEdges {
    fn new(period_count: usize, num_rows: usize) -> Self {
        Self {
            // +3 rows: day top, timeslot top, timeslot bottom, then one per course row bottom
            h_edges: vec![vec![false; period_count]; num_rows + 3],
            // +1 column for rightmost edge, +2 rows for day header and timeslot header
            v_edges: vec![vec![false; period_count + 1]; num_rows + EDGE_HEADER_ROWS],
        }
    }

    fn v_row_count(&self) -> usize {
        self.v_edges.len()
    }
}

pub struct LayoutContext {
    sizes: HashMap<OrderedWeekday, Vec2>,
    is_layout_pass: bool,
    container_rect: Rect,
    day_index: usize,

    /////////////// tb elgamel yo2morny b eh?
    x: f32,
    y: f32,
    max_layout_height: f32,
    //////////////
    layout_edges: HashMap<OrderedWeekday, LayoutEdges>,

    /// Last frame hovered course record
    ///
    /// Rendering pass
    hovered_render: Option<Rc<RefCell<CourseRecord>>>,

    /// Current frame hovered course record
    ///
    /// Interaction pass
    hovered_interaction: Option<Rc<RefCell<CourseRecord>>>,
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self {
            sizes: HashMap::new(),
            is_layout_pass: false,
            container_rect: Rect::ZERO,
            day_index: 0,
            x: 0.0,
            y: 0.0,
            max_layout_height: 0.0,
            layout_edges: HashMap::new(),
            hovered_render: None,
            hovered_interaction: None,
        }
    }
}

impl LayoutContext {
    /// Clear cached layout data, forcing a recompute on the next frame
    pub fn invalidate(&mut self) {
        self.sizes.clear();
        self.layout_edges.clear();
    }

    /// Initialize layout state for the new frame
    pub fn begin(&mut self, ui: &mut Ui) {
        self.container_rect = ui.available_rect_before_wrap();
        self.is_layout_pass = self.sizes.is_empty();
        self.day_index = 0;

        self.x = self.container_rect.left() + TIMETABLE_PADDING.x;
        self.y = self.container_rect.top() + TIMETABLE_PADDING.y;
        self.max_layout_height = 0.0;

        self.hovered_render = self.hovered_interaction.take();
    }

    /// Finalize the frame
    pub fn end(&self, ui: &mut Ui) {
        let total_height =
            self.y + self.max_layout_height + TIMETABLE_PADDING.y - self.container_rect.top();
        ui.allocate_space(Vec2::new(0.0, total_height));
    }

    /// Render a single day column (or compute its layout on the first pass)
    pub fn render_day(
        &mut self,
        app_ctx: &CrynContext,
        ui: &mut Ui,
        day: &OrderedWeekday,
        span: &CourseSpan,
    ) {
        if self.is_layout_pass {
            self.compute_layout(day, span);
            return;
        }

        let layout_size = *self.sizes.get(day).unwrap();
        if self.day_index > 0 && self.needs_wrap(&layout_size) {
            self.wrap_to_next_row();
        }

        let layout_rect = self.calculate_rect(&layout_size);
        self.advance(&layout_rect);

        render_day_header(ui, &layout_rect, day);
        render_timeslots_header(ui, &layout_rect, span);
        self.render_course_slots(app_ctx, ui, &layout_rect, day, span);
        self.render_layout_edges(ui, &layout_rect, day, span);

        self.day_index += 1;
    }

    /// First-pass: compute edge grids and day sizes without rendering
    fn compute_layout(&mut self, day: &OrderedWeekday, span: &CourseSpan) {
        let period_count = span.period_count() as usize;
        let num_rows = span.height_in_periods();

        let LayoutEdges {
            mut h_edges,
            mut v_edges,
        } = LayoutEdges::new(period_count, num_rows);

        //  --------------------
        //  | Day Header       |
        //  --------------------
        //  | Time|Slot|Header |
        //  --------------------

        // Horizontal edges for the 3 header rows
        h_edges
            .iter_mut()
            .take(3)
            .for_each(|edges| edges.fill(true));

        // Day header vertical borders
        v_edges[0][0] = true;
        v_edges[0][period_count] = true;

        // Timeslot vertical borders
        v_edges[1].fill(true);

        // Course slot edges
        for y in 0..num_rows {
            let h_top = y + EDGE_HEADER_ROWS;
            let h_bot = h_top + 1;
            let v_row = y + EDGE_HEADER_ROWS;

            let mut x = 0;
            while x < period_count {
                // https://github.com/MRKDaGods/CUFE-Dry-Run/blob/main/CUFE-Dry-Run/Views/TimeTableView.cs#L259
                // Do we have a course that starts at (x,y)?
                if let Some(record) = span.get(&(x, y)) {
                    let spanned = record.borrow().periods() as usize;

                    for row in [h_top, h_bot] {
                        h_edges[row][x..x + spanned].fill(true);
                    }

                    v_edges[v_row][x] = true;
                    v_edges[v_row][x + spanned] = true;

                    x += spanned;
                } else {
                    x += 1;
                }
            }
        }

        self.layout_edges
            .insert(*day, LayoutEdges { h_edges, v_edges });

        let layout_width = period_count as f32 * TIMESLOT_HEADER_WIDTH;
        let layout_height =
            DAY_HEADER_HEIGHT + TIMESLOT_HEADER_HEIGHT + num_rows as f32 * COURSE_SLOT_HEIGHT;

        self.sizes
            .insert(*day, Vec2::new(layout_width, layout_height));
        self.day_index += 1;
    }

    fn needs_wrap(&self, size: &Vec2) -> bool {
        let available_width = self.container_rect.width() - self.x - TIMETABLE_PADDING.x;
        size.x > available_width
    }

    fn wrap_to_next_row(&mut self) {
        self.x = self.container_rect.left() + TIMETABLE_PADDING.x;
        self.y += self.max_layout_height + TIMETABLE_SPACING.y;
        self.max_layout_height = 0.0;
    }

    fn advance(&mut self, rect: &Rect) {
        self.x += rect.width() + TIMETABLE_SPACING.x;
        self.max_layout_height = self.max_layout_height.max(rect.height());
    }

    fn calculate_rect(&self, size: &Vec2) -> Rect {
        Rect {
            min: (self.x, self.y).into(),
            max: (self.x + size.x, self.y + size.y).into(),
        }
    }

    fn render_course_slots(
        &mut self,
        app_ctx: &CrynContext,
        ui: &mut Ui,
        layout_rect: &Rect,
        day: &OrderedWeekday,
        span: &CourseSpan,
    ) {
        let num_rows = span.height_in_periods();
        let start_y = layout_rect.top() + DAY_HEADER_HEIGHT + TIMESLOT_HEADER_HEIGHT;
        let period_count = span.period_count() as usize;

        for y in 0..num_rows {
            let row_y_start = start_y + y as f32 * COURSE_SLOT_HEIGHT;

            let mut x = 0;
            while x < period_count {
                let record = span.get(&(x, y));
                let spanned = record
                    .as_ref()
                    .map_or(1, |rec| rec.borrow().periods() as usize);

                if let Some(record_rc) = record {
                    let visual_state =
                        resolve_visual_state(&self.hovered_render, app_ctx, record_rc);

                    let x_start = layout_rect.left() + x as f32 * TIMESLOT_HEADER_WIDTH;
                    let course_width = spanned as f32 * TIMESLOT_HEADER_WIDTH;
                    let rect = Rect::from_min_size(
                        egui::pos2(x_start, row_y_start),
                        [course_width, COURSE_SLOT_HEIGHT].into(),
                    );

                    // Interaction
                    let id = ui.id().with(("course", *day, y, x));
                    let response = ui.interact(rect, id, Sense::click());

                    if response.hovered() {
                        self.hovered_interaction = Some(Rc::clone(record_rc));
                    }
                    if response.clicked() {
                        app_ctx
                            .course_manager
                            .borrow_mut()
                            .toggle_selected_course(record_rc);
                    }

                    let record = record_rc.borrow();
                    let (bg_color, text_color) =
                        get_course_colors(&response, &record, &visual_state);

                    // Background
                    ui.painter().rect_filled(rect, 0.0, bg_color);

                    // Cursor + tooltip
                    response
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_ui_at_pointer(|ui| {
                            render_course_tooltip(ui, &record);
                        });

                    // Text content
                    let is_closed_selected = app_ctx.course_manager.borrow().is_selected(record_rc)
                        && record.is_closed();
                    render_course_text(ui, rect, &record, text_color, is_closed_selected);
                }

                x += spanned;
            }
        }
    }

    fn render_layout_edges(
        &self,
        ui: &mut Ui,
        layout_rect: &Rect,
        day: &OrderedWeekday,
        span: &CourseSpan,
    ) {
        let period_count = span.period_count() as usize;
        let edges = self.layout_edges.get(day).unwrap();
        let timeslots_y = layout_rect.top() + DAY_HEADER_HEIGHT;
        let courses_y = timeslots_y + TIMESLOT_HEADER_HEIGHT;

        let y_idx_to_pixel = |idx: usize| -> f32 {
            match idx {
                0 => layout_rect.top(),
                1 => timeslots_y,
                _ => courses_y + (idx - EDGE_HEADER_ROWS) as f32 * COURSE_SLOT_HEIGHT,
            }
        };

        let stroke = Stroke::new(
            1.0,
            ui.style().visuals.widgets.noninteractive.bg_stroke.color,
        );
        let painter = ui.painter();

        // Horizontal runs
        for (y_idx, row) in edges.h_edges.iter().enumerate() {
            let y = y_idx_to_pixel(y_idx);
            let mut run_start: Option<usize> = None;

            // Iterate one past the end to flush any open run
            for (x_idx, &active_flag) in row.iter().chain(&[false]).enumerate() {
                let active = x_idx < period_count && active_flag;
                if active {
                    if run_start.is_none() {
                        run_start = Some(x_idx);
                    }
                } else if let Some(sx) = run_start {
                    let px1 = layout_rect.left() + sx as f32 * TIMESLOT_HEADER_WIDTH;
                    let px2 = layout_rect.left() + x_idx as f32 * TIMESLOT_HEADER_WIDTH;
                    painter.line_segment([egui::pos2(px1, y), egui::pos2(px2, y)], stroke);
                    run_start = None;
                }
            }
        }

        // Vertical runs
        let v_row_count = edges.v_row_count();
        let v_col_count = edges.v_edges.first().map_or(0, |r| r.len());

        for x_idx in 0..v_col_count {
            let x = layout_rect.left() + x_idx as f32 * TIMESLOT_HEADER_WIDTH;
            let mut run_start: Option<usize> = None;

            for y_idx in 0..=v_row_count {
                let active = y_idx < v_row_count && edges.v_edges[y_idx][x_idx];
                if active {
                    if run_start.is_none() {
                        run_start = Some(y_idx);
                    }
                } else if let Some(sy) = run_start {
                    let py1 = y_idx_to_pixel(sy);
                    let py2 = y_idx_to_pixel(y_idx);
                    painter.line_segment([egui::pos2(x, py1), egui::pos2(x, py2)], stroke);
                    run_start = None;
                }
            }
        }
    }
}

fn render_day_header(ui: &mut Ui, layout_rect: &Rect, day: &OrderedWeekday) {
    let rect = Rect::from_min_size(
        layout_rect.left_top(),
        [layout_rect.width(), DAY_HEADER_HEIGHT].into(),
    );
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Label::new(
                RichText::new(day.to_string()).size(14.0).strong(),
            ));
        });
    });
}

fn render_timeslots_header(ui: &mut Ui, layout_rect: &Rect, span: &CourseSpan) {
    let period_count = span.period_count() as usize;
    let y = layout_rect.top() + DAY_HEADER_HEIGHT;

    for c in 0..period_count {
        let hour = span.start_hour() + c as u32;
        let offset = c as f32 * TIMESLOT_HEADER_WIDTH;
        let rect = Rect::from_min_size(
            egui::pos2(layout_rect.left() + offset, y),
            [TIMESLOT_HEADER_WIDTH, TIMESLOT_HEADER_HEIGHT].into(),
        );

        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(Label::new(format!("{}:00\n{}:50", hour, hour)));
            });
        });
    }
}

fn render_course_tooltip(ui: &mut Ui, record: &CourseRecord) {
    ui.strong(&record.course_definition.borrow().code);
    ui.strong(&record.course_definition.borrow().name);
    ui.label(format!(
        "{} G{}\nEnrolled: {}/{}\n{}\n\n{}",
        record.record_type.long_name(),
        record.group,
        record.enrolled,
        record.class_size,
        &record.location,
        record.status.to_uppercase()
    ));
}

fn render_course_text(
    ui: &mut Ui,
    slot_rect: Rect,
    record: &CourseRecord,
    text_color: Color32,
    is_closed_selected: bool,
) {
    let padded_rect = slot_rect.shrink(COURSE_SLOT_PADDING);

    ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
        ui.set_clip_rect(padded_rect.intersect(ui.clip_rect()));

        ui.with_layout(
            Layout::top_down(Align::Center)
                .with_main_align(Align::Center)
                .with_main_justify(true),
            |ui| {
                let course_name = get_trunacted_text(
                    ui,
                    &record.course_definition.borrow().name,
                    COURSE_FONT_SIZE,
                    padded_rect.width(),
                    2,
                );

                let mut job = egui::text::LayoutJob::default();

                // Course name
                job.append(
                    format!("{}\n", course_name).as_str(),
                    0.0,
                    egui::TextFormat {
                        font_id: FontId::proportional(COURSE_FONT_SIZE),
                        color: text_color,
                        line_height: Some(14.0),
                        ..Default::default()
                    },
                );

                // Spacer
                job.append(
                    "\n",
                    0.0,
                    egui::TextFormat {
                        font_id: FontId::proportional(COURSE_FONT_SIZE),
                        color: text_color,
                        line_height: Some(4.0),
                        ..Default::default()
                    },
                );

                // Details line
                let details = format!(
                    "{} G{} ({}/{})",
                    record.record_type.short_name(),
                    record.group,
                    record.enrolled,
                    record.class_size
                );
                job.append(
                    &details,
                    0.0,
                    egui::TextFormat {
                        font_id: FontId::proportional(COURSE_FONT_SIZE - 0.5),
                        color: text_color,
                        ..Default::default()
                    },
                );

                // Strikethrough if closed and selected
                if is_closed_selected {
                    job.sections.iter_mut().for_each(|section| {
                        section.format.strikethrough = Stroke::new(2.0, Color32::BLACK);
                    });
                }

                ui.add(Label::new(job).wrap_mode(TextWrapMode::Wrap));
            },
        );
    });
}

fn resolve_visual_state(
    hovered_render: &Option<Rc<RefCell<CourseRecord>>>,
    app_ctx: &CrynContext,
    record: &Rc<RefCell<CourseRecord>>,
) -> CourseVisualState {
    let borrowed = record.borrow();
    let course_manager = app_ctx.course_manager.borrow();

    // Check group relationship with hovered record
    if let Some(hovered) = hovered_render
        && !Rc::ptr_eq(record, hovered)
    {
        let hovered_borrowed = hovered.borrow();
        let same_course = Rc::ptr_eq(
            &borrowed.course_definition,
            &hovered_borrowed.course_definition,
        );

        if same_course {
            return if borrowed.group == hovered_borrowed.group {
                CourseVisualState::GroupMatch
            } else {
                CourseVisualState::GroupMismatch
            };
        }
    }

    if course_manager.is_selected(record) {
        return if course_manager.is_clashing(record) {
            CourseVisualState::Clashing
        } else {
            CourseVisualState::Selected
        };
    }

    if borrowed.is_closed() {
        return CourseVisualState::Closed;
    }

    CourseVisualState::Default
}

fn get_course_colors(
    response: &Response,
    record: &CourseRecord,
    visual_state: &CourseVisualState,
) -> (Color32, Color32) {
    let colors = match record.record_type {
        CourseRecordType::Lecture => &*COURSE_COLORS_LEC,
        CourseRecordType::Tutorial => &*COURSE_COLORS_TUT,
        CourseRecordType::None => &*COURSE_COLORS_UNK,
    };

    let interaction = match visual_state {
        CourseVisualState::Default => &colors.default,
        CourseVisualState::Selected => &colors.selected,
        CourseVisualState::Clashing => &colors.clashing,
        CourseVisualState::Closed => &colors.closed,
        CourseVisualState::GroupMatch => &colors.group_match,
        CourseVisualState::GroupMismatch => &colors.group_mismatch,
    };

    let state = if response.is_pointer_button_down_on() {
        &interaction.active
    } else if response.hovered() {
        &interaction.hovered
    } else {
        &interaction.normal
    };

    (state.bg, state.text)
}
