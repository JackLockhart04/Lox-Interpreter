// src/input/reader.rs

use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;

// An internal enum to track the origin of the input, making the reader's state explicit.
enum InputSource {
    Terminal,
    #[allow(dead_code)] 
    File { reader: BufReader<File>, path: String },
}

// The internal buffer for storing the current line of input.
pub struct Reader {
    // Stores the input line read from the terminal or the entire file content.
    char_buffer: Vec<char>,
    line_position: usize,
    line_number: usize,
    // Tracks the source of the input.
    source: InputSource, 
    at_eof: bool,
}

impl Reader {
    // Creates a new Reader instance, initialized for interactive terminal input.
    pub fn new_from_terminal() -> Self {
        Reader {
            char_buffer: Vec::new(),
            line_position: 0,
            line_number: 0,
            source: InputSource::Terminal,
            at_eof: false,
        }
    }
    
    // Initialization Method 2 (File Input)
    pub fn new_from_file(path: &str) -> Result<Self, io::Error> {
        // Open the file and create a buffered reader for incremental reading
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);

        Ok(Reader {
            char_buffer: Vec::new(),
            line_position: 0,
            line_number: 0,
            source: InputSource::File { reader: buf_reader, path: path.to_string() },
            at_eof: false,
        })
    }

    // Read in new line based on the input source
    pub fn read_in_line(&mut self) -> io::Result<bool> {
        match &mut self.source {
            // From terminal input
            InputSource::Terminal => {
                // First prompt the user for input
                print!("> ");
                io::stdout().flush().ok();
                let mut input = String::new();
                let n = io::stdin().read_line(&mut input)?;
                if n == 0 {
                    // EOF
                    self.at_eof = true;
                    return Ok(false);
                }
                // let trimmed = input.trim_end().to_string();
                // self.char_buffer = trimmed.chars().collect();
                let normalized = input.replace("\r\n", "\n").replace('\r', "");
                self.char_buffer = normalized.chars().collect();
                self.line_position = 0;
                self.line_number += 1;
                Ok(true)
            }
            // From file input
            InputSource::File { reader, .. } => {
                let mut line = String::new();
                let n = reader.read_line(&mut line)?;
                if n == 0 {
                    // EOF reached
                    self.at_eof = true;
                    return Ok(false);
                }
                // let trimmed = line.trim_end().to_string();
                // self.char_buffer = trimmed.chars().collect();
                let normalized = line.replace("\r\n", "\n").replace('\r', "");
                self.char_buffer = normalized.chars().collect();
                self.line_position = 0;
                self.line_number += 1;

                // Echo the file line being read so file-mode behaves like the REPL.
                // Ensure we always emit a terminating newline even if the input
                // file's last line does not include one; otherwise the program's
                // printed output can appear on the same line as the echoed source.
                print!("> ");
                io::stdout().flush().ok();
                print!("{}", normalized);
                if !normalized.ends_with('\n') {
                    // Normalized line lacked a newline (likely the file's last
                    // line). Emit one so subsequent println!() calls start on
                    // the next line.
                    print!("\n");
                }
                io::stdout().flush().ok();

                Ok(true)
            }
        }
    }

    // Get next character from the current line buffer, or None if at end
    pub fn next_char(&mut self) -> Option<char> {
        // Loop: keep reading lines until we find a character to return or reach EOF/error.
        loop {
            if self.line_position < self.char_buffer.len() {
                let ch = self.char_buffer[self.line_position];
                self.line_position += 1;
                return Some(ch);
            }

            // Current buffer exhausted: attempt to read the next line.
            match self.read_in_line() {
                Ok(true) => {}
                Ok(false) => return None, // EOF
                Err(_) => return None,     // I/O error treated as end
            }
        }
    }

    pub fn peek_char(&mut self) -> Option<char> {
        // Loop: keep reading lines until we find a character to return or reach EOF/error.
        loop {
            if self.line_position < self.char_buffer.len() {
                let ch = self.char_buffer[self.line_position];
                return Some(ch);
            }

            // Current buffer exhausted: attempt to read the next line.
            match self.read_in_line() {
                Ok(true) => {}
                Ok(false) => return None, // EOF
                Err(_) => return None,     // I/O error treated as end
            }
        }
    }

    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> usize {
        self.line_position
    }
}