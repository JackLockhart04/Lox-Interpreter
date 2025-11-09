use crate::input::reader::Reader;
use crate::token::token::{ Token, TokenType };

use crate::util::logger::{ LogLevel, global_logger };

use std::io;

// The internal buffer for storing the current line of input.
pub struct Scanner {
    source: Reader,
    line_number: usize,
    at_eof: bool,
    next_token_cache: Token,
    next_token_loaded: bool,
}

impl Scanner {
    // Inits
    pub fn new_from_terminal() -> Self {
        let scanner = Scanner {
            source: Reader::new_from_terminal(),
            line_number: 0,
            at_eof: false,
            next_token_cache: Token::new_token(TokenType::Eof, "".to_string(), None, 0),
            next_token_loaded: false,
        };
        scanner
    }

    pub fn new_from_file(path: &str) -> Result<Self, io::Error> {
        let scanner = Scanner {
            source: Reader::new_from_file(path)?,
            line_number: 0,
            at_eof: false,
            next_token_cache: Token::new_token(TokenType::Eof, "".to_string(), None, 0),
            next_token_loaded: false,
        };
        Ok(scanner)
    }

    // Methods
    pub fn check_single_char_token(&mut self, c: char) -> Option<Token> {
        match c {
            '(' => Some(Token::new_token(TokenType::LeftParen, "(".to_string(), None, self.source.get_line_number())),
            ')' => Some(Token::new_token(TokenType::RightParen, ")".to_string(), None, self.source.get_line_number())),
            '{' => Some(Token::new_token(TokenType::LeftBrace, "{".to_string(), None, self.source.get_line_number())),
            '}' => Some(Token::new_token(TokenType::RightBrace, "}".to_string(), None, self.source.get_line_number())),
            ',' => Some(Token::new_token(TokenType::Comma, ",".to_string(), None, self.source.get_line_number())),
            '.' => Some(Token::new_token(TokenType::Dot, ".".to_string(), None, self.source.get_line_number())),
            '-' => Some(Token::new_token(TokenType::Minus, "-".to_string(), None, self.source.get_line_number())),
            '+' => Some(Token::new_token(TokenType::Plus, "+".to_string(), None, self.source.get_line_number())),
            ';' => Some(Token::new_token(TokenType::Semicolon, ";".to_string(), None, self.source.get_line_number())),
            '*' => Some(Token::new_token(TokenType::Star, "*".to_string(), None, self.source.get_line_number())),
            _ => None,
        }
    }

    pub fn check_two_char_token(&mut self, c1: char, c2: Option<char>) -> Option<Token> {
        match (c1, c2) {
            // Check for double character tokens
            ('<', Some('=')) => Some(Token::new_token(TokenType::LessEqual, "<=".to_string(), None, self.source.get_line_number())),
            ('>', Some('=')) => Some(Token::new_token(TokenType::GreaterEqual, ">=".to_string(), None, self.source.get_line_number())),
            ('=', Some('=')) => Some(Token::new_token(TokenType::EqualEqual, "==".to_string(), None, self.source.get_line_number())),
            ('!', Some('=')) => Some(Token::new_token(TokenType::BangEqual, "!=".to_string(), None, self.source.get_line_number())),

            // Check for singles after not doubles
            ('<', _) => Some(Token::new_token(TokenType::Less, "<".to_string(), None, self.source.get_line_number())),
            ('>', _) => Some(Token::new_token(TokenType::Greater, ">".to_string(), None, self.source.get_line_number())),
            ('=', _) => Some(Token::new_token(TokenType::Equal, "=".to_string(), None, self.source.get_line_number())),
            ('!', _) => Some(Token::new_token(TokenType::Bang, "!".to_string(), None, self.source.get_line_number())),

            _ => None,
        }
    }

