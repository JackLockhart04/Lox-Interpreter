use lox_interpreter::input::reader::Reader;
use std::fs; 
use tempfile;

type TestResult = Result<(), String>;

#[test]
fn reader_from_file() -> TestResult {
    // Setup
    let mut content = String::from("Hello World!\n");
    content.push_str("This is a test file.\n");
    content.push_str("lots of lines\n\n\n\n\n");
    content.push_str("abcdefghijklmnopqrstuvwxyz!@#$%^&*()-_|[]{}~';:/<>,.+=\n");
    
    // Create a temporary, unique directory which cleans itself up when dropped.
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    // Define the path to the test file inside the temporary directory.
    let file_path = temp_dir.path().join("test_input.txt");
    
    // Write the test content to the file.
    fs::write(&file_path, &content).map_err(|e| format!("File Write Error: {}", e))?;

    // Initialize the Reader from the file path.
    let path_str = file_path.to_str().ok_or_else(|| "Invalid Path: Could not convert temp path to string".to_string())?;
    
    let mut reader = Reader::new_from_file(path_str)
        .map_err(|e| format!("Reader initialization failed unexpectedly: {}", e))?;

    let mut chars_read = 0;
    
    // Iterate through the expected characters and compare them to what the reader returns.
    for expected_char in content.chars() {
        // Test peek_char()
        match reader.peek_char() {
            Some(actual_char) => {
                if actual_char != expected_char {
                    return Err(format!("[FAIL] Mismatch at index {}. Expected '{}', but got '{}'.", 
                                        chars_read, expected_char, actual_char));
                }
            },
            None => return Err(format!("[FAIL] Prematurely reached end of buffer at index {}.", chars_read)),
        }
        // Test next_char()
        match reader.next_char() {
            Some(actual_char) => {
                if actual_char != expected_char {
                    return Err(format!("[FAIL] Mismatch at index {}. Expected '{}', but got '{}'.", 
                                        chars_read, expected_char, actual_char));
                }
            },
            None => return Err(format!("[FAIL] Prematurely reached end of buffer at index {}.", chars_read)),
        }
        chars_read += 1;
    }

    // Ensure the reader is now empty
    if reader.next_char().is_some() {
        return Err("[FAIL] Expected end of file (None), but continued reading characters.".to_string());
    }

    Ok(())
}
