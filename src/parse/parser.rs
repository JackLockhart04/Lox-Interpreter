use crate::input::scanner::Scanner;
use crate::token::token::{Token, TokenType};
use crate::parse::expr::{Expr, BinaryExpr, UnaryExpr, GroupingExpr, LiteralExpr, LiteralValue};
use crate::util::logger::{global_logger, LogLevel};

#[derive(Debug, Clone)]
pub struct ParseError {
    token: Token,
    message: String,
    line: usize,
}

pub struct Parser {
    token_source: Scanner,
    // #[allow(dead_code)]
    errors: Vec<ParseError>,
    had_error: bool,
}

impl Parser {
    // Allow dead code
    #[allow(dead_code)]
    pub fn new(token_source: Scanner) -> Self {
        Parser {
            token_source,
            errors: Vec::new(),
            had_error: false,
        }
    }

    pub fn match_token(&mut self, types: &[TokenType]) -> bool {
        // Doesnt consume token for now
        let next_token = self.token_source.peek_token();
        if let Some(token) = next_token {
            for ttype in types {
                if token.get_type() == *ttype {
                    // self.token_source.next_token();
                    return true;
                }
            }
        }
        return false;
    }

    pub fn error(&mut self, token: Token, message: &str) {
        // Ensure error hasn't been reported yet
        if self.had_error {
            return;
        }
        let parse_error = ParseError {
            token: token.clone(),
            message: message.to_string(),
            line: token.line,
        };
        self.errors.push(parse_error.clone());

        self.had_error = true;

        // Synchronize the parser state after an error
        self.synchronize();

        return;
    }

    pub fn report_errors(&mut self) {
        let logger = global_logger();
        for error in &self.errors {
            // eprintln!("[line {}] Error at '{}': {}", error.line, error.token.lexeme, error.message);
            logger.log(LogLevel::Error, format!("[line {}] Error at '{}': {}", error.line, error.token.lexeme, error.message));
        }
        self.had_error = false;
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Return true if the underlying token source (scanner) has reached EOF.
    pub fn is_at_end(&mut self) -> bool {
        self.token_source.is_at_end()
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.token_source.peek_token() {
            // If we've reached EOF, stop synchronizing.
            if token.get_type() == TokenType::Eof {
                return;
            }
            if token.get_type() == TokenType::Semicolon {
                self.token_source.next_token();
                // Print that we synchronized at semicolon
                return;
            }

            match token.get_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.token_source.next_token();
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        let expr = self.expression();
        // if the program is expected to be a statement (like in the Lox REPL).
        if expr.is_some() {
            // Check for ';' and consume it. This is usually handled by a 'consume'
            // method in Lox parsers that signals an error if the token is missing.
            match self.token_source.peek_token() {
                Some(token) if token.get_type() == TokenType::Semicolon => {
                    // Success, expression followed by ';'
                    self.token_source.next_token();
                    return expr;
                }
                Some(_token) => {
                    // Found a token, but it's not a Semicolon
                    // Add an error and return None
                    self.error(_token, "Expect ';' after expression statement.");
                    return None;
                    // return Err(self.error(token, "Expect ';' after expression statement."));
                }
                None => {
                    // Reached EOF, which is sometimes okay, but often an error if ';' was expected.
                    // Assuming an error if not followed by a semicolon for now.
                    self.error(Token::new_token(TokenType::Eof, "".to_string(), None, 0), "Expect ';' after expression statement.");
                    return None;
                }
            }
        }
        
        // If the expression itself failed to parse, just return the failure
        self.error(Token::new_token(TokenType::Eof, "".to_string(), None, 0), "Expect expression. (from parse())");
        return None;
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let expr = self.comparison();
        // Handle no expr
        if expr.is_none() {
            return None;
        }

        if self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(left) = expr {
                if let Some(right) = self.equality() {
                    return Some(Expr::Binary(BinaryExpr {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }));
                }
            }

            // If we reach here, it means we had an operator but failed to parse left or right
            self.error(operator, "Expect expression after operator.");
            return None;
        }
        
        return expr;
    }

    fn comparison(&mut self) -> Option<Expr> {
        let expr = self.term();
        // Handle no expr
        if expr.is_none() {
            return None;
        }

        if self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(left) = expr {
                if let Some(right) = self.comparison() {
                    return Some(Expr::Binary(BinaryExpr {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }));
                }
            }

            // If we reach here, it means we had an operator but failed to parse left or right
            self.error(operator, "Expect expression after operator.");
            return None;
        }
        
        return expr;
    }

    fn term(&mut self) -> Option<Expr> {
        let expr = self.factor();
        // Handle no expr
        if expr.is_none() {
            return None;
        }

        if self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(left) = expr {
                if let Some(right) = self.term() {
                    return Some(Expr::Binary(BinaryExpr {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }));
                }
            }

            // If we reach here, it means we had an operator but failed to parse left or right
            self.error(operator, "Expect expression after operator.");
            return None;
        }
        
