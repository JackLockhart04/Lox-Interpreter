use lox_interpreter::util::logger::{LogLevel, global_logger};
use lox_interpreter::parse::parser::Parser;
use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::interpret::interpreter::Interpreter;
// use std::env;


fn main() {
    let logger = global_logger();
    logger.set_level(LogLevel::Debug);
    logger.log(LogLevel::Debug, "Start");

    // We can now call public functions from the logger module using the module path (logger::).
    logger.log(LogLevel::Info, "Initializing configuration settings...");

    // Get command line arguments to see if input file is provided
    let args: Vec<String> = std::env::args().collect();
    let mut interpreter = Interpreter::new();

    if args.len() > 1 {
        let input_path = &args[1];
        logger.log(LogLevel::Info, format!("Input file provided: {}", input_path));
        match Scanner::new_from_file(input_path) {
            Ok(scanner) => {
                let mut parser = Parser::new(scanner);
                // the echoed source (like the REPL behavior).
                while !parser.is_at_end() {
                    match parser.parse() {
                        Some(stmt) => {
                            interpreter.interpret_stmt(&stmt);
                        }
                        None => {
                            if parser.had_error() {
                                parser.report_errors();
                                parser.clear_errors();
                            }
                            // continue parsing next declaration/statement
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                logger.log(LogLevel::Error, format!("Failed to open file {}: {}", input_path, e));
            }
        }
    } else {
        logger.log(LogLevel::Info, "No input file provided, starting REPL...");
        // Interactive REPL: parse and execute one statement per loop iteration.
        loop {
            // Create scanner reading from terminal (it will prompt for a line)
            let scanner = Scanner::new_from_terminal();
            let mut parser = Parser::new(scanner);

            // Parse a single statement (so the REPL doesn't try to read until EOF).
            match parser.parse() {
                Some(stmt) => {
                    interpreter.interpret_stmt(&stmt);
                }
                None => {
                    // Parsing failed for this input; show errors and continue the REPL
                    if parser.had_error() {
                        parser.report_errors();
                        parser.clear_errors();
                    }
                    continue;
                }
            }
        }
    }
}