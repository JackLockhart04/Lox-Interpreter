use lox_interpreter::input::scanner::Scanner;

use lox_interpreter::token::token::{ TokenType };

use std::fs; 
use tempfile;

type TestResult = Result<(), String>;

#[test]
fn scanner_from_file() -> TestResult {
    // Setup
    let content = String::from(";\n*.()\n");
    // Vector of expected tokens in order
    let expected_token_types = vec![TokenType::Semicolon, TokenType::Star, TokenType::Dot, TokenType::LeftParen, TokenType::RightParen];

    // let mut content = String::from("Hello World!\n");
    // content.push_str("This is a test file.\n");
    // content.push_str("lots of lines\n\n\n\n\n");
    // content.push_str("abcdefghijklmnopqrstuvwxyz!@#$%^&*()-_|[]{}~';:/<>,.+=\n");
    
    // Create a temporary, unique directory which cleans itself up when dropped.
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    // Define the path to the test file inside the temporary directory.
    let file_path = temp_dir.path().join("test_input.txt");
    
    // Write the test content to the file.
    fs::write(&file_path, &content).map_err(|e| format!("File Write Error: {}", e))?;

    // Initialize the Reader from the file path.
    let path_str = file_path.to_str().ok_or_else(|| "Invalid Path: Could not convert temp path to string".to_string())?;
    let mut scanner = Scanner::new_from_file(path_str)
        .map_err(|e| format!("Reader initialization failed unexpectedly: {}", e))?;

    let mut chars_read = 0;
    
    // Iterate through the expected characters and compare them to what the reader returns.
    for expected_token_type in expected_token_types {
        // Test peek
        let peeked_token = scanner.peek_token();
        

        // Test next_token
        let token = scanner.next_token();
        match token {
            Some(c) => {
                let token_type = c.get_type();
                if token_type != expected_token_type {
                    return Err(format!(
                        "[FAIL] Expected token '{:?}', but got '{:?}' at char position {}",
                        expected_token_type, token_type, chars_read
                    ));
                }
                chars_read += 1;
            }
            None => {
                return Err(format!(
                    "[FAIL] Expected token '{:?}', but got None (EOF reached too early)",
                    expected_token_type
                ));
            }
        }
    }

    // Ensure the scanner returns Eof token
    // match scanner.next_token().get_type() {
    //     TokenType::Eof => {
    //         // Success
    //     }
    //     other => {
    //         return Err(format!(
    //             "[FAIL] Expected Eof token, but got '{:?}'",
    //             other
    //         ));
    //     }
    // }

    Ok(())
}
