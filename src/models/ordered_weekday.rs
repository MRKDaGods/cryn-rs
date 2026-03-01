use std::fmt::Display;

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

impl Display for OrderedWeekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
        };

        write!(f, "{}", s)
    }
}
