use newport_os::time::SystemDate;
use newport_engine::*;

use std::sync::Mutex;
use std::fs::{ File, create_dir, };
use std::path::{ Path, PathBuf };
use std::io::Write;
use std::ptr::null_mut;

/// Global structure that contains the current log file
pub struct Logger {
    file: Mutex<File>,
}

impl Module for Logger {
    fn new() -> Self {
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

        let file = File::create(&path).unwrap();
        
        return Logger { file: Mutex::new(file) };
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder.post_init(|engine: &Engine| {
            let logger = engine.module::<Logger>().unwrap();

            // UNSAFE: Set the global ptr
            unsafe { LOGGER = logger as *const Logger as *mut Logger };
        })
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