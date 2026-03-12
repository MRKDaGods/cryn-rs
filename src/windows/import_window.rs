use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use egui::{Align, Button, Color32, Id, Layout, Modal, RichText, ScrollArea, TextEdit, Vec2};

use crate::models::CourseSummary;
use crate::services::CourseManager;
use crate::services::parsers::summary_parser;
use crate::windows::Window;
use crate::{CrynContext, icons};

const MODAL_WIDTH: f32 = 660.0;
const PREVIEW_MAX_HEIGHT: f32 = 180.0;
const WARNINGS_MAX_HEIGHT: f32 = 70.0;

#[derive(Default)]
enum PreviewState {
    #[default]
    Empty,
    NoMatches,
    Success(Vec<CourseSummary>, Vec<String>),
    Error(String),
}

#[derive(Default)]
pub struct ImportWindow {
    input_text: String,
    preview: PreviewState,
    last_input_hash: u64,
}

impl ImportWindow {
    fn hash_input(text: &str) -> u64 {
        let mut h = DefaultHasher::new();
        text.hash(&mut h);
        h.finish()
    }

    fn update_preview(&mut self, course_manager: &CourseManager) {
        // Compute hash and skip if unchanged
        let hash = Self::hash_input(&self.input_text);
        if hash == self.last_input_hash {
            return;
        }
        self.last_input_hash = hash;

        let data = self.input_text.trim();
        if data.is_empty() {
            self.preview = PreviewState::Empty;
            return;
        }

        // Parse preview
        let mut errors = Vec::new();
        let summaries = summary_parser::parse(course_manager, data, &mut errors);

        // Only treat as a hard error when nothing could be resolved at all
        self.preview = if summaries.is_empty() {
            if errors.is_empty() {
                PreviewState::NoMatches
            } else {
                PreviewState::Error(errors.join("\n"))
            }
        } else {
            PreviewState::Success(summaries, errors)
        };
    }

    fn can_import(&self) -> bool {
        matches!(&self.preview, PreviewState::Success(..))
    }

    fn apply_import(&self, app_ctx: &CrynContext) {
        let data = self.input_text.trim();
        let summaries = {
            let cm = app_ctx.course_manager.borrow();
            summary_parser::parse(&cm, data, &mut Vec::new())
        };
        app_ctx
            .course_manager
            .borrow_mut()
            .import_summaries(summaries);
    }

    fn render_preview(&self, ui: &mut egui::Ui) {
        match &self.preview {
            PreviewState::Empty => {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        RichText::new("Paste course data above to preview")
                            .color(Color32::from_gray(100))
                            .size(13.0),
                    );
                    ui.add_space(24.0);
                });
            }

            PreviewState::NoMatches => {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(icons::WARNING)
                            .color(Color32::from_rgb(200, 160, 50))
                            .size(18.0),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("No courses found in the provided text")
                            .color(Color32::from_rgb(200, 160, 50))
                            .size(13.0),
                    );
                    ui.label(
                        RichText::new("Make sure you're pasting the course summary text")
                            .color(Color32::from_gray(120))
                            .size(12.0),
                    );
                    ui.add_space(16.0);
                });
            }

            PreviewState::Error(msg) => {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::WARNING)
                            .color(Color32::from_rgb(220, 60, 60))
                            .size(16.0),
                    );
                    ui.label(
                        RichText::new(format!("Parse error: {msg}"))
                            .color(Color32::from_rgb(220, 60, 60))
                            .size(13.0),
                    );
                });
                ui.add_space(8.0);
            }

            PreviewState::Success(summaries, warnings) => {
                render_success(ui, summaries, warnings);
            }
        }
    }
}

