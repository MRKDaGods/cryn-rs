use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::services::CourseManager;
use crate::utils;
use crate::windows::MainWindow;

pub struct CrynContext {
    pub course_manager: Rc<RefCell<CourseManager>>,
}

pub struct CrynApp {
    /* Windows */
    main_window: MainWindow,

    /* Whatever */
    _course_manager: Rc<RefCell<CourseManager>>,
    context: CrynContext,
}

impl CrynApp {
    /// App ctor
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        utils::log("Cryn started");

        // Configure theme
        //let mut visuals = egui::Visuals::dark();
        //visuals.override_text_color = Some(egui::Color32::WHITE);
        //cc.egui_ctx.set_visuals(visuals);

        // Fonts
        Self::setup_fonts(cc);

        // Turn off text selection
        cc.egui_ctx.style_mut(|style| {
            style.interaction.selectable_labels = false;
        });

        let course_manager = Self::initialize_course_manager(); /* Original ref */
        let app_ctx = CrynContext {
            course_manager: Rc::clone(&course_manager),
        };

        Self {
            main_window: MainWindow::new(&app_ctx),
            _course_manager: course_manager,
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

        // utils::log(
        //     format!(
        //         "Courses: {:?}",
        //         course_manager
        //             .course_records
        //             .iter()
        //             .filter(|x| !x.borrow().course_definition.borrow().flags.is_empty())
        //             .collect::<Vec<_>>()
        //     )
        //     .as_str(),
        // );

        Rc::new(RefCell::new(course_manager))
    }
}

// App render loop
impl eframe::App for CrynApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render main window
        self.main_window.render(ctx, &self.context);
    }
}
