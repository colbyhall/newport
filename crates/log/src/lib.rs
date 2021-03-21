use std::sync::Mutex;
use std::fs::{
    File,
    create_dir,
};
use std::io::Write;
use std::path::{ Path, PathBuf };
use os::time::SystemDate;

/// Global structure that contains the current log file
pub struct Logger {
    file: File,
}

/// Type to easily create new categories
/// 
/// # Examples
/// 
/// ```
/// static FOO_CAT: log::Category = "Foo"
/// log_info!(FOO_CAT, "Hello, world!");
/// ```
pub type Category = &'static str;

/// Level of verbosity in a log
pub enum Verbosity {
    Debug,
    Info,
    Warning,
    Error,
}

static LOGS_PATH : &str = "logs/";
static mut LOGGER : Option<Mutex<Logger>> = None;

impl Logger {
    /// Initializes global logger for usage. Logger must be initialized to log
    pub fn init() {
        if !Path::new(LOGS_PATH).exists() {
            create_dir(LOGS_PATH).unwrap();
        }

        let date = SystemDate::now();
        let mut path = PathBuf::new();
        path.push(LOGS_PATH);
        path.push(
            format!(
                "game_{:02}_{:02}_{}_{:02}_{:02}.log", 
                date.month,
                date.day_of_month,
                date.year, 
                date.hour,
                date.minute
            )
        );

        let file = File::create(path);
        if file.is_err() { return; }
        let file = file.unwrap();
        unsafe { LOGGER = Some(Mutex::new(Logger { file })); }
    }

    pub fn log(cat: Category, verb: Verbosity, message: &str) {
        // Get verbosity as a &'static str
        let output = {
            let verb = match verb {
                Verbosity::Debug => "Debug",
                Verbosity::Info => "Info",
                Verbosity::Warning => "Warning",
                Verbosity::Error   => " Error ",
            };
            
            // @NOTE(colby): Build output
            let date = SystemDate::now();
            format!(
                "[{:02}/{:02}/{} | {:02}:{:02}:{:02}:{:02}] {} [{}] {}",
                date.month,
                date.day_of_month,
                date.year,
                date.hour,
                date.minute, 
                date.second,
                date.milli,
                verb,
                cat,
                message,
            )
        };

        unsafe {
            let logger = LOGGER.as_ref().unwrap();
            let logger = logger.lock().unwrap();
            writeln!(&logger.file, "{}", output).unwrap();
            println!("{}", output);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($message:expr) => (
        Logger::log("Temp", Verbosity::Info, $message)
    );
    ($cat:expr, $($arg:tt)*) => (
        Logger::log($cat, Verbosity::Info, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! log_debug {
    ($message:expr) => (
        Logger::log("Temp", Verbosity::Debug, $message)
    );
    ($cat:expr, $($arg:tt)*) => (
        Logger::log($cat, Verbosity::Debug, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! log_info {
    ($message:expr) => (
        Logger::log("Temp", Verbosity::Info, $message)
    );
    ($cat:expr, $($arg:tt)*) => (
        Logger::log($cat, Verbosity::Info, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! log_warning {
    ($message:expr) => (
        Logger::log("Temp", Verbosity::Warning, $message)
    );
    ($cat:expr, $($arg:tt)*) => (
        Logger::log($cat, Verbosity::Warning, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! log_error {
    ($message:expr) => (
        Logger::log("Temp", Verbosity::Error, $message)
    );
    ($cat:expr, $($arg:tt)*) => (
        Logger::log($cat, Verbosity::Error, &format!($($arg)*))
    );
}