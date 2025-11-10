use std::fmt::Display;
use std::io::{self, Write};
use std::sync::{LazyLock, Mutex}; // Import Mutex

// LogLevel needs to be Clone/Copy to be stored in the Mutex easily
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] 
pub enum LogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
}

pub struct Logger {
    // 1. Wrap the mutable state (min_level) in a Mutex
    min_level: Mutex<LogLevel>,
}

impl Logger {
    // Creates a new logger instance with the specified minimum log level.
    pub fn new(min_level: LogLevel) -> Self {
        println!("Logger initialized with minimum level: {:?}", min_level);
        Logger { 
            // 2. Initialize the Mutex
            min_level: Mutex::new(min_level) 
        }
    }

    // Sets the minimum log level for the logger.
    pub fn set_level(&self, new_level: LogLevel) {
        // Lock the Mutex to safely update the level
        let mut level = self.min_level.lock().unwrap();
        *level = new_level; // Update the value inside the Mutex
        println!("Logger level set to: {:?}", new_level);
    }

    // The core logging method.
    pub fn log<T: Display>(&self, level: LogLevel, message: T) {
        // Lock the Mutex to read the current minimum level
        let current_min_level = *self.min_level.lock().unwrap();

        // 3. Check against the locked minimum level
        if level <= current_min_level {
            // ... (rest of the logging logic remains the same)
            let output: &mut dyn Write = match level {
                LogLevel::Fatal | LogLevel::Error => &mut io::stderr(),
                _ => &mut io::stdout(),
            };

            let formatted_message = format!("[{:?}] - {}", level, message);

            if let Err(e) = writeln!(output, "{}", formatted_message) {
                eprintln!("Logger failed to write: {}", e);
            }
            if let Err(e) = output.flush() {
                eprintln!("Logger failed to flush: {}", e);
            }
        }
    }
}

// --- Global Singleton Setup (No change needed here) ---

// The static global logger instance
static GLOBAL_LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger::new(LogLevel::Info));

// Accessor function to get a reference to the global Logger.
pub fn global_logger() -> &'static Logger {
    &GLOBAL_LOGGER
}
