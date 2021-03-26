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
    fn as_any(&self) -> &dyn Any{ self }

    fn post_init(&mut self, _: &mut Engine) {
        unsafe { LOGGER = self };
    }
}

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

    pub fn log(verb: Verbosity, message: &str) {
        // Get verbosity as a &'static str
        let output = {
            let verb = match verb {
                Verbosity::Debug => "Debug",
                Verbosity::Info => "Info",
                Verbosity::Warning => "Warning",
                Verbosity::Error   => " Error ",
            };
            
            // Build output
            let date = SystemDate::now();
            format!(
                "{} [{:02}/{:02}/{} | {:02}:{:02}:{:02}:{:02}] {}",
                verb,
                date.month,
                date.day_of_month,
                date.year,
                date.hour,
                date.minute, 
                date.second,
                date.milli,
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
macro_rules! debug {
    ($($arg:tt)*) => (
        Logger::log(Verbosity::Debug, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        Logger::log(Verbosity::Info, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        Logger::log(Verbosity::Warning, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        Logger::log(Verbosity::Error, &format!($($arg)*))
    );
}