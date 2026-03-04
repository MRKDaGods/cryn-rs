use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::services::CourseManager;
use crate::utils::{self, Signal};
use crate::windows::{ImportWindow, MainWindow, Window};

pub struct CrynContext {
    pub course_manager: Rc<RefCell<CourseManager>>,
    show_import_window: Signal,
    dispose_import_window: Signal,
}

impl CrynContext {
    pub fn new(course_manager: Rc<RefCell<CourseManager>>) -> Self {
        Self {
            course_manager,
            show_import_window: Signal::default(),
            dispose_import_window: Signal::default(),
        }
    }

    pub fn show_import_window(&self) {
        self.show_import_window.request();
    }

    pub fn dispose_import_window(&self) {
        self.dispose_import_window.request();
    }
}

pub struct CrynApp {
    main_window: MainWindow,

    /// Windows arent like views.
    /// They are only created when requested, and destroyed when closed.
    /// Hence keep import window as option.
    import_window: Option<ImportWindow>,

    context: CrynContext,
}

impl CrynApp {
    /// App ctor
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        utils::log("Cryn started");

        // Fonts
        Self::setup_fonts(cc);

        // Turn off text selection
        cc.egui_ctx.style_mut(|style| {
            style.interaction.selectable_labels = false;
        });

        let course_manager = Self::initialize_course_manager(); /* Original ref */
        let app_ctx = CrynContext::new(course_manager);

        let mut main_window = MainWindow::default();
        main_window.initialize(&app_ctx);

        Self {
            main_window,
            import_window: None,
            context: app_ctx,
        }
    }

    fn setup_fonts(cc: &eframe::CreationContext<'_>) {
        let mut fonts = egui::FontDefinitions::default();

        // Segoe UI, mdl2
        let extra_fonts: Vec<(&str, &[u8])> = vec![
            ("segoeui", include_bytes!("../assets/fonts/segoeui.ttf")),
            ("segmdl2", include_bytes!("../assets/fonts/segmdl2.ttf")),
        ];
        for (font_name, font_data) in extra_fonts {
            fonts.font_data.insert(
                font_name.to_owned(),
                Arc::new(egui::FontData::from_static(font_data)),
            );

            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, font_name.to_owned());

            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, font_name.to_owned());
        }

        cc.egui_ctx.set_fonts(fonts);
    }

    fn initialize_course_manager() -> Rc<RefCell<CourseManager>> {
        let mut course_manager = CourseManager::default();
        let data = include_str!("../assets/data/sample_courses.txt");
        course_manager.parse_courses(data);
        Rc::new(RefCell::new(course_manager))
    }
}

// App render loop
impl eframe::App for CrynApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render main window
        self.main_window.render(ctx, &self.context);

        // Check if import window requested
        let import_window_requested = self.context.show_import_window.consume();
        if import_window_requested && self.import_window.is_none() {
            // Create import window!
            let import_window = ImportWindow::default();
            self.import_window = Some(import_window);

            // Clear stray dispose reqs
            self.context.dispose_import_window.consume();
        }

        // Render import window if shown
        if let Some(import_window) = &mut self.import_window {
            import_window.render(ctx, &self.context);

            // Check if import window requested to close
            let import_window_disposed = self.context.dispose_import_window.consume();
            if import_window_disposed {
                // Dispose window
                import_window.on_dispose(&self.context);

                // Rip
                self.import_window = None;
            }
        }
    }
}