        return expr;
    }

    fn factor(&mut self) -> Option<Expr> {
        // 1. Get the left-hand expression result.
        let expr = self.unary();
        // Handle no expr
        if expr.is_none() {
            return None;
        }

        // 2. Check for the operator and consume it.
        if self.match_token(&[TokenType::Star, TokenType::Slash]) {
            // NOTE: If match_token returns true, consume the operator token.
            let operator = self.token_source.next_token().unwrap();

            // 3. Move the value out of 'expr' only if it exists (i.e., unary() succeeded).
            // This handles the E0382 error by ensuring we don't try to use 'expr' later.
            let left = match expr {
                Some(e) => e, // Success: 'e' is the left operand (moved out of 'expr')
                None => {
                    // Error Case 1: Operator found, but no left operand (e.g., '* 3').
                    // We must report an error here since the token was consumed.
                    self.error(operator, "Expect expression before operator.");
                    return None; 
                }
            };
            
            // 4. Recursively parse the right-hand expression.
            let right = match self.factor() {
                Some(e) => e,
                None => {
                    // Error Case 2: Operator found, but no right operand (e.g., '3 *').
                    self.error(operator, "Expect expression after operator.");
                    return None;
                }
            };

            // 5. Success: Construct and return the new Binary expression.
            return Some(Expr::Binary(BinaryExpr {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }));
        }

        // 6. If the token did NOT match (or if match_token failed), 
        //    return the original result from unary(). 
        //    This is safe because 'expr' was only moved inside the 'if' block.
        return expr;
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_token(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(right) = self.unary() {
                return Some(Expr::Unary(UnaryExpr {
                    operator,
                    right: Box::new(right),
                }));
            }
        }

        return self.primary();
    }

    fn primary(&mut self) -> Option<Expr> {
        // Parenthesized grouping expression
        if self.match_token(&[TokenType::LeftParen]) {
            // consume '('
            let _ = self.token_source.next_token();
            let inner = self.expression();

            // Expect closing ')'
            match self.token_source.peek_token() {
                Some(t) if t.get_type() == TokenType::RightParen => {
                    // consume ')'
                    self.token_source.next_token();
                    if let Some(expr_inner) = inner {
                        return Some(Expr::Grouping(GroupingExpr { expression: Box::new(expr_inner) }));
                    } else {
                        // No inner expression parsed
                        let tok = self.token_source.peek_token().unwrap_or(Token::new_token(TokenType::Eof, "".to_string(), None, 0));
                        self.error(tok, "Expect expression inside parentheses.");
                        return None;
                    }
                }
                Some(t) => {
                    // Found a token but not a right paren
                    self.error(t.clone(), "Expect ')' after expression.");
                    return None;
                }
                None => {
                    self.error(Token::new_token(TokenType::Eof, "".to_string(), None, 0), "Expect ')' after expression.");
                    return None;
                }
            }
        }
        // False, True, Nil
        if self.match_token(&[TokenType::False]) {
            let _token = self.token_source.next_token();
            return Some(Expr::Literal(LiteralExpr {
                value: Some(LiteralValue::Bool(false)),
            }));
        }
        if self.match_token(&[TokenType::True]) {
            let _token = self.token_source.next_token();
            return Some(Expr::Literal(LiteralExpr {
                value: Some(LiteralValue::Bool(true)),
            }));
        }
        if self.match_token(&[TokenType::Nil]) {
            let _token = self.token_source.next_token();
            return Some(Expr::Literal(LiteralExpr { value: None }));
        }

        // Number, String
        if self.match_token(&[TokenType::Number, TokenType::String]) {
            let token = self.token_source.peek_token().unwrap();
            match token.get_type() {
                TokenType::Number => {
                    let number_content = token.lexeme.parse::<f64>().ok()?;
                    // Use the token after parsing
                    self.token_source.next_token();
                    return Some(Expr::Literal(LiteralExpr {
                        value: Some(LiteralValue::Number(number_content)),
                    }));
                }
                TokenType::String => {
                    // Use the token after parsing
                    self.token_source.next_token().unwrap();
                    return Some(Expr::Literal(LiteralExpr {
                        value: Some(LiteralValue::Str(token.lexeme)),
                    }));
                }
                _ => {}
            }

            // // Not handled: Grouping, Identifiers, etc. For now, throw an error
            // let token = self.token_source.next_token().unwrap_or(Token::new_token(TokenType::Eof, "".to_string(), None, 0));
            // self.error(token, "Expect expression.");
            // return None;
        }

        // Not handled: Grouping, Identifiers, etc. For now, throw an error
        let token = self.token_source.peek_token().unwrap_or(Token::new_token(TokenType::Eof, "".to_string(), None, 0));
        // Log the token that caused the error
        self.error(token, "Expect expression.");
        return None;
    }
}