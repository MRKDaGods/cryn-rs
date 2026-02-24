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
        default: build_interaction(base, 190), // Base gray for text
        selected: build_interaction([45, 90, 160], 220), // Vivid blue accent
        clashing: build_interaction([160, 45, 45], 220), // Vivid red accent
        closed: build_interaction([100, 45, 48], 160), // Muted red-gray
        group_match: build_interaction([40, 100, 60], 210), // Muted green-gray
        group_mismatch: build_interaction([120, 75, 30], 210), // Muted orange-gray
    }
}

fn build_interaction(base: [u8; 3], text_gray: u8) -> CourseInteractionColors {
    CourseInteractionColors {
        normal: CourseStateColors {
            bg: Color32::from_rgb(base[0], base[1], base[2]),
            text: Color32::from_gray(text_gray),
        },
        hovered: CourseStateColors {
            bg: lighten(base, 18),
            text: Color32::from_gray(text_gray.saturating_add(30)),
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
pub static COURSE_COLORS_LEC: LazyLock<CourseColors> = LazyLock::new(|| build([48, 56, 70]));

// Muted green-gray base
pub static COURSE_COLORS_TUT: LazyLock<CourseColors> = LazyLock::new(|| build([56, 64, 58]));

// Neutral gray base
pub static COURSE_COLORS_UNK: LazyLock<CourseColors> = LazyLock::new(|| build([55, 55, 60]));
