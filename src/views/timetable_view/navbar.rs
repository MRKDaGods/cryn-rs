use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use egui::{
    Align, Color32, CornerRadius, CursorIcon, Frame, Label, Layout, Margin, Popup,
    PopupCloseBehavior, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};

use super::colors::COURSE_COLORS_UNK;
use super::listener::TimeTableListenerState;
use crate::models::CourseDefinition;
use crate::utils::ui::get_trunacted_text;
use crate::windows::NavbarInterface;
use crate::{CrynContext, icons};

pub(super) fn render_navbar(
    listener_state_rc: &Rc<RefCell<TimeTableListenerState>>,
    ui: &mut Ui,
    app_ctx: &CrynContext,
    _interface: &NavbarInterface,
) {
    let listener_state = listener_state_rc.borrow();
    if listener_state.course_summaries.is_empty() {
        return;
    }

    // Count issues
    let (closed_count, clashing_count) = listener_state
        .course_summaries
        .iter()
        .map(|s| (s.has_closed as u32, s.has_clashing as u32))
        .reduce(|prev, curr| (prev.0 + curr.0, prev.1 + curr.1))
        .unwrap_or_default();

    // Build inline summary text
    let mut summary_parts: Vec<String> = listener_state
        .course_summaries
        .iter()
        .map(|s| {
            let mut part = format!(
                "{}{} ({}/{})",
                if s.has_non_unique_name {
                    format!("[{}] ", &s.code)
                } else {
                    "".to_owned()
                },
                &s.name,
                s.selected_lec.map_or("NA".to_owned(), |g| g.to_string()),
                s.selected_tut.map_or("NA".to_owned(), |g| g.to_string()),
            );

            // Append status indicators
            if s.has_clashing {
                part.push_str(&format!(" {}", icons::WARNING));
            }

            if s.has_closed {
                part.push_str(&format!(" {}", icons::LOCK));
            }

            part
        })
        .collect();

    // Append issue summary
    if clashing_count > 0 || closed_count > 0 {
        let mut issues = Vec::new();
        if clashing_count > 0 {
            issues.push(format!("{} clashing", clashing_count));
        }

        if closed_count > 0 {
            issues.push(format!("{} closed", closed_count));
        }

        summary_parts.push(format!("[{}]", issues.join(", ")));
    }

    let summary_text = summary_parts.join(format!(" {} ", icons::SEPARATOR).as_str());

    // Render hover fill
    let rect = ui.max_rect();
    let area_response = ui.interact(rect, ui.id().with("summary_area"), Sense::click());

    // Check if popup is open
    let popup_id = Popup::default_response_id(&area_response);
    let is_popup_open = Popup::is_id_open(ui.ctx(), popup_id);
    let is_hovered = area_response.hovered();
    let is_active = is_hovered || is_popup_open;

    // Hover highlight
    if is_active {
        let highlight_color = if is_popup_open {
            Color32::from_white_alpha(16)
        } else {
            Color32::from_white_alpha(10)
        };
        ui.painter().rect_filled(rect, 0.0, highlight_color);
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(0.5, Color32::from_white_alpha(25)),
            StrokeKind::Inside,
        );
    }

    // Cursor
    if is_hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // Render summary text and manually pad
    let label_padding = 12.0;
    let padded_rect = rect.shrink2(Vec2::new(label_padding, 0.0));

    ui.scope_builder(egui::UiBuilder::new().max_rect(padded_rect), |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let width = ui.available_width();
            let summary_font_size = 13.0;
            let text = get_trunacted_text(ui, &summary_text, summary_font_size, width, 1);

            let mut rich_text = RichText::new(text).size(summary_font_size);
            if !is_active {
                rich_text = rich_text.weak();
            }

            ui.add(Label::new(rich_text).extend());
        });
    });

    // Popup
    let accent_color = ui.visuals().widgets.hovered.bg_fill;

    Popup::from_toggle_button_response(&area_response)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .frame(Frame {
            inner_margin: Margin::symmetric(12, 8),
            corner_radius: CornerRadius::same(8),
            shadow: egui::Shadow {
                offset: [0, -2],
                blur: 12,
                spread: 0,
                color: Color32::from_black_alpha(60),
            },
            stroke: Stroke::new(1.0, Color32::from_white_alpha(18)),
            fill: ui.visuals().window_fill,
            ..Default::default()
        })
        .show(|ui| {
            ui.set_min_width(320.0);
            ui.set_max_width(600.0);
            ui.spacing_mut().item_spacing.y = 4.0;

            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new(icons::LIBRARY).size(14.0).color(accent_color));
                ui.strong(format!(
                    "{} courses selected",
                    listener_state.course_summaries.len()
                ));
            });

            // Issue badges
            if clashing_count > 0 || closed_count > 0 {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    if clashing_count > 0 {
                        let clash_color = Color32::from_rgb(211, 84, 0);
                        ui.label(
                            RichText::new(format!(
                                "{} {} clashing",
                                icons::WARNING,
                                clashing_count
                            ))
                            .size(11.5)
                            .color(clash_color),
                        );
                    }

                    if closed_count > 0 {
                        let closed_color = Color32::from_gray(140);
                        ui.label(
                            RichText::new(format!("{} {} closed", icons::LOCK, closed_count))
                                .size(11.5)
                                .color(closed_color),
                        );
                    }
                });
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);

            // Course list
            let mut to_deselect = Vec::<*const RefCell<CourseDefinition>>::new();

            for (i, s) in listener_state.course_summaries.iter().enumerate() {
                let row_rect = ui
                    .horizontal(|ui| {
                        // Code
                        ui.label(RichText::new(&s.code).strong());

                        ui.label(RichText::new(icons::SEPARATOR).small().weak());

                        // Name
                        ui.label(RichText::new(&s.name));

                        // Lec/Tut groups
                        let lec = s.selected_lec.map_or("NA".to_owned(), |g| g.to_string());
                        let tut = s.selected_tut.map_or("NA".to_owned(), |g| g.to_string());

                        ui.label(
                            RichText::new(format!("(Lec {} / Tut {})", lec, tut))
                                .size(11.0)
                                .weak(),
                        );

                        // Remove button + status indicators
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let remove_btn = ui.add(
                                egui::Button::new(RichText::new(icons::CLOSE).size(9.0))
                                    .frame(false),
                            );

                            if remove_btn.hovered() {
                                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                            }

                            if remove_btn.on_hover_text("Remove course").clicked() {
                                to_deselect.push(s.definition);
                            }

                            // Status indicators
                            if s.has_closed {
                                ui.label(
                                    RichText::new(format!("{} Closed", icons::LOCK))
                                        .size(10.5)
                                        .color(Color32::from_gray(140)),
                                );
                            }

                            if s.has_clashing {
                                let clash_col = COURSE_COLORS_UNK.clashing.normal.bg;

                                ui.label(
                                    RichText::new(format!("{} Clash", icons::WARNING))
                                        .size(10.5)
                                        .color(clash_col),
                                );
                            }
                        });
                    })
                    .response
                    .rect;

                // Row background
                let row_bg_rect = row_rect.expand2(Vec2::new(4.0, 1.0));
                if s.has_clashing {
                    ui.painter().rect_filled(
                        row_bg_rect,
                        2.0,
                        Color32::from_rgba_premultiplied(211, 84, 0, 18),
                    );
                } else if s.has_closed {
                    ui.painter().rect_filled(
                        row_bg_rect,
                        2.0,
                        Color32::from_rgba_premultiplied(80, 80, 80, 18),
                    );
                } else if i % 2 == 1 {
                    ui.painter()
                        .rect_filled(row_bg_rect, 2.0, Color32::from_white_alpha(6));
                }
            }

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(4.0);

            // Action buttons row
            let btn_height = 28.0;
            ui.columns(3, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    let copy_id = ui.id().with("summary_copy");
                    let last_copy_time = ui.data(|d| d.get_temp::<SystemTime>(copy_id));

                    let is_copied = last_copy_time.is_some_and(|t| {
                        SystemTime::now()
                            .duration_since(t)
                            .is_ok_and(|d| d.as_secs() < 2)
                    });

                    // Disable if recently copied
                    if is_copied {
                        ui.disable();
                    }

                    let text = if is_copied { "Copied!" } else { "Copy" };
                    if ui
                        .add_sized(
                            [ui.available_width(), btn_height],
                            egui::Button::new(RichText::new(text).size(12.5)).corner_radius(4.0),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(summary_text);

                        // Update last copy time
                        ui.data_mut(|d| d.insert_temp(copy_id, SystemTime::now()));
                    }
                });
                cols[1].vertical_centered_justified(|ui| {
                    if ui
                        .add_sized(
                            [ui.available_width(), btn_height],
                            egui::Button::new(RichText::new("Clear All").size(12.5))
                                .corner_radius(4.0),
                        )
                        .clicked()
                    {
                        to_deselect = listener_state
                            .course_summaries
                            .iter()
                            .map(|s| s.definition)
                            .collect();
                    }
                });
                cols[2].vertical_centered_justified(|ui| {
                    if ui
                        .add_sized(
                            [ui.available_width(), btn_height],
                            egui::Button::new(RichText::new("Import").size(12.5))
                                .corner_radius(4.0),
                        )
                        .clicked()
                    {
                        app_ctx.show_import_window();
                        Popup::close_id(ui.ctx(), popup_id);
                    }
                });
            });

            // Apply deselections
            // Release borrow since notify_listeners will borrow listener_state again
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
