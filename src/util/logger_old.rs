use std::fmt::Display;
use std::io::{self, Write}; // Import Write for flushing

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    // Critical errors that halt the application
    Fatal,
    // Errors that prevent specific functionality but the app can continue
    Error,
    // Unexpected events or problems
    Warn,
    // General information about the app's state or actions
    Info,
    // Detailed information useful for debugging
    Debug,
}

pub struct Logger {
    min_level: LogLevel,
}

impl Logger {
    /// Creates a new logger instance with the specified minimum log level.
    pub fn new(min_level: LogLevel) -> Self {
        Logger { min_level }
    }

    /// The core logging method.
    pub fn log<T: Display>(&self, level: LogLevel, message: T) {
        // 1. Check if the message level meets the minimum required level
        if level <= self.min_level {
            // 2. Determine where to output the message
            let output: &mut dyn Write = match level {
                // Errors and Fatal messages should go to standard error (stderr)
                LogLevel::Fatal | LogLevel::Error => &mut io::stderr(),
                // All other levels go to standard output (stdout)
                _ => &mut io::stdout(),
            };

            // 3. Format and print the message
            let formatted_message = format!("[{:?}] - {}", level, message);

            // 4. Write the message and flush the output buffer
            if let Err(e) = writeln!(output, "{}", formatted_message) {
                // Basic error handling for the logger itself
                eprintln!("Logger failed to write: {}", e);
            }

            // Ensure the message is displayed immediately
            if let Err(e) = output.flush() {
                eprintln!("Logger failed to flush: {}", e);
            }
        }
    }
}