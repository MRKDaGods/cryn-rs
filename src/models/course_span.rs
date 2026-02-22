use crate::models::CourseRecord;
use chrono::{NaiveTime, Timelike};
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub struct CourseSpan {
    map: HashMap<NaiveTime, Vec<Rc<RefCell<CourseRecord>>>>,
    min_from: Option<NaiveTime>,
    max_to: Option<NaiveTime>,

    grid: Vec<Vec<bool>>,
}

impl CourseSpan {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            min_from: None,
            max_to: None,
            grid: Vec::new(),
        }
    }

    pub fn insert_course_record(&mut self, record: &Rc<RefCell<CourseRecord>>) {
        let CourseRecord {
            start_time,
            end_time,
            ..
        } = *record.borrow();

        self.entry(start_time)
            .or_insert(Vec::new())
            .push(Rc::clone(record));

        if self.min_from.is_none() || start_time < self.min_from.unwrap() {
            self.min_from = Some(start_time);
        }

        if self.max_to.is_none() || end_time > self.max_to.unwrap() {
            self.max_to = Some(end_time);
        }

        // Update grid
        let start_pos = start_time.hour() as usize - self.start_hour() as usize;
        let end_pos = end_time.hour() as usize - self.start_hour() as usize;
        let width = end_pos - start_pos;

        let mut y = 0;
        while !self.test_record(start_pos, y, width) {
            y += 1;
        }

        // Mark grid positions as occupied
        for i in start_pos..end_pos {
            self.ensure_pos_exists(i, y);
            self.grid[y][i] = true;
        }
    }

    fn ensure_pos_exists(&mut self, x: usize, y: usize) {
        while self.grid.len() <= y {
            self.grid.push(Vec::new());
        }

        let row = &mut self.grid[y];
        while row.len() <= x {
            row.push(false);
        }
    }

    /// Check if there's space in the given
    fn test_record(&mut self, x: usize, y: usize, width: usize) -> bool {
        // Ensure buffers
        self.ensure_pos_exists(x, y);
        self.ensure_pos_exists(x + width, y);

        for i in x..(x + width) {
            if self.grid[y][i] {
                return false;
            }
        }

        return true;
    }

    pub fn period_count(&self) -> u32 {
        // 8->8:50 period
        // 9->9:50 period, etc

        if self.min_from.is_none() || self.max_to.is_none() {
            return 0;
        }

        self.max_to.unwrap().hour() - self.min_from.unwrap().hour() + 1
    }

    pub fn start_hour(&self) -> u32 {
        self.min_from.unwrap().hour()
    }

    pub fn height_in_periods(&self) -> usize {
        self.grid.len()
    }
}

impl Deref for CourseSpan {
    type Target = HashMap<NaiveTime, Vec<Rc<RefCell<CourseRecord>>>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for CourseSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
