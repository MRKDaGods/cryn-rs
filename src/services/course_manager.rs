use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::parsers::standard_course_parser;
use crate::models::{
    CourseDefinition, CourseEvent, CourseEventListener, CourseRecord, CourseRecordType,
    CourseSummary,
};

#[derive(Default)]
pub struct CourseManager {
    pub course_definitions: Vec<Rc<RefCell<CourseDefinition>>>,
    pub course_records: Vec<Rc<RefCell<CourseRecord>>>, // Shouldve seen this coming lmao
    pub selected_records: Vec<Rc<RefCell<CourseRecord>>>,
    clashing_records: HashSet<*const RefCell<CourseRecord>>,
    consumers: Vec<Rc<RefCell<dyn CourseEventListener>>>,
}

impl CourseManager {
    /// Returns the course definition for the given code, creating it if it doesnt exist.
    ///
    /// The boolean indicates whether a new definition was created.
    pub fn get_or_add_course_definition(
        &mut self,
        code: &str,
        name: &str,
    ) -> (Rc<RefCell<CourseDefinition>>, bool) {
        let pos = self
            .course_definitions
            .iter()
            .position(|c| c.borrow().code == code);

        match pos {
            Some(idx) => (Rc::clone(&self.course_definitions[idx]), false),

            // Create new def
            None => {
                let new_def = Rc::new(RefCell::new(CourseDefinition::new(code, name))); /* Allocation here */
                self.course_definitions.push(Rc::clone(&new_def)); /* yes, i just got to know rc */
                (new_def, true)
            }
        }
    }

    pub fn get_definition_by_code(&self, code: &str) -> Option<Rc<RefCell<CourseDefinition>>> {
        self.course_definitions
            .iter()
            .find(|c| c.borrow().code == code)
            .cloned()
    }

    pub fn get_definitions_by_name(&self, name: &str) -> Vec<Rc<RefCell<CourseDefinition>>> {
        self.course_definitions
            .iter()
            .filter(|c| c.borrow().name == name)
            .cloned()
            .collect()
    }

    pub fn parse_courses(&mut self, data: &str) {
        standard_course_parser::parse(self, data)
    }

    /// Applies a parsed summary as a preset selection.
    ///
    /// Clears existing selections, marks each definition as selected,
    /// and selects the matching lec/tut records.
    pub fn import_summaries(&mut self, summaries: Vec<CourseSummary>) {
        // Clear previous selections
        self.course_definitions
            .iter()
            .for_each(|def_rc| def_rc.borrow_mut().selected = false);
        self.selected_records.clear();

        for summary in &summaries {
            // Mark the definition as selected
            let Some(def_rc) = self
                .course_definitions
                .iter()
                .find(|d| Rc::as_ptr(d) == summary.definition)
                .cloned()
            else {
                // Definition not found, skip this summary
                continue;
            };
            def_rc.borrow_mut().selected = true;

            // Find and push the first matching record
            let mut select_record = |record_type: CourseRecordType, group: i32| {
                if let Some(record) = self
                    .course_records
                    .iter()
                    .find(|r| {
                        let r = r.borrow();
                        Rc::as_ptr(&r.course_definition) == summary.definition
                            && r.record_type == record_type
                            && r.group == group
                    })
                    .cloned()
                {
                    self.selected_records.push(record);
                }
            };

            if let Some(lec) = summary.selected_lec {
                select_record(CourseRecordType::Lecture, lec);
            }

            if let Some(tut) = summary.selected_tut {
                select_record(CourseRecordType::Tutorial, tut);
            }
        }

        // Dirty!
        self.on_courses_changed();

        // Notify listeners of rebuild request
        self.notify_listeners(CourseEvent::SummaryImported);
    }

    pub fn get_available_course_records(&self) -> Vec<Rc<RefCell<CourseRecord>>> {
        self.course_records
            .iter()
            .filter(|&record| record.borrow().course_definition.borrow().selected)
            .cloned() // &Rc<RefCell<CourseRecord>> -> Rc<RefCell<CourseRecord>>
            .collect()
    }

