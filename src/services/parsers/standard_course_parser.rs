use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use chrono::{Duration, NaiveTime, Timelike, Weekday};
use regex::Regex;

use crate::models::{
    CourseDefinition, CourseFlags, CourseParseFormat, CourseRecord, CourseRecordType,
};
use crate::services::CourseManager;

const MIN_HOUR: u32 = 8;

const COURSE_RECORD_REGEX: &str = r#"(?x)
    <td>__([^_]+)__(?:[^<]*)<\/td>   # 1: Code
    <td>([^<]*)<\/td>                # 2: Name
    <td>([^<]*)<\/td>                # 3: Group
    <td>([^<]*)<\/td>                # 4: Type
    <td>([^<]*)<\/td>                # 5: Day
    <td>([^<]*)<\/td>                # 6: From
    <td>([^<]*)<\/td>                # 7: To
    <td>([^<]*)<\/td>                # 8: Class size
    <td>([^<]*)<\/td>                # 9: Enrolled
    <td>([^<]*)<\/td>                # 10: Waiting
    <td>([^<]*)<\/td>                # 11: Status
    <td>([^<]*)<\/td>                # 12: Location
"#;

pub fn parse(course_manager: &mut CourseManager, data: &str) {
    // Clear existing data
    course_manager.course_records.clear();
    course_manager.course_definitions.clear();

    // Track non unique names
    let mut name_map = HashMap::<String, Vec<Rc<RefCell<CourseDefinition>>>>::new();

    // Parse new data
    let re = Regex::new(COURSE_RECORD_REGEX).unwrap();
    for c in re.captures_iter(data) {
        let mut code = get_capture_value(&c, 1);

        // Fixup name
        let name_fixup = get_capture_value(&c, 2).replace("&amp;", "&");
        let mut name = name_fixup.as_str();

        // Parse group with potential irregular format
        // Auto fixes code and name if needed
        let group_str = get_capture_value(&c, 3);
        let (group, parse_format) = parse_group(&mut code, &mut name, group_str);

        // Everything else
        let record_type: CourseRecordType = parse_direct(&c, 4);
        let day: Weekday = parse_direct(&c, 5);
        let mut from: NaiveTime = parse_direct(&c, 6);
        let mut to: NaiveTime = parse_direct(&c, 7);
        let class_size: i32 = parse_direct(&c, 8);
        let enrolled: i32 = parse_direct(&c, 9);
        let waiting: i32 = parse_direct(&c, 10);
        let status = sanitize_str(get_capture_value(&c, 11));
        let location = sanitize_str(get_capture_value(&c, 12));

        // Validate timespans
        fix_timespan(&mut from, None);
        fix_timespan(&mut to, Some(&from));

        // Get course def and register record
        let (course_definition_rc, has_created) =
            course_manager.get_or_add_course_definition(code, name);

        // Track unique names
        if has_created {
            let same_name_courses = name_map.entry(name_fixup.clone()).or_default();
            same_name_courses.push(Rc::clone(&course_definition_rc));
        }

        let mut course_definition = course_definition_rc.borrow_mut();

        // Update course stats
        match record_type {
            CourseRecordType::Lecture => course_definition.lecture_count += 1,
            CourseRecordType::Tutorial => course_definition.tutorial_count += 1,
            CourseRecordType::None => {
                panic!("Invalid course type {:?}", course_definition_rc.borrow())
            }
        }

        let record_rc = Rc::new(RefCell::new(CourseRecord {
            course_definition: Rc::clone(&course_definition_rc),
            group,
            record_type,
            day,
            start_time: from,
            end_time: to,
            class_size,
            enrolled,
            waiting,
            status,
            location,
            parse_format,
            ..Default::default()
        }));
        course_manager.course_records.push(record_rc);
    }

    // Sort courses and apply flags
    post_process_courses(course_manager, name_map);
}

