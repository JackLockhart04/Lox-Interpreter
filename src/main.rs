use lox_interpreter::util::logger::{LogLevel, global_logger};
use lox_interpreter::parse::parser::Parser;
use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::interpret::interpreter::Interpreter;

fn main() {
    let logger = global_logger();
    logger.set_level(LogLevel::Debug);
    global_logger().log(LogLevel::Debug, "main: Start");

    global_logger().log(LogLevel::Info, "main: Initializing configuration settings...");

    // Choose a scanner based on whether a filename was provided.
    let args: Vec<String> = std::env::args().collect();
    let mut interpreter = Interpreter::new();

    let scanner = if args.len() > 1 {
        let input_path = &args[1];
    global_logger().log(LogLevel::Info, format!("main: Input file provided: {}", input_path));
        match Scanner::new_from_file(input_path) {
            Ok(s) => s,
            Err(e) => {
                global_logger().log(LogLevel::Error, format!("main: Failed to open file {}: {}", input_path, e));
                return;
            }
        }
    } else {
    global_logger().log(LogLevel::Info, "main: No input file provided, starting REPL...");
        Scanner::new_from_terminal()
    };

    let mut parser = Parser::new(scanner);

    // Flat incremental loop: parse one declaration/statement and execute it
    // immediately. This keeps control flow simple and identical for file
    // and terminal input.
    while !parser.is_at_end() {
        match parser.parse() {
            Some(stmt) => interpreter.interpret_stmt(&stmt),
            None => {
                if parser.had_error() {
                    parser.report_errors();
                    parser.clear_errors();
                }
            }
        }
    }
}