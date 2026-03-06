use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::models::{
    CourseDefinition, CourseEvent, CourseEventListener, CourseFlags, CourseRecord,
    CourseRecordType, CourseSummary,
};
use crate::services::CourseManager;

#[derive(Default)]
pub(super) struct TimeTableListenerState {
    pub course_summaries: Vec<CourseSummary>,
}

impl TimeTableListenerState {
    fn rebuild_summary(
        &mut self,
        course_manager: &CourseManager,
        selected_records: &[Rc<RefCell<CourseRecord>>],
    ) {
        // Rebuild summaries
        self.course_summaries.clear();

        if selected_records.is_empty() {
            return;
        }

        // Group selected records by course def
        let mut course_record_map =
            HashMap::<*const RefCell<CourseDefinition>, Vec<Rc<RefCell<CourseRecord>>>>::new();

        selected_records.iter().for_each(|r| {
            let record = r.borrow();
            course_record_map
                .entry(Rc::as_ptr(&record.course_definition))
                .or_default()
                .push(Rc::clone(r));
        });

        // Map into CourseSummary
        let mut summaries = course_record_map
            .iter()
            .map(|(def_ptr, records)| {
                let def = unsafe { &**def_ptr }.borrow();

                let mut summary = CourseSummary {
                    code: def.code.clone(),
                    name: def.name.clone(),
                    definition: *def_ptr,
                    has_non_unique_name: def.flags.contains(CourseFlags::NonUniqueName),
                    ..Default::default()
                };

                records.iter().for_each(|r| {
                    let record = r.borrow();
                    match record.record_type {
                        CourseRecordType::Lecture => summary.selected_lec = Some(record.group),
                        CourseRecordType::Tutorial => summary.selected_tut = Some(record.group),
                        CourseRecordType::None => (),
                    }

                    if record.is_closed() {
                        summary.has_closed = true;
                    }

                    if course_manager.is_clashing_raw(Rc::as_ptr(r)) {
                        summary.has_clashing = true;
                    }
                });

                summary
            })
            .collect::<Vec<_>>();

        // Sort by code
        summaries.sort_by(|a, b| a.code.cmp(&b.code));

        // Update summaries
        self.course_summaries = summaries;
    }
}

impl CourseEventListener for TimeTableListenerState {
    fn on_course_event(&mut self, course_manager: &CourseManager, event: &CourseEvent) {
        match event {
            CourseEvent::SelectionChanged(selected_records) => {
                self.rebuild_summary(course_manager, selected_records);
            }
        }
    }
}
