use super::parsers::standard_course_parser;
use crate::models::{CourseDefinition, CourseRecord};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct CourseManager {
    pub course_definitions: Vec<Rc<RefCell<CourseDefinition>>>,
    pub course_records: Vec<Rc<RefCell<CourseRecord>>>, // Shouldve seen this coming lmao
    pub selected_records: Vec<Rc<RefCell<CourseRecord>>>,
    clashing_records: HashSet<*const RefCell<CourseRecord>>,
}

impl CourseManager {
    pub fn new() -> Self {
        CourseManager {
            course_definitions: Vec::new(),
            course_records: Vec::new(),
            selected_records: Vec::new(),
            clashing_records: HashSet::new(),
        }
    }

    pub fn get_or_add_course_definition(
        &mut self,
        code: &str,
        name: &str,
    ) -> Rc<RefCell<CourseDefinition>> {
        let pos = self
            .course_definitions
            .iter()
            .position(|c| c.borrow().code == code);

        match pos {
            Some(idx) => Rc::clone(&self.course_definitions[idx]),

            // Create new def
            None => {
                let new_def = Rc::new(RefCell::new(CourseDefinition::new(code, name))); /* Allocation here */
                self.course_definitions.push(Rc::clone(&new_def)); /* yes, i just got to know rc */
                new_def
            }
        }
    }

    pub fn parse_courses(&mut self, data: &str) {
        standard_course_parser::parse(self, data)
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
        self.clashing_records.contains(&Rc::as_ptr(record))
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
            self.selected_records.push(Rc::clone(record));
        }

        // Update clash cache
        self.recompute_clashes();
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
