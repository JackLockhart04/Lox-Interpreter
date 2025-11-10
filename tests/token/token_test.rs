use lox_interpreter::token::token::Token;
use lox_interpreter::token::token::TokenType;

type TestResult = Result<(), String>;

#[test]
fn token_functionality() -> TestResult {
    // Create tokens of different types and test their properties.
    // Identifier token
    let token1 = Token {
        token_type: TokenType::Identifier,
        lexeme: "myVar".to_string(),
        literal: None,
        line: 1,
    };
    if token1.token_type != TokenType::Identifier {
        return Err("[FAIL] Token type mismatch for identifier.".to_string());
    }
    // Check lexeme
    if token1.lexeme != "myVar" {
        return Err("[FAIL] Token lexeme mismatch.".to_string());
    }
    // Check line number
    if token1.line != 1 {
        return Err("[FAIL] Token line number mismatch.".to_string());
    }



    Ok(())
}
