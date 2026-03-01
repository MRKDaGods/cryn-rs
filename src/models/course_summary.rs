use std::cell::RefCell;

use crate::models::CourseDefinition;

#[derive(Default)]
pub struct CourseSummary {
    pub code: String, // For sorting
    pub name: String,
    pub selected_lec: Option<i32>,
    pub selected_tut: Option<i32>,
    pub definition: *const RefCell<CourseDefinition>, // shhh
}