fn post_process_courses(
    course_manager: &mut CourseManager,
    name_map: HashMap<String, Vec<Rc<RefCell<CourseDefinition>>>>,
) {
    // Sort course definitions by code
    course_manager
        .course_definitions
        .sort_by(|a, b| a.borrow().code.cmp(&b.borrow().code));

    // Sort records by day then time
    course_manager.course_records.sort_by_key(|record| {
        (
            record.borrow().day.days_since(Weekday::Sat),
            record.borrow().start_time,
        )
    });

    // Flags
    course_manager.course_definitions.iter().for_each(|def_rc| {
        let mut lecture_group_map = HashMap::<i32, i32>::new();
        let mut tutorial_group_map = HashMap::<i32, i32>::new();

        // How many lecs/tuts for each group?
        let mut count_groups = |record: &mut CourseRecord, record_type: CourseRecordType| {
            if record.record_type != record_type {
                return;
            }

            let group_map: &mut HashMap<i32, i32>;
            let mul_index: &mut i32;

            match record_type {
                CourseRecordType::Lecture => {
                    group_map = &mut lecture_group_map;
                    mul_index = &mut record.mullec_index;
                }

                CourseRecordType::Tutorial => {
                    group_map = &mut tutorial_group_map;
                    mul_index = &mut record.multut_index;
                }

                CourseRecordType::None => return,
            }

            // Increment
            let count = group_map.entry(record.group).or_insert(0);
            *count += 1;
            *mul_index = *count;
        };

        // Start counting
        course_manager
            .course_records
            .iter_mut()
            .filter(|record| {
                record.borrow().course_definition.borrow().code == def_rc.borrow().code
            })
            .for_each(|record| {
                let record = &mut *record.borrow_mut();
                count_groups(record, CourseRecordType::Lecture);
                count_groups(record, CourseRecordType::Tutorial);
            });

        // Check for MultipleLectures
        if lecture_group_map.iter().any(|(_, count)| *count > 1) {
            def_rc.borrow_mut().flags |= CourseFlags::MultipleLectures;
        }

        // Check for MultipleTutorials
        if tutorial_group_map.iter().any(|(_, count)| *count > 1) {
            def_rc.borrow_mut().flags |= CourseFlags::MultipleTutorials;
        }
    });

    // Name uniqueness
    for (_, defs) in name_map {
        // Check if more than one course has the same name
        if defs.len() > 1 {
            defs.iter()
                .for_each(|def_rc| def_rc.borrow_mut().flags |= CourseFlags::NonUniqueName);
        }
    }
}

fn fix_timespan(timespan: &mut NaiveTime, relv_time: Option<&NaiveTime>) {
    if timespan.hour() < MIN_HOUR || relv_time.is_some_and(|t| *t > *timespan) {
        *timespan += Duration::hours(12);
    }
}

fn parse_group<'a>(
    code: &mut &'a str,
    name: &mut &'a str,
    group_str: &'a str,
) -> (i32, CourseParseFormat) {
    let group: i32;
    let mut parse_format: CourseParseFormat;

    // Irregular format detection
    let has_irregular_format =
        *code == "LECS000" || *code == "TUTS000" || group_str.contains(*code);
    if has_irregular_format {
        // I forgot how this used to work so im copying the c# impl lmao

        // Determine format
        // x-yyyy
        let sep = group_str.find('-').unwrap_or(usize::MAX);
        if group_str.len() < 9 || sep == usize::MAX {
            // Assuming group < 10
            panic!("Irregular format is invalid");
        }

        // Which format?
        //	1-MTHS002
        //  MDPS478-Vehicle System Dynamics and Control- 3

        // Okay
        // Find next
        if let Some(sep2) = group_str[sep + 1..].rfind('-') {
            parse_format = CourseParseFormat::IrregularWithName;
            *code = &group_str[..sep];
            *name = &group_str[(sep + 1)..sep2];
            group = group_str[(sep + sep2)..].parse::<i32>().unwrap();
        } else {
            //	5-MTHS003
            //	INTS203-G.1
            match group_str[..sep].parse::<i32>() {
                Ok(potential_group) => {
                    parse_format = CourseParseFormat::IrregularWithoutName;
                    group = potential_group;
                    *code = &group_str[(sep + 1)..];

                    //	5-5MTHS003
                    if code.chars().next().unwrap().is_ascii_digit() {
                        parse_format = CourseParseFormat::IrregularWithoutNameGroupPrefixed;
                        *code = &(*code)[1..];
                    }
                }
                Err(_) => {
                    //	INTS203-G.1
                    parse_format = CourseParseFormat::IrregularWithoutNameGroupPostFixed;
                    *code = &group_str[..sep];

                    // Crazy
                    let group_part = group_str[..(sep + 1)].replace("G.", "");
                    match group_part.parse::<i32>() {
                        Ok(g) => group = g,
                        Err(_) => {
                            parse_format = CourseParseFormat::IrregularWithNameNoGroup;
                            group = -1;
                            *name = &group_str[(sep + 1)..];
                        }
                    }
                }
            }
        }
    } else {
        group = group_str.parse::<i32>().unwrap();
        parse_format = CourseParseFormat::Standard;
    }

    (group, parse_format)
}

fn sanitize_str(data: &str) -> String {
    data.trim().replace("_", "")
}

fn get_capture_value<'a>(c: &'a regex::Captures<'a>, idx: usize) -> &'a str {
    c.get(idx).unwrap().as_str().trim()
}

fn parse_direct<T>(c: &regex::Captures, idx: usize) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    sanitize_str(get_capture_value(c, idx))
        .parse::<T>()
        .unwrap()
}
