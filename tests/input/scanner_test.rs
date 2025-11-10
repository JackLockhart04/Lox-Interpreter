use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::token::token::TokenType;

use std::fs;
use tempfile;

type TestResult = Result<(), String>;

// Helper to write content to temp file and return its path string
fn write_temp_file(temp_dir: &tempfile::TempDir, filename: &str, content: &str) -> Result<String, String> {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).map_err(|e| format!("File Write Error: {}", e))?;
    Ok(file_path.to_str().ok_or_else(|| "Invalid Path".to_string())?.to_string())
}

#[test]
fn scanner_basic_tokens() -> TestResult {
    // Setup: input exercising comments, keywords, identifiers, numbers, strings,
    // two-character tokens, and single-char tokens.
    let content = "// a line comment\nvar a = 123;\nfun f() { return a <= 10 != false; }\n\"hello\"\n==\n";

    // Expected token sequence (excluding comments and whitespace)
    let expected_token_types = vec![
        TokenType::Var,
        TokenType::Identifier,
        TokenType::Equal,
        TokenType::Number,
        TokenType::Semicolon,

        TokenType::Fun,
        TokenType::Identifier,
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier,
        TokenType::LessEqual,
        TokenType::Number,
        TokenType::BangEqual,
        TokenType::False,
        TokenType::Semicolon,
        TokenType::RightBrace,

        TokenType::String,

        TokenType::EqualEqual,
    ];

    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "test_input.txt", content)?;
    let mut scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader initialization failed: {}", e))?;

    for (idx, expected) in expected_token_types.iter().enumerate() {
        let _peek = scanner.peek_token();
        let tok = scanner.next_token().ok_or_else(|| format!("[FAIL] Expected token {:?} but got None", expected))?;
        if tok.get_type() != *expected {
            return Err(format!("[FAIL] Expected {:?} got {:?} at index {}", expected, tok.get_type(), idx));
        }

        // Spot-check lexemes/literals for a few types
        match tok.get_type() {
            TokenType::Identifier => {
                if tok.lexeme != "a" && tok.lexeme != "f" {
                    return Err(format!("[FAIL] Identifier lexeme unexpected: {}", tok.lexeme));
                }
            }
            TokenType::Number => {
                if tok.lexeme != "123" && tok.lexeme != "10" { return Err(format!("[FAIL] Number lexeme unexpected: {}", tok.lexeme)); }
            }
            TokenType::String => {
                if tok.lexeme != "hello" { return Err(format!("[FAIL] String lexeme unexpected: {}", tok.lexeme)); }
            }
            _ => {}
        }
    }

    // final EOF
    let eof = scanner.next_token().ok_or_else(|| "[FAIL] Expected EOF but got None".to_string())?;
    if eof.get_type() != TokenType::Eof { return Err(format!("[FAIL] Expected EOF got {:?}", eof.get_type())); }

    Ok(())
}

#[test]
fn scanner_nested_block_comments() -> TestResult {
    let content = "/* outer /* nested */ still outer */ var x = 1;";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "test_input2.txt", content)?;
    let mut scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader initialization failed: {}", e))?;

    let expect = vec![TokenType::Var, TokenType::Identifier, TokenType::Equal, TokenType::Number, TokenType::Semicolon, TokenType::Eof];
    for et in expect {
        let tok = scanner.next_token().ok_or_else(|| format!("[FAIL] Expected token {:?} but got None", et))?;
        if tok.get_type() != et { return Err(format!("[FAIL] In nested comment test expected {:?} got {:?}", et, tok.get_type())); }
    }

    Ok(())
}

#[test]
fn scanner_unterminated_string() -> TestResult {
    // Unterminated string should cause scanner to log an error and produce EOF
    let content = "\"unterminated string\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "test_input3.txt", content)?;
    let mut scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader initialization failed: {}", e))?;

    let tok = scanner.next_token().ok_or_else(|| "[FAIL] Expected EOF token but got None".to_string())?;
    if tok.get_type() != TokenType::Eof { return Err(format!("[FAIL] Unterminated string test expected EOF, got {:?}", tok.get_type())); }

    // Ensure the scanner recorded an unterminated-string error
    let errs = scanner.take_errors();
    if errs.is_empty() { return Err("[FAIL] Expected unterminated string error but none recorded".to_string()); }
    if !errs.iter().any(|s| s.contains("Unterminated string")) {
        return Err(format!("[FAIL] Expected unterminated string message, got: {:?}", errs));
    }

    Ok(())
}
