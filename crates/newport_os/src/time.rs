use chrono::{Datelike, Local, Timelike, Weekday};

/// Date gathered from the OS
#[derive(Copy, Clone, PartialEq)]
pub struct SystemDate {
    pub year: i32,
    pub month: u32,
    pub day_of_week: Weekday,
    pub day_of_month: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub milli: u32,
}

impl SystemDate {
    pub fn now() -> Self {
        let local = Local::now();
        Self {
            year: local.year(),
            month: local.month(),
            day_of_week: local.weekday(),
            day_of_month: local.day(),
            hour: local.hour(),
            minute: local.minute(),
            second: local.second(),
            milli: local.timestamp_subsec_millis(),
        }
    }
}
