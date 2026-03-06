use std::cell::RefCell;

use crate::models::CourseDefinition;

#[derive(Default)]
pub struct CourseSummary {
    pub code: String, // for sorting
    pub name: String,
    pub selected_lec: Option<i32>,
    pub selected_tut: Option<i32>,
    pub definition: *const RefCell<CourseDefinition>, // shhh
    pub has_non_unique_name: bool,
    pub has_closed: bool,
    pub has_clashing: bool,
}