    // Helpers 
    pub fn is_white_space(c: char) -> bool {
        match c {
            ' ' | '\r' | '\t' | '\n' => true,
            _ => false,
        }
    }
    pub fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }
    pub fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }
    pub fn is_alphanumeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    // Comment handling
    pub fn skip_comment(&mut self) {
        while let Some(c) = self.source.next_char() {
            if c == '\n' {
                break;
            }
        }
    }
    pub fn skip_block_comment(&mut self) {
        loop {
            let first_char = self.source.next_char();
            if first_char.is_none() {
                break; // EOF reached
            }
            let first_char = first_char.unwrap();
            // Check for end of block comment
            if first_char == '*' {
                let second_char = self.source.peek_char();
                if let Some('/') = second_char {
                    self.source.next_char(); // Consume the '/'
                    break; // End of block comment
                }
            }

            // Check for nested block comments
            if first_char == '/' {
                let second_char = self.source.peek_char();
                if let Some('*') = second_char {
                    self.source.next_char(); // Consume the '*'
                    self.skip_block_comment(); // Recursively skip nested comment
                }
            }
        }
    }

    // Main token loading function
    pub fn load_token(&mut self) {
        let logger = global_logger();
        // If we've already hit EOF previously, keep the EOF token in the cache and return silently.
        if self.at_eof {
            self.next_token_cache = Token::new_token(TokenType::Eof, "".to_string(), None, self.source.get_line_number());
            return;
        }
        // logger.log(LogLevel::Debug, "Loading next token");
        let first_char_wrapper = self.source.next_char();
        // logger.log(LogLevel::Debug, format!("First char for token: '{:?}'", first_char_wrapper));

        let first_char = match first_char_wrapper {
            Some(c) => c,
            None => {
                // Reached EOF
                logger.log(LogLevel::Debug, "Reached EOF or unrecognized character, setting EOF token");
                self.at_eof = true;
                self.next_token_cache = Token::new_token(TokenType::Eof, "".to_string(), None, self.source.get_line_number());
                return;
            }
        };

        if Scanner::is_white_space(first_char) {
            // Skip whitespace and load next token
            self.load_token();
            return;
        }

        // Single token chars
        if let Some(single_char_token) = self.check_single_char_token(first_char) {
            // It's a single-character token
            self.next_token_cache = single_char_token;
            return;
        }

        // Two character tokens (and single-character fallbacks like '!' are
        // produced by check_two_char_token). Only consume the second input
        // character when we actually produced a two-character token (like
        // '!=' or '=='). Otherwise don't consume the peeked character.
        let second_char_wrapper = self.source.peek_char();
        if let Some(tok) = self.check_two_char_token(first_char, second_char_wrapper) {
            // If the token lexeme length is 2, it's a two-character token
            // and we should consume the second character from the source.
            if tok.lexeme.len() == 2 {
                self.source.next_char(); // Consume the second character
            }
            self.next_token_cache = tok;
            return;
        }

        // Division and comments handled below
        if first_char == '/' {
            if let Some(second_char) = second_char_wrapper {
                if second_char == '/' {
                    // It's a comment, consume until end of line
                    self.skip_comment();
                    // After consuming the comment, load the next token
                    self.load_token();
                    return;
                } else if second_char == '*' {
                    // It's a block comment, consume until matching */
                    self.skip_block_comment();
                    // After consuming the comment, load the next token
                    self.load_token();
                    return;
                } else {
                    // It's a division token
                    self.next_token_cache = Token::new_token(TokenType::Slash, first_char.to_string(), None, self.source.get_line_number());
                    return;
                }
            } else {
                // It's a division token at EOF
                self.next_token_cache = Token::new_token(TokenType::Slash, first_char.to_string(), None, self.source.get_line_number());
                return;
            }
        }

        // String literals
        if first_char == '"' {
            let mut string_content = String::new();
            while let Some(c) = self.source.next_char() {
                if c == '"' {
                    // End of string
                    self.next_token_cache = Token::new_token(TokenType::String, string_content, None, self.source.get_line_number());
                    return;
                } else {
                    string_content.push(c);
                }
            }
            // If we reach here, the string was not terminated
            logger.log(LogLevel::Error, "Unterminated string literal");
            self.at_eof = true;
            self.next_token_cache = Token::new_token(TokenType::Eof, "".to_string(), None, self.source.get_line_number());
            return;
        }

        // Number literals
        if Scanner::is_digit(first_char) {
            let mut number_content = String::new();
            number_content.push(first_char);

            while let Some(c) = self.source.peek_char() {
                if Scanner::is_digit(c) {
                    number_content.push(c);
                    self.source.next_char(); // Consume the digit
                } else {
                    break;
                }
            }

            // Handle fractional part
            if let Some('.') = self.source.peek_char() {
                // Peek ahead to see if there's a digit after the dot
                self.source.next_char(); // Consume the dot
                if let Some(next_c) = self.source.peek_char() {
                    if Scanner::is_digit(next_c) {
                        number_content.push('.'); // Add the dot
                    } else {
                        // No digit after dot
                        // Cause an error
                        logger.log(LogLevel::Error, "Invalid number format");
                    }
                }
            }

            // Get decimal digits after the dot
            while let Some(c) = self.source.peek_char() {
                if Scanner::is_digit(c) {
                    number_content.push(c);
                    self.source.next_char(); // Consume the digit
                } else {
                    break;
                }
            }

            // let number_value = number_content.parse::<f64>();
            // if number_value.is_err() {
            //     logger.log(LogLevel::Error, "Invalid number format");
            //     self.next_token_cache = Token::new_token(TokenType::Eof, "".to_string(), None, 0);
            //     return;
            // }
            self.next_token_cache = Token::new_token(TokenType::Number, number_content, None, self.source.get_line_number());
            return;
        }

        // Identifiers and keywords
        if Scanner::is_alpha(first_char) {
            let mut identifier_content = String::new();
            identifier_content.push(first_char);

            while let Some(c) = self.source.peek_char() {
                if Scanner::is_alphanumeric(c) {
                    identifier_content.push(c);
                    self.source.next_char(); // Consume the character
                } else {
                    break;
                }
            }

            // FIXME to check for reserved keywords here
            // Map reserved keywords to their token types
            let token_type = match identifier_content.as_str() {
                "and" => TokenType::And,
                "class" => TokenType::Class,
                "else" => TokenType::Else,
                "false" => TokenType::False,
                "for" => TokenType::For,
                "fun" => TokenType::Fun,
                "if" => TokenType::If,
                "nil" => TokenType::Nil,
                "or" => TokenType::Or,
                "print" => TokenType::Print,
                "return" => TokenType::Return,
                "super" => TokenType::Super,
                "this" => TokenType::This,
                "true" => TokenType::True,
                "var" => TokenType::Var,
                "while" => TokenType::While,
                _ => TokenType::Identifier,
            };

            self.next_token_cache = Token::new_token(token_type, identifier_content, None, self.source.get_line_number());
            return;
        }

        logger.log(LogLevel::Debug, "Reached EOF or unrecognized character, setting EOF token");
        self.at_eof = true;
        self.next_token_cache = Token::new_token(TokenType::Eof, "".to_string(), None, self.source.get_line_number());
    }

    pub fn next_token(&mut self) -> Option<Token> {
        // Implementation for tokenizing input goes here.
        if !self.next_token_loaded {
            self.load_token();
        }
        self.next_token_loaded = false;
        Some(self.next_token_cache.clone())
    }

    pub fn peek_token(&mut self) -> Option<Token> {
        if !self.next_token_loaded {
            self.load_token();
            self.next_token_loaded = true;
        }

        return Some(self.next_token_cache.clone());
    }

    pub fn get_line_number(&self) -> usize {
        return self.line_number;
    }

    #[allow(dead_code)]
    pub fn is_at_end(&self) -> bool {
        return self.at_eof;
    }
}