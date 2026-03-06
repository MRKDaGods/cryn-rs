use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use chrono::{NaiveTime, Timelike};

use crate::models::CourseRecord;

#[derive(Default)]
pub struct CourseSpan {
    map: HashMap<(usize, usize), Rc<RefCell<CourseRecord>>>,
    records: Vec<Rc<RefCell<CourseRecord>>>,
    min_from: Option<NaiveTime>,
    max_to: Option<NaiveTime>,
    grid: Vec<Vec<bool>>,
    dirty: bool,
}

impl CourseSpan {
    pub fn insert(&mut self, record: &Rc<RefCell<CourseRecord>>) {
        let borrowed = record.borrow();

        if self.min_from.is_none() || borrowed.start_time < self.min_from.unwrap() {
            self.min_from = Some(borrowed.start_time);
        }

        if self.max_to.is_none() || borrowed.end_time > self.max_to.unwrap() {
            self.max_to = Some(borrowed.end_time);
        }

        drop(borrowed);
        self.records.push(Rc::clone(record));
        self.dirty = true;
    }

    /// Rebuild grid
    pub fn rebuild(&mut self) {
        if !self.dirty {
            return;
        }

        self.grid.clear();
        self.map.clear();

        let start_hour = self.start_hour();

        // Sort by start time
        self.records.sort_by_key(|r| r.borrow().start_time);
        let records = self.records.to_vec(); // Clone so we can borrow later

        for record in records {
            let borrowed = record.borrow();

            let start_pos = (borrowed.start_time.hour() - start_hour) as usize;
            let end_pos = (borrowed.end_time.hour() - start_hour) as usize;
            let width = end_pos - start_pos + 1;
            drop(borrowed);

            // Find first available row
            let mut y = 0;
            while !self.test_record(start_pos, y, width) {
                y += 1;
            }

            // Mark occupied
            for i in start_pos..=end_pos {
                self.ensure_pos_exists(i, y);
                self.grid[y][i] = true;
            }

            self.map.insert((start_pos, y), Rc::clone(&record));
        }

        self.dirty = false;
    }

    fn ensure_pos_exists(&mut self, x: usize, y: usize) {
        while self.grid.len() <= y {
            self.grid.push(Vec::new());
        }
        while self.grid[y].len() <= x {
            self.grid[y].push(false);
        }
    }

    fn test_record(&mut self, x: usize, y: usize, width: usize) -> bool {
        self.ensure_pos_exists(x + width - 1, y);
        for i in x..(x + width) {
            if self.grid[y][i] {
                return false;
            }
        }
        true
    }

    pub fn period_count(&self) -> u32 {
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
    type Target = HashMap<(usize, usize), Rc<RefCell<CourseRecord>>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for CourseSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