fn render_success(ui: &mut egui::Ui, summaries: &[CourseSummary], errors: &[String]) {
    // Summary line
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} course{}",
                summaries.len(),
                if summaries.len() != 1 { "s" } else { "" },
            ))
            .color(Color32::from_rgb(80, 180, 80))
            .strong()
            .size(13.0),
        );
    });

    ui.add_space(6.0);

    // Course grid
    ScrollArea::vertical()
        .id_salt("preview_courses")
        .max_height(PREVIEW_MAX_HEIGHT)
        .show(ui, |ui| {
            egui::Grid::new("course_preview_grid")
                .num_columns(5)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let hdr = |ui: &mut egui::Ui, text: &str| {
                        ui.label(
                            RichText::new(text)
                                .color(Color32::from_gray(140))
                                .size(11.0)
                                .strong(),
                        );
                    };

                    // Header row
                    hdr(ui, "Code");
                    hdr(ui, "Name");
                    hdr(ui, "Lec");
                    hdr(ui, "Tut");
                    ui.end_row();

                    // Data rows
                    for course in summaries {
                        ui.label(
                            RichText::new(&course.code)
                                .strong()
                                .size(12.0)
                                .color(Color32::from_rgb(130, 170, 210)),
                        );
                        ui.label(RichText::new(&course.name).size(12.0));
                        ui.label(
                            RichText::new(
                                course
                                    .selected_lec
                                    .map_or_else(|| "NA".to_string(), |g| g.to_string()),
                            )
                            .size(12.0),
                        );
                        ui.label(
                            RichText::new(
                                course
                                    .selected_tut
                                    .map_or_else(|| "NA".to_string(), |g| g.to_string()),
                            )
                            .size(12.0),
                        );

                        ui.end_row();
                    }
                });
        });

    // Errors
    if !errors.is_empty() {
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} {} error{}",
                    icons::WARNING,
                    errors.len(),
                    if errors.len() != 1 { "s" } else { "" }
                ))
                .color(Color32::LIGHT_RED)
                .strong()
                .size(12.0),
            );
        });

        ui.add_space(2.0);

        ScrollArea::vertical()
            .id_salt("preview_warnings")
            .max_height(WARNINGS_MAX_HEIGHT)
            .show(ui, |ui| {
                for warning in errors {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("  •")
                                .color(Color32::from_rgb(230, 180, 50))
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(warning)
                                .color(Color32::from_gray(160))
                                .size(11.0),
                        );
                    });
                }
            });
    }
}

impl Window for ImportWindow {
    fn render(&mut self, ctx: &egui::Context, app_ctx: &CrynContext) {
        Modal::new(Id::new("import_window")).show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);

            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icons::IMPORT)
                        .size(20.0)
                        .color(Color32::from_rgb(100, 160, 220)),
                );
                ui.label(RichText::new("Import Courses").size(20.0).strong());
            });

            ui.add_space(2.0);
            ui.label(
                RichText::new(
                    "Paste the course summary text below. Entries are parsed in real-time.",
                )
                .color(Color32::from_gray(140))
                .size(12.0),
            );

            ui.add_space(10.0);

            // Input area
            ui.add(
                TextEdit::multiline(&mut self.input_text)
                    .hint_text("Paste course summary text here...")
                    .desired_width(f32::INFINITY)
                    .desired_rows(8)
                    .font(egui::TextStyle::Monospace),
            );

            // Parse on text change
            let course_manager = app_ctx.course_manager.borrow();
            self.update_preview(&course_manager);
            drop(course_manager);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Preview header
            ui.label(
                RichText::new("Preview")
                    .size(14.0)
                    .strong()
                    .color(Color32::from_gray(180)),
            );
            ui.add_space(4.0);

            // Preview content
            self.render_preview(ui);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Action buttons
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Cancel
                    if ui
                        .add(
                            Button::new(RichText::new("Cancel").size(13.0))
                                .min_size(Vec2::new(80.0, 30.0)),
                        )
                        .clicked()
                    {
                        app_ctx.dispose_import_window();
                    }

                    ui.add_space(4.0);

                    // Import
                    let can_import = self.can_import();
                    let import_btn = ui.add_enabled(
                        can_import,
                        Button::new(
                            RichText::new(format!("{} Import", icons::IMPORT))
                                .size(13.0)
                                .color(if can_import {
                                    Color32::WHITE
                                } else {
                                    Color32::from_gray(100)
                                }),
                        )
                        .fill(if can_import {
                            Color32::from_rgb(40, 100, 160)
                        } else {
                            Color32::from_gray(50)
                        })
                        .min_size(Vec2::new(100.0, 30.0)),
                    );

                    if import_btn.clicked() {
                        self.apply_import(app_ctx);
                        app_ctx.dispose_import_window();
                    }
                });
            });
        });
    }
}
