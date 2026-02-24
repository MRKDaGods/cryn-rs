use super::CourseDefinition;
use chrono::{NaiveTime, Timelike, Weekday};
use std::{cell::RefCell, fmt::Display, rc::Rc};
use strum::EnumString;

#[derive(Debug, EnumString, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum CourseRecordType {
    None,
    Lecture,
    Tutorial,
}

impl CourseRecordType {
    pub fn short_name(&self) -> &str {
        match self {
            CourseRecordType::None => "UNK",
            CourseRecordType::Lecture => "LEC",
            CourseRecordType::Tutorial => "TUT",
        }
    }

    pub fn long_name(&self) -> &str {
        match self {
            CourseRecordType::None => "Unknown",
            CourseRecordType::Lecture => "Lecture",
            CourseRecordType::Tutorial => "Tutorial",
        }
    }
}

impl Display for CourseRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.long_name())
    }
}

/// Backwards compatibility
#[derive(Debug)]
pub enum CourseParseFormat {
    Standard,
    IrregularWithoutNameGroupPrefixed,
    IrregularWithoutNameGroupPostFixed,
    IrregularWithoutName,
    IrregularWithNameNoGroup,
    IrregularWithName,
    Excel,
}

#[derive(Debug)]
pub struct CourseRecord {
    pub course_definition: Rc<RefCell<CourseDefinition>>,
    pub group: i32,
    pub record_type: CourseRecordType,
    pub day: Weekday,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub class_size: i32,
    pub enrolled: i32,
    pub waiting: i32,
    pub status: String,
    pub location: String,
    pub parse_format: CourseParseFormat,

    // Mullec/Multut flags indices
    pub mullec_index: i32,
    pub multut_index: i32,
}

impl CourseRecord {
    pub fn new(
        course_definition: Rc<RefCell<CourseDefinition>>,
        group: i32,
        record_type: CourseRecordType,
        day: Weekday,
        start_time: NaiveTime,
        end_time: NaiveTime,
        class_size: i32,
        enrolled: i32,
        waiting: i32,
        status: String,
        location: String,
        parse_format: CourseParseFormat,
    ) -> Self {
        Self {
            course_definition,
            group,
            record_type,
            day,
            start_time,
            end_time,
            class_size,
            enrolled,
            waiting,
            status,
            location,
            parse_format,
            mullec_index: -1,
            multut_index: -1,
        }
    }

    pub fn periods(&self) -> u32 {
        self.end_time.hour() - self.start_time.hour() + 1
    }

    pub fn is_closed(&self) -> bool {
        self.status.to_lowercase() != "opened"
    }
}
