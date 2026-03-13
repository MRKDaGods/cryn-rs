use std::cell::RefCell;
use std::rc::Rc;

use egui::epaint::MarginF32;
use egui::{
    Align, CentralPanel, Frame, Label, Layout, Response, Sense, TextEdit, TextWrapMode,
    TopBottomPanel,
};
use egui_extras::{Column, TableBuilder};

use super::View;
use crate::CrynContext;
use crate::models::CourseDefinition;
use crate::views::MainWindowView;
use crate::windows::main_window::CONTENT_PADDING;
use crate::windows::{NavbarInterface, Window};

const SEARCH_HEIGHT: f32 = 35.0;
const HEADER_HEIGHT: f32 = 25.0;
const ROW_HEIGHT: f32 = 30.0;

#[derive(Default)]
pub struct CoursesView {
    hovered_row_idx: Option<usize>,
    selected_row_idx: Option<usize>,
    any_hovered: bool, // Prevent hover when mouse isnt over the table
    search_query: String,
    last_search_query: String,
    filtered_indices: Vec<usize>,
}

impl CoursesView {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_filter(&mut self, definitions: &[Rc<RefCell<CourseDefinition>>]) {
        let query = self.search_query.to_lowercase();
        if query == self.last_search_query {
            return;
        }

        // Invalidate hovered and selected states
        self.hovered_row_idx = None;
        self.selected_row_idx = None;

        if query.is_empty() {
            self.create_default_indices(definitions);
            self.last_search_query = query;
            return;
        }

        let mut code_indices: Vec<usize> = Vec::new();
        let mut name_indices: Vec<usize> = Vec::new();

        // Quick feat for testing
        // Cmps123, Ammr666
        let queries: Vec<&str> = query
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        definitions.iter().enumerate().for_each(|(index, def)| {
            let def = def.borrow();

            queries.iter().for_each(|&q| {
                if def.code.to_lowercase().contains(q) {
                    code_indices.push(index);
                }

                if def.name.to_lowercase().contains(q) {
                    name_indices.push(index);
                }
            });
        });

        self.filtered_indices = code_indices;
        self.filtered_indices.extend(name_indices);

        self.last_search_query = query;
    }

    fn create_default_indices(&mut self, definitions: &[Rc<RefCell<CourseDefinition>>]) {
        // Sort by selection, then by code
        let mut indices: Vec<usize> = (0..definitions.len()).collect();
        indices.sort_by(|&a, &b| {
            let def_a = definitions[a].borrow();
            let def_b = definitions[b].borrow();

            // Selected first
            if def_a.selected && !def_b.selected {
                return std::cmp::Ordering::Less;
            } else if !def_a.selected && def_b.selected {
                return std::cmp::Ordering::Greater;
            }

            // Then by code
            def_a.code.cmp(&def_b.code)
        });

        self.filtered_indices = indices;
    }

    /// Cant borrow self here since this is called in an already borrowed Self
    fn handle_row_events(
        row_response: &Response,
        row_index: usize,
        definition_selected: &mut bool,
        hovered_row_idx: &mut Option<usize>,
        selected_row_idx: &mut Option<usize>,
        any_hovered: &mut bool,
    ) {
        if row_response.double_clicked() {
            *definition_selected = !*definition_selected;
        }

        if row_response.clicked() {
            *selected_row_idx = Some(row_index);
            *hovered_row_idx = None;
        } else if row_response.hovered() {
            *hovered_row_idx = Some(row_index);
            *any_hovered = true;
        }
    }
}

impl View for CoursesView {
    fn name(&self) -> &str {
        "Courses"
    }

    fn on_show(&mut self, app_ctx: &CrynContext) {
        if self.search_query.is_empty() {
            let definitions = &app_ctx.course_manager.borrow().course_definitions;
            self.create_default_indices(definitions);
        }
    }

    fn on_hide(&mut self, app_ctx: &CrynContext) {
        self.hovered_row_idx = None;
        self.selected_row_idx = None;

        // Notify!
        app_ctx
            .course_manager
            .borrow_mut()
            .update_selected_records();
    }

