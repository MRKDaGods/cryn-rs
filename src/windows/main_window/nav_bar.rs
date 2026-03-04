#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::any::TypeId;

use egui::text::LayoutJob;
use egui::{
    Align, Button, Color32, Context, CursorIcon, FontId, Frame, Layout, Margin, Stroke, TextFormat,
    TopBottomPanel, Ui, UiBuilder,
};

use super::{MainWindow, NAVBAR_HEIGHT};
use crate::views::{CoursesView, MainWindowView, PlaceholderView, TimeTableView};
use crate::{CrynContext, icons};

/// Padding for view-specific content
const NAVBAR_VIEW_CONTENT_PADDING: f32 = 12.0;

pub struct NavbarInterface<'a> {
    pub render_button_fn: &'a dyn Fn(&mut Ui, &str, &str, Option<f32>, Option<&dyn Fn()>),
}

pub fn render_nav_bar(main_window: &mut MainWindow, ctx: &Context, app_ctx: &CrynContext) {
    let button_width = (ctx.content_rect().width() / 8.0).clamp(100.0, 150.0);

    // Setup interface
    let navbar_interface = NavbarInterface {
        render_button_fn: &|ui, icon, label, requested_width, on_click| {
            let clicked = render_button(
                None,
                ui,
                icon,
                label,
                requested_width.unwrap_or(button_width),
                None::<fn(&mut MainWindow)>,
                None,
            );

            if clicked && let Some(on_click) = on_click {
                on_click();
            }
        },
    };

    TopBottomPanel::bottom("navbar")
        .frame(Frame {
            inner_margin: Margin::ZERO,
            fill: ctx.style().visuals.window_fill,
            ..Default::default()
        })
        .exact_height(NAVBAR_HEIGHT)
        .show_separator_line(true)
        .show(ctx, |ui| {
            ui.scope(|ui| {
                let style = ui.style_mut();

                // No button spacing
                style.spacing.item_spacing.x = 0.0;

                // Remove bg
                style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

                // Remove border
                style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                style.visuals.widgets.active.bg_stroke = Stroke::NONE;

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    // Left side buttons
                    render_button_view::<TimeTableView>(
                        main_window,
                        app_ctx,
                        ui,
                        icons::CALENDAR,
                        "Time Table",
                        button_width,
                    );

                    render_button_view::<CoursesView>(
                        main_window,
                        app_ctx,
                        ui,
                        icons::LIBRARY,
                        "Courses",
                        button_width,
                    );

                    // Right side buttons
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        render_button_view::<PlaceholderView>(
                            main_window,
                            app_ctx,
                            ui,
                            icons::SETTINGS,
                            "Settings",
                            button_width,
                        );

                        render_button(
                            Some(main_window),
                            ui,
                            icons::SCREENSHOT,
                            "Screenshot",
                            button_width,
                            Some(|_: &mut MainWindow| {}),
                            Some(false),
                        );

                        // Render view-specific navbar content
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if let Some(current_view) = main_window.get_current_view() {
                                let padding = current_view
                                    .navbar_padding()
                                    .unwrap_or(NAVBAR_VIEW_CONTENT_PADDING);
                                let max_rect = ui.available_rect_before_wrap().shrink(padding);
                                ui.scope_builder(UiBuilder::new().max_rect(max_rect), |ui| {
                                    current_view.on_navbar_gui(ui, app_ctx, &navbar_interface);
                                });
                            }
                        });
                    });
                });
            });
        });
}

fn render_button_view<V: MainWindowView + 'static>(
    main_window: &mut MainWindow,
    app_ctx: &CrynContext,
    ui: &mut Ui,
    icon: &str,
    label: &str,
    button_width: f32,
) {
    let is_active = main_window.current_view_id == Some(TypeId::of::<V>());

    render_button(
        Some(main_window),
        ui,
        icon,
        label,
        button_width,
        Some(|mw: &mut MainWindow| mw.switch_to_view::<V>(app_ctx)),
        Some(is_active),
    );
}

fn render_button(
    main_window: Option<&mut MainWindow>,
    ui: &mut Ui,
    icon: &str,
    label: &str,
    button_width: f32,
    on_click: Option<impl Fn(&mut MainWindow)>,
    is_active: Option<bool>,
) -> bool {
    let hover_id = ui.id().with((label, "hover"));
    let is_hovered = ui.data(|d| d.get_temp::<bool>(hover_id).unwrap_or(false));

    let fore_color = match is_active {
        Some(true) => ui.style().visuals.strong_text_color(),
        _ if is_hovered => ui.style().visuals.widgets.hovered.fg_stroke.color,
        _ => ui.style().visuals.text_color(),
    };

    let mut job = LayoutJob::default();

    // Icon
    job.append(
        icon,
        0.0,
        TextFormat {
            font_id: FontId::proportional(14.5),
            color: fore_color,
            line_height: Some(6.0),
            valign: Align::TOP, // Fixup
            ..Default::default()
        },
    );

    // Label
    job.append(
        label,
        8.0,
        TextFormat {
            font_id: FontId::proportional(13.5),
            color: fore_color,
            ..Default::default()
        },
    );

    let response = ui.add_sized(
        egui::vec2(button_width, ui.available_height()),
        Button::new(job).corner_radius(0.0),
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // Store hover state
    ui.data_mut(|d| d.insert_temp(hover_id, response.hovered()));

    let clicked = response.clicked();

    if clicked
        && let Some(on_click) = on_click
        && let Some(main_window) = main_window
    {
        on_click(main_window);
    }

    clicked
}
