use egui::Color32;
use std::sync::LazyLock;

pub struct CourseStateColors {
    pub bg: Color32,
    pub text: Color32,
}

pub struct CourseInteractionColors {
    pub normal: CourseStateColors,
    pub hovered: CourseStateColors,
    pub active: CourseStateColors,
}

pub struct CourseColors {
    pub default: CourseInteractionColors,
    pub selected: CourseInteractionColors,
    pub clashing: CourseInteractionColors,
    pub closed: CourseInteractionColors,
    pub group_match: CourseInteractionColors,
    pub group_mismatch: CourseInteractionColors,
}

#[derive(Default, Debug)]
pub enum CourseVisualState {
    #[default]
    Default,
    Selected,
    Clashing,
    Closed,
    GroupMatch,
    GroupMismatch,
}

fn build(base: [u8; 3]) -> CourseColors {
    CourseColors {
        default: build_interaction(base, 210),                       // Base gray for text
        selected: build_interaction([211, 84, 0], 230),              // Vivid blue accent
        clashing: build_interaction_rgb([211, 84, 0], [255, 0, 0]),  // Same color as selected but with red text
        closed: build_interaction([8, 8, 8], 170),                   // Dark gray
        group_match: build_interaction([40, 100, 60], 220),          // Muted green-gray
        group_mismatch: build_interaction([120, 75, 30], 220),       // Muted orange-gray
    }
}

fn build_interaction(base: [u8; 3], text_gray: u8) -> CourseInteractionColors {
    build_interaction_rgb(base, [text_gray; 3])
}

fn build_interaction_rgb(base: [u8; 3], text_rgb: [u8; 3]) -> CourseInteractionColors {
    CourseInteractionColors {
        normal: CourseStateColors {
            bg: Color32::from_rgb(base[0], base[1], base[2]),
            text: Color32::from_rgb(text_rgb[0], text_rgb[1], text_rgb[2]),
        },
        hovered: CourseStateColors {
            bg: lighten(base, 18),
            text: Color32::from_rgb(
                text_rgb[0].saturating_add(30),
                text_rgb[1].saturating_add(30),
                text_rgb[2].saturating_add(30),
            ),
        },
        active: CourseStateColors {
            bg: lighten(base, 30),
            text: Color32::WHITE,
        },
    }
}

fn lighten(base: [u8; 3], amount: u8) -> Color32 {
    Color32::from_rgb(
        base[0].saturating_add(amount),
        base[1].saturating_add(amount),
        base[2].saturating_add(amount),
    )
}

// I need better colors ngl

// Muted blue-gray base
pub static COURSE_COLORS_LEC: LazyLock<CourseColors> = LazyLock::new(|| build([52, 73, 94]));

// Muted green-gray base
pub static COURSE_COLORS_TUT: LazyLock<CourseColors> = LazyLock::new(|| build([44, 120, 115]));

// Neutral gray base
pub static COURSE_COLORS_UNK: LazyLock<CourseColors> = LazyLock::new(|| build([55, 55, 60]));