    fn on_gui(&mut self, ui: &mut egui::Ui, app_ctx: &CrynContext, _window: &mut dyn Window) {
        let definitions = &app_ctx.course_manager.borrow().course_definitions;

        if definitions.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.heading("No courses found :( (Empty parse data)");
            });
            return;
        }

        // Search bar
        TopBottomPanel::top("courses_view_top_panel")
            .frame(
                Frame::new()
                    .inner_margin(MarginF32::same(CONTENT_PADDING))
                    .fill(ui.visuals().faint_bg_color),
            )
            .exact_height(SEARCH_HEIGHT)
            .show_inside(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search by code or name...")
                        .frame(false)
                        .vertical_align(Align::Center)
                        .horizontal_align(Align::Center),
                );
            });

        // Handle filtering
        self.update_filter(definitions);

        if self.filtered_indices.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.heading("No matching courses");
            });
            return;
        }

        // Render course table
        // HACK: Wrap table in a central panel cuz it glitches tf out lol
        // because of deferred padding
        CentralPanel::default()
            .frame(Frame::new().outer_margin(MarginF32 {
                left: CONTENT_PADDING,
                ..Default::default()
            }))
            .show_inside(ui, |ui| {
                // Full viewport
                let height = ui.available_height();

                // Keep track of last rendered course group to render groups
                let mut last_rendered_course_group = String::new();

                let Self {
                    filtered_indices,
                    hovered_row_idx,
                    selected_row_idx,
                    any_hovered,
                    ..
                } = self;

                // Clear hovered state
                *any_hovered = false;

                TableBuilder::new(ui)
                    .max_scroll_height(height)
                    .resizable(false)
                    .striped(true)
                    .auto_shrink(false)
                    .sense(Sense::click())
                    .column(Column::initial(100.0)) // Code
                    .column(Column::remainder().clip(true)) // Name
                    .column(Column::initial(80.0)) // Lecs
                    .column(Column::initial(80.0)) // Tuts
                    .column(Column::auto().at_least(200.0)) // Flags
                    .header(HEADER_HEIGHT, |mut header| {
                        header.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.strong("Code");
                            });
                        });

                        header.col(|ui| {
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.strong("Name");
                            });
                        });

                        header.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.strong("Lectures");
                            });
                        });

                        header.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.strong("Tutorials");
                            });
                        });

                        header.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.strong("Flags");
                            });
                        });
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, filtered_indices.len(), |mut row| {
                            let CourseDefinition {
                                code,
                                name,
                                flags,
                                selected,
                                lecture_count,
                                tutorial_count,
                                ..
                            } = &mut *definitions[filtered_indices[row.index()]].borrow_mut();

                            row.set_hovered(*hovered_row_idx == Some(row.index()));
                            row.set_selected(*selected_row_idx == Some(row.index()));

                            // Draw overline if new course group
                            let course_group = &code[..4];
                            row.set_overline(course_group != last_rendered_course_group);

                            if course_group != last_rendered_course_group {
                                last_rendered_course_group = course_group.to_owned();
                            }

                            // Code
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    let imm_code: &str = code.as_str();
                                    ui.checkbox(selected, imm_code);
                                });
                            });

                            // Name
                            row.col(|ui| {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    let imm_name: &str = name.as_str();
                                    ui.label(imm_name);
                                });
                            });

                            // Lectures
                            row.col(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(lecture_count.to_string());
                                });
                            });

                            // Tutorials
                            row.col(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(tutorial_count.to_string());
                                });
                            });

                            // Flags
                            row.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    flags.iter().for_each(|flag| {
                                        ui.add(
                                            Label::new(flag.to_string())
                                                .wrap_mode(TextWrapMode::Extend),
                                        );
                                    });
                                });
                            });

                            // Handle row events
                            CoursesView::handle_row_events(
                                &row.response(),
                                row.index(),
                                selected,
                                hovered_row_idx,
                                selected_row_idx,
                                any_hovered,
                            );
                        });
                    });
            });

        // Dont render hover state if none of our rows are hovered
        if !self.any_hovered {
            self.hovered_row_idx = None;
        }
    }
}

impl MainWindowView for CoursesView {
    fn navbar_padding(&self) -> Option<f32> {
        Some(0.0) // No padding, append Import button to navbar
    }

    fn on_navbar_gui(
        &mut self,
        ui: &mut egui::Ui,
        app_ctx: &CrynContext,
        interface: &NavbarInterface,
    ) {
        // Wesh endir hachkak rijali
        // Taste fr
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            (interface.render_button_fn)(
                ui,
                crate::icons::IMPORT,
                "Import List",
                None,
                Some(&|_| {
                    app_ctx.show_import_window();
                }),
            );
        });
    }
}
