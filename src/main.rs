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
                // Parse expressions until EOF. If parse() returns None and the parser
                // has reached EOF, stop; otherwise report the error and continue.
                loop {
                    match parser.parse() {
                        Some(expr) => {
                                    match interpreter.interpret(&expr) {
                                        Ok(res) => println!("=> {:?}", res),
                                        Err(e) => {
                                            logger.log(LogLevel::Error, format!("[line {}] Runtime error at '{}': {}", e.token.line, e.token.lexeme, e.message));
                                        }
                                    }
                        }
                        None => {
                            if parser.is_at_end() {
                                break;
                            }
                            parser.report_errors();
                            parser.clear_errors();
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

        // Interactive REPL: create a Scanner + Parser each loop and interpret the parsed expression.
        loop {
            // Create scanner reading from terminal (it will prompt for a line)
            let scanner = Scanner::new_from_terminal();
            let mut parser = Parser::new(scanner);

            match parser.parse() {
                Some(expr) => {
                    match interpreter.interpret(&expr) {
                        Ok(res) => println!("=> {:?}", res),
                        Err(e) => {
                            logger.log(LogLevel::Error, format!("[line {}] Runtime error at '{}': {}", e.token.line, e.token.lexeme, e.message));
                        }
                    }
                }
                None => {
                    // Parsing failed; print errors (if any) and continue the REPL
                    parser.report_errors();
                    parser.clear_errors();
                }
            }
        }
    }
}