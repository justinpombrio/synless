use crate::util::SynlessBug;
use std::default::Default;
use std::fmt;
use std::fs;
use std::sync::{Mutex, MutexGuard, OnceLock};

const LOG_PATH: &str = "log.txt";

static LOG: OnceLock<Mutex<Log>> = OnceLock::new();

pub use crate::log;

pub struct Log {
    entries: Vec<LogEntry>,
    log_file: fs::File,
}

// TODO: time stamps
pub struct LogEntry {
    level: LogLevel,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Log {
    fn new() -> Log {
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_PATH)
            .bug_msg("Failed to open log file for writing");
        Log {
            entries: Vec::new(),
            log_file,
        }
    }

    #[doc(hidden)]
    pub fn with_log<R>(mut callback: impl FnOnce(&mut Log) -> R) -> R {
        let log_mutex: &'static Mutex<Log> = LOG.get_or_init(|| Mutex::new(Log::new()));
        let mut log_guard: MutexGuard<Log> = log_mutex.lock().bug();
        callback(&mut log_guard)
    }

    #[doc(hidden)]
    pub fn push(&mut self, entry: LogEntry) {
        use std::io::Write;

        let _ = writeln!(self.log_file, "{}", entry);
        self.entries.push(entry);
    }

    pub fn to_string() -> String {
        Log::with_log(|log| log.to_string())
    }
}

impl Default for Log {
    fn default() -> Log {
        Log::new()
    }
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> LogEntry {
        LogEntry { level, message }
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level = format!("[{}]", self.level);
        let msg = &self.message;
        write!(f, "{level:<7} {msg}")
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{}", entry)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($level:ident, $message:literal) => {
        $crate::log!($level, $message,)
    };
    ($level:ident, $message:literal, $( $arg:expr ),*) => {
        {
            let level = $crate::LogLevel::$level;
            let message = format!($message, $( $arg ),*);
            let entry = $crate::LogEntry::new(level, message);
            $crate::Log::with_log(|log| log.push(entry));
        }
    };
}
