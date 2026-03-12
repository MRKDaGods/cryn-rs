use std::cell::RefCell;
use std::rc::Rc;

use crate::models::CourseRecord;
use crate::services::CourseManager;

#[derive(Debug, Clone)]
pub enum CourseEvent {
    SelectionChanged(Vec<Rc<RefCell<CourseRecord>>>),
    SummaryImported,
}

pub trait CourseEventListener {
    fn on_course_event(&mut self, course_manager: &CourseManager, event: &CourseEvent);
}
