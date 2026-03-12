use std::rc::Rc;
use std::sync::LazyLock;

use regex::Regex;

use crate::models::CourseSummary;
use crate::services::CourseManager;

const SUMMARY_REGEX_PATTERN: &str = r#"(?x)
    [^\[\w\d]*                  # Skip non-alphanumeric characters
    (?:\[(\w{4}\d{3})\]\s*)?    # 1: Optional course code
    ([^(]+)                     # 2: Course name
    \(
    (NA|\d{1,2})                # 3: Lecture group
    /
    (NA|\d{1,2})                # 4: Tutorial group
    \)
"#;

static SUMMARY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(SUMMARY_REGEX_PATTERN).unwrap());

pub fn parse(
    course_manager: &CourseManager,
    data: &str,
    errors: &mut Vec<String>,
) -> Vec<CourseSummary> {
    let mut summaries = Vec::new();

    for c in SUMMARY_REGEX.captures_iter(data) {
        let mut code = None;
        if let Some(c) = c.get(1) {
            code = Some(c.as_str().to_string());
        }

        let name = c.get(2).unwrap().as_str().trim();

        // Parse groups
        let parse_group = |g: Option<regex::Match>| -> Option<i32> {
            g.map(|m| m.as_str().parse::<i32>().ok()).unwrap_or(None)
        };

        let lec = parse_group(c.get(3));
        let tut = parse_group(c.get(4));

        // Resolve definition
        let mut skip_error = false;
        let definition = match code {
            // If we have a code, get def directly
            Some(code) => course_manager.get_definition_by_code(&code),

            // If no code, try find by name
            None => {
                let defs = course_manager.get_definitions_by_name(name);

                // If multiple defs, return None
                if defs.len() > 1 {
                    errors.push(format!("Multiple definitions found for course {}", name));
                    skip_error = true;
                    None
                } else {
                    defs.into_iter().next()
                }
            }
        };

        if let Some(definition) = definition {
            let def = definition.borrow();
            summaries.push(CourseSummary {
                code: def.code.clone(),
                name: def.name.clone(),
                definition: Rc::as_ptr(&definition),
                selected_lec: lec,
                selected_tut: tut,
                ..Default::default()
            });
        } else if !skip_error {
            errors.push(format!("No definition found for course {}", name));
        }
    }

    summaries
}
