use super::colors::*;
use crate::{
    CrynContext,
    models::{CourseRecord, CourseRecordType, CourseSpan, OrderedWeekday},
};
use egui::{
    Align, Color32, CursorIcon, FontId, Label, Layout, Rect, Response, RichText, Sense, Stroke,
    TextWrapMode, Ui, UiBuilder, Vec2,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const TIMETABLE_PADDING: Vec2 = Vec2::new(8.0, 8.0);
const TIMETABLE_SPACING: Vec2 = Vec2::new(12.0, 32.0);

const TIMESLOT_HEADER_WIDTH: f32 = 95.0;
const TIMESLOT_HEADER_HEIGHT: f32 = 43.0;

const DAY_HEADER_HEIGHT: f32 = 42.0;

const COURSE_SLOT_HEIGHT: f32 = 65.5;
const COURSE_SLOT_PADDING: f32 = 4.0;
const COURSE_FONT_SIZE: f32 = 12.5;

struct LayoutEdges {
    h_edges: Vec<Vec<bool>>,
    v_edges: Vec<Vec<bool>>,
}

impl LayoutEdges {
    fn new(period_count: usize, num_rows: usize) -> Self {
        Self {
            h_edges: vec![vec![false; period_count]; num_rows + 3], // +3: Day top, timeslot top, timeslot bottom
            v_edges: vec![vec![false; period_count + 1]; num_rows + 2], // +1 for rightmost edge, +2 for day header and timeslot header
        }
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
    /// Render pass
    hovered_render: Option<Rc<RefCell<CourseRecord>>>,

    /// Current frame hovered course record
    /// Interaction pass
    hovered_interaction: Option<Rc<RefCell<CourseRecord>>>,
}

impl LayoutContext {
    pub fn new() -> Self {
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

pub fn invalidate_layout(ctx: &mut LayoutContext) {
    ctx.sizes.clear();
    ctx.layout_edges.clear();
}

pub fn begin(ctx: &mut LayoutContext, ui: &mut Ui) {
    ctx.container_rect = ui.available_rect_before_wrap();
    ctx.is_layout_pass = ctx.sizes.is_empty();
    ctx.day_index = 0;

    ctx.x = ctx.container_rect.left() + TIMETABLE_PADDING.x;
    ctx.y = ctx.container_rect.top() + TIMETABLE_PADDING.y;
    ctx.max_layout_height = 0.0;

    ctx.hovered_render = ctx.hovered_interaction.take();
}

pub fn end(ctx: &mut LayoutContext, ui: &mut Ui) {
    // We've been doing layout ourselves
    // Let egui know how much space we took for scrolling
    let total_height =
        ctx.y + ctx.max_layout_height + TIMETABLE_PADDING.y - ctx.container_rect.top();

    ui.allocate_space(Vec2::new(0.0, total_height));
}

fn do_layout(ctx: &mut LayoutContext, _ui: &mut Ui, day: &OrderedWeekday, span: &CourseSpan) {
    // Not really layout, just compute edges and calc size for render pass
    // Keeping ui for backwards compatibility

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

    // Day header and timeslot header horizontal borders
    for x in 0..period_count {
        h_edges[0][x] = true; // Day header top border
        h_edges[1][x] = true; // Timeslot header top border
        h_edges[2][x] = true; // Timeslot header bottom border
    }

    v_edges[0][0] = true; // Day header left border
    v_edges[0][period_count] = true; // Day header right border

    // Timeslot vertical borders and separators
    for x in 0..=period_count {
        v_edges[1][x] = true;
    }

    // Course slot edges
    for y in 0..num_rows {
        let mut x = 0;
        while x < period_count {
            // https://github.com/MRKDaGods/CUFE-Dry-Run/blob/main/CUFE-Dry-Run/Views/TimeTableView.cs#L259
            // Do we have a course that starts at (x,y)?
            if let Some(record) = span.get(&(x, y)) {
                let spanned = record.borrow().periods() as usize;

                // Top and bottom border
                for x in x..x + spanned {
                    h_edges[y + 2][x] = true;
                    h_edges[y + 3][x] = true;
                }

                // Left and right border
                v_edges[y + 2][x] = true; // Left border
                v_edges[y + 2][x + spanned] = true; // Right border

                // Skip spanned columns
                x += spanned;
            } else {
                x += 1;
            }
        }
    }

    ctx.layout_edges
        .insert(day.clone(), LayoutEdges { h_edges, v_edges });

    // Compute layout size
    let layout_width = period_count as f32 * TIMESLOT_HEADER_WIDTH;
    let layout_height =
        DAY_HEADER_HEIGHT + TIMESLOT_HEADER_HEIGHT + num_rows as f32 * COURSE_SLOT_HEIGHT;
    let size = Vec2::new(layout_width, layout_height);

    ctx.sizes.insert(day.clone(), size);
    ctx.day_index += 1;
}

pub fn render_day(
    ctx: &mut LayoutContext,
    app_ctx: &CrynContext,
    ui: &mut Ui,
    day: &OrderedWeekday,
    span: &CourseSpan,
) {
    if ctx.is_layout_pass {
        do_layout(ctx, ui, day, span);
        return;
    }

    // Do we need to wrap?
    let layout_size = ctx.sizes.get(day).unwrap().clone();
    if ctx.day_index > 0 && needs_wrap(ctx, &layout_size) {
        wrap(ctx);
    }

    // Calc rect and advance cursor
    let layout_rect = calculate_rect(ctx, &layout_size);
    advance(ctx, &layout_rect);

    // Render Day header
    render_day_header(ui, &layout_rect, day);

    // Time slots
    render_timeslots_header(ui, &layout_rect, span);

    // Render course slots
    render_course_slots(ctx, app_ctx, ui, &layout_rect, day, span);

    // Draw borders
    render_layout_edges(ctx, ui, &layout_rect, day, span);

    ctx.day_index += 1;
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

fn render_course_slots(
    ctx: &mut LayoutContext,
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
        let y_offset = y as f32 * COURSE_SLOT_HEIGHT;
        let row_y_start = start_y + y_offset;

        let mut x = 0;
        while x < period_count {
            let record = span.get(&(x, y));
            let spanned = record
                .as_ref()
                .map_or(1, |rec| rec.borrow().periods() as usize);

            // Shawerly bas w oly 3aleh

            if let Some(record_rc) = record {
                let visual_state = resolve_visual_state(ctx, app_ctx, record_rc);

                let x_offset = x as f32 * TIMESLOT_HEADER_WIDTH;
                let x_start = layout_rect.left() + x_offset;

                let course_width = spanned as f32 * TIMESLOT_HEADER_WIDTH;
                let rect = Rect::from_min_size(
                    egui::pos2(x_start, row_y_start),
                    [course_width, COURSE_SLOT_HEIGHT].into(),
                );

                // Register interaction
                let id = ui.id().with(("course", day.clone(), y, x));
                let mut response = ui.interact(rect, id, Sense::click());

                if response.hovered() {
                    ctx.hovered_interaction = Some(Rc::clone(record_rc));
                }
                if response.clicked() {
                    // Toggle selected
                    app_ctx
                        .course_manager
                        .borrow_mut()
                        .toggle_selected_course(record_rc);

                    println!("sup {:?}", record);
                }

                // Shadow em babes
                let record = record_rc.borrow();

                // Get bg and text color based on interaction state
                let (bg_color, text_color) =
                    get_course_colors(app_ctx, &response, record_rc, &record, &visual_state);

                // Render background
                ui.painter().rect_filled(rect, 0.0, bg_color);

                // Change cursor on hover
                response = response.on_hover_cursor(CursorIcon::PointingHand);

                // Show tooltip on hover
                response = response.on_hover_ui_at_pointer(|ui| {
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

                    // Debug visual state
                    ui.separator();
                    ui.label(format!("Visual State: {:?}", visual_state));
                });

                // Render text
                let padded_rect = rect.shrink(COURSE_SLOT_PADDING);
                ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                    // Clip text w/respect to scroll area clip
                    ui.set_clip_rect(padded_rect.intersect(ui.clip_rect()));

                    ui.with_layout(
                        Layout::top_down(Align::Center)
                            .with_main_align(Align::Center)
                            .with_main_justify(true),
                        |ui| {
                            let mut course_name = record.course_definition.borrow().name.clone();

                            // Measure how many lines the course name takes
                            // if more than 2, truncate
                            let rows = &ui
                                .painter()
                                .layout(
                                    course_name.clone(),
                                    FontId::proportional(COURSE_FONT_SIZE),
                                    text_color,
                                    padded_rect.width(),
                                )
                                .rows;

                            // Truncate and add ellipsis if needed
                            if rows.len() > 2 {
                                course_name = String::new();

                                for row in rows.iter().take(2) {
                                    course_name.push_str(&row.text());
                                }

                                // Remove last 3 chars and add ellipsis
                                course_name = course_name
                                    .chars()
                                    .take(course_name.len().saturating_sub(3))
                                    .collect();
                                course_name.push_str("...");
                            }

                            let mut job = egui::text::LayoutJob::default();
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

                            // Add extra vertical spacing
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

                            // Course type and group on second line
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

                            // Strikethrough if both closed and selected
                            let is_closed_selected =
                                app_ctx.course_manager.borrow().is_selected(record_rc)
                                    && record.is_closed();
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

            x += spanned;
        }
    }
}

fn render_layout_edges(
    ctx: &LayoutContext,
    ui: &mut Ui,
    layout_rect: &Rect,
    day: &OrderedWeekday,
    span: &CourseSpan,
) {
    let period_count = span.period_count() as usize;
    let num_rows = span.height_in_periods();

    let LayoutEdges { h_edges, v_edges } = ctx.layout_edges.get(day).unwrap();
    let timeslots_y = layout_rect.top() + DAY_HEADER_HEIGHT;
    let courses_y = timeslots_y + TIMESLOT_HEADER_HEIGHT;

    let stroke = Stroke::new(
        1.0,
        ui.style().visuals.widgets.noninteractive.bg_stroke.color,
    );
    let painter = ui.painter();

    // Draw horizontal edges
    for y_idx in 0..=num_rows + 2 {
        // Where are we drawing?
        let y = match y_idx {
            0 => layout_rect.top(),
            1 => timeslots_y,
            2 => courses_y,
            _ => courses_y + (y_idx - 2) as f32 * COURSE_SLOT_HEIGHT,
        };

        let mut start_x: Option<usize> = None;
        for x_idx in 0..=period_count {
            let is_edge = x_idx < period_count && h_edges[y_idx][x_idx];
            if is_edge {
                if start_x.is_none() {
                    start_x = Some(x_idx);
                }
            } else if let Some(sx) = start_x {
                let px1 = layout_rect.left() + sx as f32 * TIMESLOT_HEADER_WIDTH;
                let px2 = layout_rect.left() + x_idx as f32 * TIMESLOT_HEADER_WIDTH;
                painter.line_segment([egui::pos2(px1, y), egui::pos2(px2, y)], stroke);
                start_x = None;
            }
        }
    }

    // Draw vertical edges
    for x_idx in 0..=period_count {
        let x = layout_rect.left() + x_idx as f32 * TIMESLOT_HEADER_WIDTH;

        let mut start_y: Option<usize> = None;
        for y_idx in 0..=num_rows + 2 {
            let is_edge = y_idx < num_rows + 2 && v_edges[y_idx][x_idx];
            if is_edge {
                if start_y.is_none() {
                    start_y = Some(y_idx);
                }
            } else if let Some(sy) = start_y {
                let py1 = match sy {
                    0 => layout_rect.top(),
                    1 => timeslots_y,
                    2 => courses_y,
                    _ => courses_y + (sy - 2) as f32 * COURSE_SLOT_HEIGHT,
                };
                let py2 = match y_idx {
                    0 => layout_rect.top(),
                    1 => timeslots_y,
                    2 => courses_y,
                    _ => courses_y + (y_idx - 2) as f32 * COURSE_SLOT_HEIGHT,
                };
                painter.line_segment([egui::pos2(x, py1), egui::pos2(x, py2)], stroke);
                start_y = None;
            }
        }
    }
}

fn needs_wrap(ctx: &LayoutContext, size: &Vec2) -> bool {
    let available_width = ctx.container_rect.width() - ctx.x - TIMETABLE_PADDING.x;
    size.x > available_width
}

fn wrap(ctx: &mut LayoutContext) {
    ctx.x = ctx.container_rect.left() + TIMETABLE_PADDING.x;
    ctx.y += ctx.max_layout_height + TIMETABLE_SPACING.y;
    ctx.max_layout_height = 0.0;
}

fn advance(ctx: &mut LayoutContext, rect: &Rect) {
    ctx.x += rect.width() + TIMETABLE_SPACING.x;
    ctx.max_layout_height = ctx.max_layout_height.max(rect.height());
}

fn calculate_rect(ctx: &mut LayoutContext, size: &Vec2) -> Rect {
    Rect {
        min: (ctx.x, ctx.y).into(),
        max: (ctx.x + size.x, ctx.y + size.y).into(),
    }
}

fn resolve_visual_state(
    ctx: &LayoutContext,
    app_ctx: &CrynContext,
    record: &Rc<RefCell<CourseRecord>>,
) -> CourseVisualState {
    let borrowed = record.borrow();
    let course_manager = app_ctx.course_manager.borrow_mut();

    // Hovered? Check group relationship
    if let Some(hovered) = &ctx.hovered_render {
        if !Rc::ptr_eq(record, hovered) {
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
    }

    // Selected + clashing?
    if course_manager.is_selected(record) {
        return if course_manager.is_clashing(record) {
            CourseVisualState::Clashing
        } else {
            CourseVisualState::Selected
        };
    }

    // Closed?
    if borrowed.is_closed() {
        return CourseVisualState::Closed;
    }

    CourseVisualState::Default
}

fn get_course_colors(
    app_ctx: &CrynContext,
    response: &Response,
    record_rc: &Rc<RefCell<CourseRecord>>, // For clash checking
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

    // Override text color to red if clashing
    let text_color = app_ctx
        .course_manager
        .borrow()
        .is_clashing(record_rc)
        .then(|| Color32::RED)
        .unwrap_or(state.text);

    (state.bg, text_color)
}
