use std::any::TypeId;
use std::collections::HashMap;

use egui::epaint::MarginF32;
use egui::{CentralPanel, Frame};

use crate::CrynContext;
use crate::views::{CoursesView, MainWindowView, PlaceholderView, TimeTableView};
use crate::windows::Window;

mod nav_bar;
mod title_bar;

pub use nav_bar::NavbarInterface;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;

const TITLEBAR_HEIGHT: f32 = 40.0;
const NAVBAR_HEIGHT: f32 = 42.0;

#[allow(unused)]
pub const CONTENT_PADDING: f32 = 8.0;

#[derive(Default)]
pub struct MainWindow {
    views: HashMap<TypeId, Box<dyn MainWindowView>>,
    current_view_id: Option<TypeId>,

    /// Post-render target view switch request
    requested_target_view_id: Option<TypeId>,

    /// Safe guard to prevent switching views while rendering content
    is_rendering_content: bool,
}

impl MainWindow {
    fn register_view<V: MainWindowView + 'static>(&mut self, view: V) {
        self.views.insert(TypeId::of::<V>(), Box::new(view));
    }

    pub fn switch_to_view<V: MainWindowView + 'static>(&mut self, app_ctx: &CrynContext) {
        if self.is_rendering_content {
            // Defer view switch until after rendering
            self.request_switch_to_view::<V>();
            return;
        }

        self.switch_to_view_internal(TypeId::of::<V>(), app_ctx);
    }

    fn switch_to_view_internal(&mut self, target_id: TypeId, app_ctx: &CrynContext) {
        if self.current_view_id == Some(target_id) {
            return;
        }

        // Does target view exist?
        let view_exists = self.views.contains_key(&target_id);
        if !view_exists {
            return;
        }

        // Hide current
        if let Some(current_view_id) = self.current_view_id {
            let current_view = &mut self.views.get_mut(&current_view_id).unwrap();
            if !current_view.can_hide(app_ctx) {
                return;
            }

            current_view.on_hide(app_ctx);
        }

        // Update to new view
        self.current_view_id = Some(target_id); /* Copied */
        let target_view = &mut self.views.get_mut(&target_id).unwrap();
        target_view.on_show(app_ctx);
    }

    pub fn request_switch_to_view<V: MainWindowView + 'static>(&mut self) {
        self.requested_target_view_id = Some(TypeId::of::<V>());
    }

    /// Render the main content
    fn render_content(&mut self, ctx: &egui::Context, app_ctx: &CrynContext) {
        let view_padding = self
            .get_current_view()
            .and_then(|v| v.padding())
            .unwrap_or(MarginF32::ZERO);

        CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(view_padding)
                    .fill(ctx.style().visuals.window_fill),
            )
            .show(ctx, |ui| {
                // Render current view of ammar magnus
                if let Some(mut view) = self
                    .views
                    .remove(&self.current_view_id.unwrap_or(TypeId::of::<()>()))
                {
                    // Temporarily separate view
                    view.on_gui(ui, app_ctx, self);

                    // Put it back
                    self.views.insert(self.current_view_id.unwrap(), view);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.heading(match self.current_view_id {
                            Some(current_view_id) => {
                                format!("View {:?} not found", current_view_id)
                            }

                            // No view?
                            None => "No view set".to_owned(),
                        });
                    });
                }
            });
    }

    fn get_current_view(&mut self) -> Option<&mut dyn MainWindowView> {
        self.current_view_id
            .and_then(|id| self.views.get_mut(&id))
            .map(|v| v.as_mut())
    }
}

impl Window for MainWindow {
    fn initialize(&mut self, app_ctx: &CrynContext) {
        // Register views
        self.register_view(TimeTableView::default());
        self.register_view(CoursesView::new());
        self.register_view(PlaceholderView);

        // TT view by def
        self.switch_to_view::<TimeTableView>(app_ctx);
    }

    /// Main render method
    fn render(&mut self, ctx: &egui::Context, app_ctx: &CrynContext) {
        #[cfg(not(target_arch = "wasm32"))]
        desktop::handle_resize_events(ctx);

        // Title bar and window controls
        title_bar::render_title_bar(ctx, self.get_current_view().as_deref());

        // Nav bar
        nav_bar::render_nav_bar(self, ctx, app_ctx);

        // Content
        self.is_rendering_content = true;
        self.render_content(ctx, app_ctx);
        self.is_rendering_content = false;

        // Handle post-render requested view switch
        if let Some(target_view_id) = self.requested_target_view_id {
            self.switch_to_view_internal(target_view_id, app_ctx);
            self.requested_target_view_id = None;
        }
    }
}
