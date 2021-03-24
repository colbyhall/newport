use newport_os::time::SystemDate;
use newport_engine::*;

use std::any::Any;
use std::sync::Mutex;
use std::fs::{
    File,
    create_dir,
};
use std::path::{ Path, PathBuf };
use std::io::Write;
use std::ptr::null_mut;

/// Global structure that contains the current log file
pub struct Logger {
    file: Mutex<File>,
}

impl ModuleCompileTime for Logger {
    fn new() -> Result<Self, String> {
        if !Path::new(LOGS_PATH).exists() {
            let result = create_dir(LOGS_PATH);
            if result.is_err() {
                let err = result.err().unwrap();
                let err = format!("Failed to create directory at \"{}\" due to {:?}", LOGS_PATH, err.kind());
                return Err(err);
            }
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

        let file = File::create(&path);
        if file.is_err() { 
            let err = file.err().unwrap();
            let err = format!("Failed to create logger file at {:?} due to {:?}", path, err.kind());
            return Err(err);
         }
        let file = file.unwrap();
        
        return Ok(Logger { file: Mutex::new(file) });
    }
}

impl ModuleRuntime for Logger {
    fn post_init(&mut self, _: &mut Engine) {
        unsafe { LOGGER = self };
    }

    fn as_any(&self) -> &dyn Any { self }
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

static LOGS_PATH:   &str = "logs/";
static mut LOGGER:  *mut Logger = null_mut();

impl Logger {
    pub fn set_global(&mut self) {
        unsafe{ LOGGER = self };
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
            if LOGGER == null_mut() {
                println!("{}", output); // UNSAFE: Not a thread safe print
            } else {
                let logger = LOGGER.as_ref().unwrap();
                let mut file = logger.file.lock().unwrap();
                writeln!(file, "{}", output).unwrap();
                println!("{}", output);
            }
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