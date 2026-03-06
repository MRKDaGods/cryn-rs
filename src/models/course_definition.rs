use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default)]
    pub struct CourseFlags: u8 {
        const None = 0;
        const MultipleLectures = 1 << 0;
        const MultipleTutorials = 1 << 1;

        /// For courses like gp, to clear disambiguation when importing
        const NonUniqueName = 1 << 2;
    }
}

impl std::fmt::Display for CourseFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::<String>::new();

        if self.contains(CourseFlags::MultipleLectures) {
            flags.push("Multiple Lectures".to_owned());
        }

        if self.contains(CourseFlags::MultipleTutorials) {
            flags.push("Multiple Tutorials".to_owned());
        }

        if self.contains(CourseFlags::NonUniqueName) {
            flags.push("NonUniqueName".to_owned());
        }

        if flags.is_empty() {
            flags.push("None".to_owned());
        }

        write!(f, "{}", flags.join(", "))
    }
}

#[derive(Debug, Default)]
pub struct CourseDefinition {
    pub code: String,
    pub name: String,
    pub flags: CourseFlags, // For ykyk ;) bas we're graduating 5alas :(
    pub selected: bool,
    pub lecture_count: u32,
    pub tutorial_count: u32,
}

impl CourseDefinition {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: code.to_owned(),
            name: name.to_owned(),
            ..Default::default()
        }
    }
}
