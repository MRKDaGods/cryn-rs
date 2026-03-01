use std::sync::LazyLock;

use egui::Color32;

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
    let white = Color32::WHITE;
    let muted = Color32::from_gray(180);
    let red = Color32::from_rgb(255, 80, 80);

    CourseColors {
        default: build_interaction(base, white),
        selected: build_interaction([211, 84, 0], white),
        clashing: build_interaction([211, 84, 0], red),
        closed: build_interaction([8, 8, 8], muted),
        group_match: build_interaction([40, 100, 60], white),
        group_mismatch: build_interaction([120, 75, 30], white),
    }
}

fn build_interaction(base: [u8; 3], text: Color32) -> CourseInteractionColors {
    CourseInteractionColors {
        normal: CourseStateColors {
            bg: Color32::from_rgb(base[0], base[1], base[2]),
            text,
        },
        hovered: CourseStateColors {
            bg: lighten(base, 18),
            text,
        },
        active: CourseStateColors {
            bg: lighten(base, 30),
            text,
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
