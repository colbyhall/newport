use engine::*;
use platform::time::SystemDate;

use std::fs::{
	create_dir,
	File,
};
use std::panic;
use std::path::{
	Path,
	PathBuf,
};

use std::io::Write;
use std::sync::Mutex;

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
		path.push(format!(
			"game_{:02}_{:02}_{}_{:02}_{:02}.log",
			date.month, date.day_of_month, date.year, date.hour, date.minute
		));

		let file = File::create(&path).unwrap();

		panic::set_hook(Box::new(|info| {
			// The current implementation always returns `Some`.
			let location = info.location().unwrap();

			let msg = match info.payload().downcast_ref::<&'static str>() {
				Some(s) => *s,
				None => match info.payload().downcast_ref::<String>() {
					Some(s) => &s[..],
					None => "Box<Any>",
				},
			};
			let thread = std::thread::current();
			let name = thread.name().unwrap_or("<unnamed>");

			error!("thread '{}' panicked at '{}', {}", name, msg, location);
			// error!("{:?}", std::backtrace::capture());
		}));

		Logger {
			file: Mutex::new(file),
		}
	}
}

/// Level of verbosity in a log
pub enum Verbosity {
	Debug,
	Info,
	Warning,
	Error,
}

static LOGS_PATH: &str = "logs/";

impl Logger {
	pub fn log(verb: Verbosity, message: &str) {
		let logger: &Logger = Engine::module().unwrap();

		// Get verbosity as a &'static str
		let output = {
			let verb = match verb {
				Verbosity::Debug => "Debug",
				Verbosity::Info => "Info",
				Verbosity::Warning => "Warning",
				Verbosity::Error => " Error ",
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

		let mut file = logger.file.lock().unwrap();
		writeln!(file, "{}", output).unwrap();
		println!("{}", output)
	}
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        $crate::Logger::log($crate::Verbosity::Debug, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        $crate::Logger::log($crate::Verbosity::Info, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        $crate::Logger::log($crate::Verbosity::Warning, &format!($($arg)*))
    );
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        $crate::Logger::log($crate::Verbosity::Error, &format!($($arg)*))
    );
}
