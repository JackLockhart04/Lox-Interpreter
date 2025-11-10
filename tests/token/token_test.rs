use lox_interpreter::token::token::Token;
use lox_interpreter::token::token::TokenType;

type TestResult = Result<(), String>;

#[test]
fn token_functionality() -> TestResult {
    // Create tokens of different types and test their properties.
    // Create a variety of tokens and test their properties.
    let tokens = vec![
        // Identifier token
        Token {
            token_type: TokenType::Identifier,
            lexeme: "myVar".to_string(),
            literal: None,
            line: 1,
        },
        // Number token (literal holds the numeric text)
        Token {
            token_type: TokenType::Number,
            lexeme: "3.14".to_string(),
            literal: Some("3.14".to_string()),
            line: 2,
        },
        // String token (literal holds the string contents)
        Token {
            token_type: TokenType::String,
            lexeme: "\"hello\"".to_string(),
            literal: Some("hello".to_string()),
            line: 3,
        },
        // Single-character tokens
        Token {
            token_type: TokenType::LeftParen,
            lexeme: "(".to_string(),
            literal: None,
            line: 4,
        },
        Token {
            token_type: TokenType::Plus,
            lexeme: "+".to_string(),
            literal: None,
            line: 4,
        },
    ];

    // Validate each token's fields
    // Identifier
    let t0 = &tokens[0];
    if t0.token_type != TokenType::Identifier { return Err("[FAIL] Token type mismatch for identifier.".to_string()); }
    if t0.lexeme != "myVar" { return Err("[FAIL] Token lexeme mismatch for identifier.".to_string()); }
    if t0.literal.is_some() { return Err("[FAIL] Identifier should have no literal.".to_string()); }
    if t0.line != 1 { return Err("[FAIL] Token line number mismatch for identifier.".to_string()); }

    // Number
    let t1 = &tokens[1];
    if t1.token_type != TokenType::Number { return Err("[FAIL] Token type mismatch for number.".to_string()); }
    if t1.lexeme != "3.14" { return Err("[FAIL] Token lexeme mismatch for number.".to_string()); }
    if t1.literal.as_deref() != Some("3.14") { return Err("[FAIL] Token literal mismatch for number.".to_string()); }
    if t1.line != 2 { return Err("[FAIL] Token line number mismatch for number.".to_string()); }

    // String
    let t2 = &tokens[2];
    if t2.token_type != TokenType::String { return Err("[FAIL] Token type mismatch for string.".to_string()); }
    if t2.lexeme != "\"hello\"" { return Err("[FAIL] Token lexeme mismatch for string.".to_string()); }
    if t2.literal.as_deref() != Some("hello") { return Err("[FAIL] Token literal mismatch for string.".to_string()); }
    if t2.line != 3 { return Err("[FAIL] Token line number mismatch for string.".to_string()); }

    // Single-character tokens (LeftParen, Plus)
    let t3 = &tokens[3];
    if t3.token_type != TokenType::LeftParen { return Err("[FAIL] Token type mismatch for left paren.".to_string()); }
    if t3.lexeme != "(" { return Err("[FAIL] Token lexeme mismatch for left paren.".to_string()); }

    let t4 = &tokens[4];
    if t4.token_type != TokenType::Plus { return Err("[FAIL] Token type mismatch for plus.".to_string()); }
    if t4.lexeme != "+" { return Err("[FAIL] Token lexeme mismatch for plus.".to_string()); }



    Ok(())
}
