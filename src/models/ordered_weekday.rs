use chrono::Weekday;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderedWeekday(Weekday);

impl Ord for OrderedWeekday {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .days_since(Weekday::Sat)
            .cmp(&other.0.days_since(Weekday::Sat))
    }
}

impl PartialOrd for OrderedWeekday {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Weekday> for OrderedWeekday {
    fn from(value: Weekday) -> Self {
        Self(value)
    }
}

impl ToString for OrderedWeekday {
    fn to_string(&self) -> String {
        match self.0 {
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
        }
        .to_owned()
    }
}
