use crate::models::{CourseSpan, OrderedWeekday};
use egui::{Frame, Label, Rect, RichText, Separator, Ui, UiBuilder, Vec2, epaint::MarginF32};
use std::collections::HashMap;

const TIMETABLE_PADDING: Vec2 = Vec2::new(8.0, 8.0);
const TIMETABLE_SPACING: Vec2 = Vec2::new(12.0, 32.0);

const TIMESLOT_HEADER_WIDTH: f32 = 95.0;
const TIMESLOT_HEADER_HEIGHT: f32 = 43.0;

const DAY_HEADER_HEIGHT: f32 = 42.0;
const COURSE_SLOT_HEIGHT: f32 = 65.5;

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
        }
    }
}

pub fn invalidate_layout(ctx: &mut LayoutContext) {
    ctx.sizes.clear();
}

pub fn begin(ctx: &mut LayoutContext, ui: &mut Ui) {
    ctx.container_rect = ui.available_rect_before_wrap();
    ctx.is_layout_pass = ctx.sizes.is_empty();
    ctx.day_index = 0;

    ctx.x = ctx.container_rect.left() + TIMETABLE_PADDING.x;
    ctx.y = ctx.container_rect.top() + TIMETABLE_PADDING.y;
    ctx.max_layout_height = 0.0;
}

pub fn render_day(ctx: &mut LayoutContext, ui: &mut Ui, day: &OrderedWeekday, span: &CourseSpan) {
    let mut ui_builder = UiBuilder::new();
    if !ctx.is_layout_pass {
        if let Some(size) = ctx.sizes.get(&day).cloned() {
            // Check for wrap, skip first day though
            if ctx.day_index > 0 && needs_wrap(ctx, &size) {
                wrap(ctx);
            }

            // Calculate rect and advance cursor
            let layout_rect = calculate_rect(ctx, &size);
            advance(ctx, &layout_rect);

            ui_builder = ui_builder.max_rect(layout_rect);
        } else {
            panic!("Missing layout size for day {:?} !!!", day);
        }
    }

    let period_count = span.period_count();
    let width = (period_count as f32) * TIMESLOT_HEADER_WIDTH + ((period_count - 1) as f32); // 1px for separator

    let layout_rect = ui
        .scope_builder(ui_builder, |ui| {
            Frame::group(ui.style())
                .inner_margin(MarginF32::ZERO)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Day header
                        ui.add_sized(
                            [width, DAY_HEADER_HEIGHT],
                            Label::new(RichText::new(day.to_string()).size(14.0).strong()),
                        );

                        // Day separator
                        ui.add_sized([width, 1.0], Separator::default().spacing(1.0));

                        // Time slots
                        ui.horizontal(|ui| {
                            for i in 0..period_count {
                                if i > 0 {
                                    ui.add_sized(
                                        [1.0, TIMESLOT_HEADER_HEIGHT],
                                        Separator::default().spacing(1.0),
                                    );
                                }

                                let hour = span.start_hour() + i;
                                ui.add_sized(
                                    [TIMESLOT_HEADER_WIDTH, TIMESLOT_HEADER_HEIGHT],
                                    Label::new(format!("{}:00\n{}:50", hour, hour)),
                                );
                            }
                        });
                    });
                });
        })
        .response
        .rect;

    // Record size for layout pass
    if ctx.is_layout_pass {
        ctx.sizes.insert(day.clone(), layout_rect.size());
    }

    ctx.day_index += 1;
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
