use crate::input::scanner::Scanner;
use crate::token::token::{Token, TokenType};
use crate::parse::expr::{Expr, BinaryExpr, UnaryExpr, GroupingExpr, LiteralExpr, LiteralValue, AssignExpr};
use crate::parse::stmt::Stmt;
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

    // Return whether the parser has recorded a parsing error.
    pub fn had_error(&self) -> bool {
        self.had_error
    }

    /// Return true if the underlying token source (scanner) has reached EOF.
    pub fn is_at_end(&mut self) -> bool {
        // Previously this delegated to the scanner's `is_at_end()` flag.
        // That can race with the parser's control flow if the scanner only
        // creates the EOF token when asked; instead check the next token
        // directly so the parser stops parsing when the next token is EOF.
        match self.token_source.peek_token() {
            Some(tok) => tok.get_type() == TokenType::Eof,
            None => true,
        }
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

    // Parse a full program: a sequence of statements until EOF.
    pub fn parse(&mut self) -> Option<Stmt> {
        // either a declaration or a statement.
        self.declaration()
    }

    // Parse a declaration (top-level): currently only var declarations or statements.
    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(&[TokenType::Var]) {
            // consume the 'var' keyword
            let _ = self.token_source.next_token();
            return self.var_declaration();
        }
        return self.statement();
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        // Expect an identifier
        let name = match self.consume(TokenType::Identifier, "Expect variable name.") {
            Some(t) => t,
            None => return None,
        };

        // Optional initializer
        let mut initializer: Option<Expr> = None;
        if self.match_token(&[TokenType::Equal]) {
            // consume '='
            let _ = self.token_source.next_token();
            if let Some(expr) = self.expression() {
                initializer = Some(expr);
            } else {
                // expression parse will have recorded an error
                return None;
            }
        }

        // Consume terminating semicolon
        if self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.").is_none() {
            return None;
        }

        Some(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_token(&[TokenType::Print]) {
            // match_token only peeks; consume the 'print' token so
            // print_statement() can parse the following expression.
            let _ = self.token_source.next_token();
            return self.print_statement();
        }
        if self.match_token(&[TokenType::If]) {
            // consume 'if'
            let _ = self.token_source.next_token();
            return self.if_statement();
        }
        // Block statement
        if self.match_token(&[TokenType::LeftBrace]) {
            // consume '{'
            let _ = self.token_source.next_token();
            let stmts = self.block();
            return Some(Stmt::Block(stmts));
        }
        return self.expression_statement();
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        // Expect '('
        if self.consume(TokenType::LeftParen, "Expect '(' after 'if'.").is_none() {
            return None;
        }

        let condition = match self.expression() {
            Some(e) => e,
            None => return None,
        };

        if self.consume(TokenType::RightParen, "Expect ')' after if condition.").is_none() {
            return None;
        }

        // then branch
        let then_branch = match self.statement() {
            Some(s) => s,
            None => return None,
        };

        // optional else
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_token(&[TokenType::Else]) {
            // consume 'else'
            let _ = self.token_source.next_token();
            if let Some(eb) = self.statement() {
                else_branch = Some(Box::new(eb));
            } else {
                return None;
            }
        }

        Some(Stmt::If { condition, then_branch: Box::new(then_branch), else_branch })
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        while let Some(tok) = self.token_source.peek_token() {
            if tok.get_type() == TokenType::RightBrace || tok.get_type() == TokenType::Eof {
                break;
            }
            if let Some(decl) = self.declaration() {
                statements.push(decl);
            } else {
                // If a declaration failed, synchronize and continue parsing
                self.synchronize();
            }
        }

        // Consume the closing '}'
        let _ = self.consume(TokenType::RightBrace, "Expect '}' after block.");
        statements
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        // We have already consumed the 'print' in match_token; parse the expression
        let expr = self.expression();
        // consume semicolon
        if self.consume(TokenType::Semicolon, "Expect ';' after value.").is_none() {
            return None;
        }
        expr.map(|e| Stmt::Print(e))
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression();
        if self.consume(TokenType::Semicolon, "Expect ';' after expression.").is_none() {
            return None;
        }
        expr.map(|e| Stmt::Expression(e))
    }

    fn consume(&mut self, ttype: TokenType, message: &str) -> Option<Token> {
        match self.token_source.peek_token() {
            Some(tok) if tok.get_type() == ttype => return self.token_source.next_token(),
            Some(tok) => {
                self.error(tok, message);
                return None;
            }
            None => {
                self.error(Token::new_token(TokenType::Eof, "".to_string(), None, 0), message);
                return None;
            }
        }
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        // Parse the left-hand side as a logic_or (higher precedence than assignment)
        let expr = self.logic_or();
        if expr.is_none() {
            return None;
        }

    // If we see '=', this is an assignment expression (right-associative)
        if self.match_token(&[TokenType::Equal]) {
            // consume '='
            let equals = self.token_source.next_token().unwrap();
            // Parse the right-hand side as another assignment (right-associative)
            let value = self.assignment();

            // Ensure the left side is a valid assignment target (currently only simple variables)
            let left_expr = expr.unwrap();
            if let Some(val_expr) = value {
                match left_expr {
                    Expr::Variable(name) => {
                        return Some(Expr::Assign(AssignExpr { name, value: Box::new(val_expr) }));
                    }
                    _ => {
                        self.error(equals, "Invalid assignment target.");
                        return None;
                    }
                }
            } else {
                // right-hand side failed to parse
                self.error(equals, "Expect expression after '='.");
                return None;
            }
        }

        // No assignment; return the previously parsed expression
        expr
    }

    fn logic_or(&mut self) -> Option<Expr> {
        let mut expr = match self.logic_and() {
            Some(e) => e,
            None => return None,
        };

        while self.match_token(&[TokenType::Or]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(right) = self.logic_and() {
                expr = Expr::Logical(crate::parse::expr::LogicalExpr { left: Box::new(expr), operator, right: Box::new(right) });
            } else {
                self.error(operator, "Expect expression after 'or'.");
                return None;
            }
        }

        Some(expr)
    }

    fn logic_and(&mut self) -> Option<Expr> {
        let mut expr = match self.equality() {
            Some(e) => e,
            None => return None,
        };

        while self.match_token(&[TokenType::And]) {
            let operator = self.token_source.next_token().unwrap();
            if let Some(right) = self.equality() {
                expr = Expr::Logical(crate::parse::expr::LogicalExpr { left: Box::new(expr), operator, right: Box::new(right) });
            } else {
                self.error(operator, "Expect expression after 'and'.");
                return None;
            }
        }

        Some(expr)
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

        // Identifier (variable access)
        if self.match_token(&[TokenType::Identifier]) {
            // consume identifier
            if let Some(tok) = self.token_source.next_token() {
                return Some(Expr::Variable(tok));
            }
        }

        // Not handled: Grouping, Identifiers, etc. For now, throw an error
        let token = self.token_source.peek_token().unwrap_or(Token::new_token(TokenType::Eof, "".to_string(), None, 0));
        // Log the token that caused the error
        self.error(token, "Expect expression.");
        return None;
    }
}