use lox_interpreter::util::logger::{LogLevel, global_logger};
use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::token::token::{Token, TokenType};
use lox_interpreter::interpret::expr::{Expr, BinaryExpr, UnaryExpr, GroupingExpr, LiteralExpr, LiteralValue};
use lox_interpreter::util::ast_printer::AstPrinter;
use std::env;


fn main() {
    let logger = global_logger();
    logger.set_level(LogLevel::Debug);
    logger.log(LogLevel::Debug, "Start");

    // We can now call public functions from the logger module using the module path (logger::).
    logger.log(LogLevel::Info, "Initializing configuration settings...");

    // Test ast printer
    let left = Expr::Unary(UnaryExpr {
        operator: Token::new_token(TokenType::Minus, "-".to_string(), None, 1),
        right: Box::new(Expr::Literal(LiteralExpr { value: Some(LiteralValue::Number(123.0)) })),
    });

    let right = Expr::Grouping(GroupingExpr {
        expression: Box::new(Expr::Literal(LiteralExpr { value: Some(LiteralValue::Number(45.67)) })),
    });

    let expr = Expr::Binary(BinaryExpr {
        left: Box::new(left),
        operator: Token::new_token(TokenType::Star, "*".to_string(), None, 1),
        right: Box::new(right),
    });

    let mut printer = AstPrinter;
    let out = printer.print(&expr);
    println!("{}", out);

    // // Example usage of the Scanner
    // let args: Vec<String> = env::args().collect();
    // let is_file = args.len() > 1;
    // let mut scanner = if is_file {
    //     match Scanner::new_from_file(&args[1]) {
    //         Ok(s) => s,
    //         Err(e) => {
    //             logger.log(LogLevel::Error, format!("Failed to read file: {}", e));
    //             return;
    //         }
    //     }
    // } else {
    //     Scanner::new_from_terminal()
    // };

    // // Unified loop: only call next_token(); scanner handles buffering and I/O.
    // loop {
    //     match scanner.next_token() {
    //         Some(token) => {
    //             match token.get_type() {
    //                 TokenType::Eof => {
    //                     logger.log(LogLevel::Debug, "Reached end of file");
    //                     break;
    //                 }
    //                 _ => {
    //                     logger.log(LogLevel::Debug, format!("Read token: '{:?}'", token));
    //                 }
    //             }
    //         }
    //         None => break,
    //     }
    //     // Print current line number for debugging
    //     let line_num = scanner.get_line_number();
    //     logger.log(LogLevel::Debug, format!("Current line number: {}", line_num));
    // }
}