    pub fn is_selected(&self, record: &Rc<RefCell<CourseRecord>>) -> bool {
        self.selected_records.iter().any(|r| Rc::ptr_eq(r, record))
    }

    pub fn is_clashing(&self, record: &Rc<RefCell<CourseRecord>>) -> bool {
        self.is_clashing_raw(Rc::as_ptr(record))
    }

    pub fn is_clashing_raw(&self, record_ptr: *const RefCell<CourseRecord>) -> bool {
        self.clashing_records.contains(&record_ptr)
    }

    pub fn toggle_selected_course(&mut self, record: &Rc<RefCell<CourseRecord>>) {
        // Remove if exists, add if not
        if let Some(record_idx) = self
            .selected_records
            .iter()
            .position(|r| Rc::ptr_eq(r, record))
        {
            self.selected_records.remove(record_idx);
        } else {
            // If we select a lec of say group 1, we should deselect any other lec
            // We guarantee that there is only 1 selected lec/tutorial per group

            if let Some(other_idx) = self.selected_records.iter().position(|r| {
                // Same course and same type
                let our_record = record.borrow();
                let other_record = r.borrow();
                Rc::ptr_eq(
                    &other_record.course_definition,
                    &our_record.course_definition,
                ) && other_record.record_type == our_record.record_type
            }) {
                // Deselect it
                self.selected_records.remove(other_idx);
            }

            // Select our new one
            self.selected_records.push(Rc::clone(record));
        }

        self.on_courses_changed();
    }

    /// Removes selected records that have their course definition unselected
    pub fn update_selected_records(&mut self) {
        self.selected_records
            .retain(|record| record.borrow().course_definition.borrow().selected);

        self.on_courses_changed();
    }

    pub fn register_listener(&mut self, listener: Rc<RefCell<dyn CourseEventListener>>) {
        self.consumers.push(listener);
    }

    fn notify_listeners(&mut self, event: CourseEvent) {
        self.consumers.iter().for_each(|listener| {
            listener.borrow_mut().on_course_event(self, &event);
        });
    }

    pub fn unregister_listener(&mut self, listener: Rc<RefCell<dyn CourseEventListener>>) {
        self.consumers.retain(|l| !Rc::ptr_eq(l, &listener));
    }

    /// Completely deselects all records of the provided course definition
    pub fn deselect_course_records(
        &mut self,
        def_ptr: *const RefCell<CourseDefinition>,
        is_batch: bool,
    ) {
        self.selected_records
            .retain(|record| def_ptr != Rc::as_ptr(&record.borrow().course_definition));

        // Dont bother if we're removing courses in a batch
        if !is_batch {
            self.on_courses_changed();
        }
    }

    /// Called when courses have changed (selections updated, records added/removed, etc)
    fn on_courses_changed(&mut self) {
        // Update clash cache
        self.recompute_clashes();

        // Notify listeners
        self.notify_listeners(CourseEvent::SelectionChanged(self.selected_records.clone()));
    }

    fn recompute_clashes(&mut self) {
        // Clear old cache
        self.clashing_records.clear();

        // Check for overlaps
        for i in 0..self.selected_records.len() {
            for j in (i + 1)..self.selected_records.len() {
                let a = self.selected_records[i].borrow();
                let b = self.selected_records[j].borrow();

                if a.day == b.day && CourseManager::times_overlap(&a, &b) {
                    // Both clash
                    self.clashing_records
                        .insert(Rc::as_ptr(&self.selected_records[i]));
                    self.clashing_records
                        .insert(Rc::as_ptr(&self.selected_records[j]));
                }
            }
        }
    }

    fn times_overlap(a: &CourseRecord, b: &CourseRecord) -> bool {
        a.start_time < b.end_time && b.start_time < a.end_time
    }
}
