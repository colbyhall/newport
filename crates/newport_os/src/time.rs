#[cfg(target_os = "windows")]
use crate::win32::*;

/// Date gathered from the OS
#[derive(Copy, Clone, PartialEq)]
pub struct SystemDate {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day_of_month: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milli: u16,
}

impl SystemDate {
    /// Returns a `SystemDate` at the local time
    pub fn now() -> Self {
        let mut system_time = SYSTEMTIME::default();
        unsafe { GetLocalTime(&mut system_time) };
        Self {
            year: system_time.wYear,
            month: system_time.wMonth,
            day_of_week: system_time.wDayOfWeek,
            day_of_month: system_time.wDay,
            hour: system_time.wHour,
            minute: system_time.wMinute,
            second: system_time.wSecond,
            milli: system_time.wMilliseconds,
        }
    }
}