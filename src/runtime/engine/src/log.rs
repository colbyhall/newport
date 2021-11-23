use crate::*;
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

/// Level of verbosity in a log
pub enum Verbosity {
	Debug,
	Info,
	Warning,
	Error,
}

pub struct Entry {
	pub verbosity: Verbosity,
	pub category: Category,
	pub date: SystemDate,
	pub message: String,
}

#[derive(Copy, Clone)]
pub struct Category(&'static str);

impl Category {
	pub const fn new(name: &'static str) -> Self {
		Self(name)
	}
}

#[macro_export]
macro_rules! define_log_category {
	($category:ident, $name:ident) => {
		pub const $name: $crate::Category = $crate::Category::new(stringify!($category));
	};
}

define_log_category!(Temp, TEMP_CATEGORY);
define_log_category!(Engine, ENGINE_CATEGORY);

struct Inner {
	file: File,
	entries: Vec<Entry>,
}

/// Global structure that contains the current log file
pub struct Logger(Mutex<Inner>);

impl Logger {
	pub(crate) fn new() -> Self {
		if !Path::new(LOGS_PATH).exists() {
			create_dir(LOGS_PATH).unwrap();
		}

		let date = SystemDate::now();
		let mut path = PathBuf::new();
		path.push(LOGS_PATH);
		path.push(format!(
			"game_{:02}_{:02}_{}_{:02}_{:02}_{:02}.log",
			date.month, date.day_of_month, date.year, date.hour, date.minute, date.second
		));

		let file = File::create(&path).unwrap();

		let og_hook = panic::take_hook();

		panic::set_hook(Box::new(move |info| {
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

			(og_hook)(info);
			error!(
				ENGINE_CATEGORY,
				"thread '{}' panicked at '{}', {}", name, msg, location
			);
		}));

		Logger(Mutex::new(Inner {
			file,
			entries: Vec::with_capacity(4096),
		}))
	}
}

static LOGS_PATH: &str = "target/logs/";

pub fn log(verbosity: Verbosity, category: Category, message: String) {
	let logger: &Logger = Engine::logger();

	let date = SystemDate::now();

	let verb = match verbosity {
		Verbosity::Debug => " Debug ",
		Verbosity::Info => " Info  ",
		Verbosity::Warning => "Warning",
		Verbosity::Error => " Error ",
	};

	// Get verbosity as a &'static str
	let file_output = format!(
		"[{:02}/{:02}/{:03} | {:02}:{:02}:{:02}:{:02}] {} [{}] {}",
		date.month,
		date.day_of_month,
		date.year,
		date.hour,
		date.minute,
		date.second,
		date.milli,
		verb,
		category.0,
		&message,
	);

	let std_output = format!("{} [{}] {}", verb, category.0, &message,);

	let mut inner = logger.0.lock().unwrap();
	writeln!(inner.file, "{}", file_output).unwrap();

	match verbosity {
		Verbosity::Error => eprintln!("{}", std_output),
		_ => println!("{}", std_output),
	}

	inner.entries.push(Entry {
		verbosity,
		category,
		date,
		message,
	})
}

#[macro_export]
macro_rules! debug {
    ($category:ident, $($arg:tt)*) => (
        $crate::log($crate::Verbosity::Debug, $category, format!($($arg)*))
    );
	($($arg:tt)*) => (
        $crate::log($crate::Verbosity::Debug, $crate::TEMP_CATEGORY, format!($($arg)*))
    );
}

#[macro_export]
macro_rules! info {
    ($category:ident, $($arg:tt)*) => (
        $crate::log($crate::Verbosity::Info, $category, format!($($arg)*))
    );
	($($arg:tt)*) => (
        $crate::log($crate::Verbosity::Info, $crate::TEMP_CATEGORY, format!($($arg)*))
    );
}

#[macro_export]
macro_rules! warn {
    ($category:ident, $($arg:tt)*) => (
        $crate::log($crate::Verbosity::Warning, $category, format!($($arg)*))
    );
	($($arg:tt)*) => (
        $crate::log($crate::Verbosity::Warning, $crate::TEMP_CATEGORY, format!($($arg)*))
    );
}

#[macro_export]
macro_rules! error {
    ($category:ident, $($arg:tt)*) => (
        $crate::log($crate::Verbosity::Error, $category, format!($($arg)*))
    );
	($($arg:tt)*) => (
        $crate::log($crate::Verbosity::Error, $crate::TEMP_CATEGORY, format!($($arg)*))
    );
}
