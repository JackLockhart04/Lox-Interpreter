use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::parse::parser::Parser;
use lox_interpreter::parse::expr::Expr;
use lox_interpreter::parse::expr::LiteralValue;
use lox_interpreter::parse::stmt::Stmt;
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
fn parser_literal_expression() -> TestResult {
    // Parse a single literal expression
    let content = "123;\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p1.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;

    match parsed {
        Stmt::Expression(expr) => {
            match expr {
                Expr::Literal(lit) => {
                    match lit.value {
                        Some(LiteralValue::Number(n)) => {
                            if (n - 123.0).abs() > std::f64::EPSILON { return Err(format!("Expected 123.0 got {}", n)); }
                        }
                        _ => return Err("Expected Number literal".to_string()),
                    }
                }
                _ => return Err("Expected literal expression".to_string()),
            }
        }
        _ => return Err("Expected expression statement".to_string()),
    }

    Ok(())
}

#[test]
fn parser_precedence_expression() -> TestResult {
    // Test that precedence parsing produces 1 + (2 * 3)
    let content = "1 + 2 * 3;\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p2.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;

    // Expect top-level: Binary(+, left=1, right=Binary(*,2,3))
    match parsed {
        Stmt::Expression(expr) => {
            if let Expr::Binary(bin) = expr {
                // top operator should be Plus
                if bin.operator.get_type() != TokenType::Plus { return Err(format!("Expected + as top operator, got {:?}", bin.operator.get_type())); }
                // left should be literal 1
                if let Expr::Literal(left_lit) = *bin.left {
                    match left_lit.value {
                        Some(LiteralValue::Number(n)) => if (n - 1.0).abs() > std::f64::EPSILON { return Err(format!("Left literal expected 1, got {}", n)); },
                        _ => return Err("Left was not a number literal".to_string()),
                    }
                } else { return Err("Left side not literal".to_string()); }

                // right should be a Binary with '*'
                if let Expr::Binary(right_bin) = *bin.right {
                    if right_bin.operator.get_type() != TokenType::Star { return Err(format!("Expected * in right subtree, got {:?}", right_bin.operator.get_type())); }
                    // check literals 2 and 3
                    if let Expr::Literal(l2) = *right_bin.left {
                        match l2.value { Some(LiteralValue::Number(n)) => if (n - 2.0).abs() > std::f64::EPSILON { return Err(format!("Expected 2 got {}", n)); }, _ => return Err("Expected number literal".to_string()) }
                    } else { return Err("Right.left not literal".to_string()); }
                    if let Expr::Literal(l3) = *right_bin.right {
                        match l3.value { Some(LiteralValue::Number(n)) => if (n - 3.0).abs() > std::f64::EPSILON { return Err(format!("Expected 3 got {}", n)); }, _ => return Err("Expected number literal".to_string()) }
                    } else { return Err("Right.right not literal".to_string()); }
                } else { return Err("Right side not binary".to_string()); }

            } else { return Err("Top-level expression not binary".to_string()); }
        }
        _ => return Err("Expected expression statement".to_string()),
    }

    Ok(())
}

#[test]
fn parser_call_expression() -> TestResult {
    // Parse a call: foo(1, 2);
    let content = "foo(1, 2);\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p3.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;

    match parsed {
        Stmt::Expression(expr) => {
            if let Expr::Call(call) = expr {
                // check that callee is variable 'foo'
                if let Expr::Variable(tok) = *call.callee {
                    if tok.lexeme != "foo" { return Err(format!("Expected callee 'foo', got {}", tok.lexeme)); }
                } else { return Err("Callee not a variable".to_string()); }

                if call.arguments.len() != 2 { return Err(format!("Expected 2 args got {}", call.arguments.len())); }

                // first arg literal 1
                if let Expr::Literal(l1) = &call.arguments[0] {
                    match &l1.value { Some(LiteralValue::Number(n)) => if (*n - 1.0).abs() > std::f64::EPSILON { return Err(format!("Expected 1 got {}", n)); }, _ => return Err("Arg1 not number".to_string()) }
                } else { return Err("Arg1 not literal".to_string()); }

                // second arg literal 2
                if let Expr::Literal(l2) = &call.arguments[1] {
                    match &l2.value { Some(LiteralValue::Number(n)) => if (*n - 2.0).abs() > std::f64::EPSILON { return Err(format!("Expected 2 got {}", n)); }, _ => return Err("Arg2 not number".to_string()) }
                } else { return Err("Arg2 not literal".to_string()); }

            } else { return Err("Top-level expression not call".to_string()); }
        }
        _ => return Err("Expected expression statement".to_string()),
    }

    Ok(())
}

#[test]
fn parser_function_declaration() -> TestResult {
    // Parse a function declaration
    let content = "fun f(a, b) { return a; }\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p4.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;

    match parsed {
        Stmt::Function { name, params, body } => {
            if name.lexeme != "f" { return Err(format!("Expected function name f got {}", name.lexeme)); }
            if params.len() != 2 { return Err(format!("Expected 2 params got {}", params.len())); }
            if params[0].lexeme != "a" || params[1].lexeme != "b" { return Err("Param names mismatch".to_string()); }
            if body.is_empty() { return Err("Function body empty".to_string()); }
            // first body stmt should be a return
            match &body[0] {
                Stmt::Return { keyword: _, value } => {
                    if value.is_none() { return Err("Return had no value".to_string()); }
                }
                _ => return Err("Expected return stmt in body".to_string()),
            }
        }
        _ => return Err("Expected Function declaration".to_string()),
    }

    Ok(())
}

#[test]
fn parser_var_declaration() -> TestResult {
    let content = "var x = 42;\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p5.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }

    match parsed {
        Stmt::Var { name, initializer } => {
            if name.lexeme != "x" { return Err(format!("Expected var name x got {}", name.lexeme)); }
            if let Some(Expr::Literal(lit)) = initializer {
                if let Some(LiteralValue::Number(n)) = lit.value { if (n - 42.0).abs() > std::f64::EPSILON { return Err(format!("Expected 42 got {}", n)); } }
                else { return Err("Initializer not number literal".to_string()); }
            } else { return Err("Initializer missing or not literal".to_string()); }
        }
        _ => return Err("Expected Var declaration".to_string()),
    }

    Ok(())
}


#[test]
fn parser_if_else_and_block() -> TestResult {
    let content = "if (true) { 1; } else 2;\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p6.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }

    match parsed {
        Stmt::If { condition, then_branch, else_branch } => {
            // condition should be literal true
            if let Expr::Literal(lit) = condition {
                match lit.value { Some(LiteralValue::Bool(b)) => if !b { return Err("Expected true condition".to_string()); }, _ => return Err("Condition not boolean".to_string()) }
            } else { return Err("Condition not literal".to_string()); }

            // then branch should be a block
            match *then_branch {
                Stmt::Block(ref vec) => {
                    if vec.is_empty() { return Err("Then block empty".to_string()); }
                }
                _ => return Err("Then branch not block".to_string()),
            }

            // else branch present
            if else_branch.is_none() { return Err("Expected else branch".to_string()); }
        }
        _ => return Err("Expected If statement".to_string()),
    }

    Ok(())
}


#[test]
fn parser_assignment_and_block() -> TestResult {
    let content = "{ var a = 1; a = 2; }\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p7.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }

    match parsed {
        Stmt::Block(stmts) => {
            if stmts.len() != 2 { return Err(format!("Expected 2 statements in block got {}", stmts.len())); }
            match &stmts[1] {
                Stmt::Expression(expr) => {
                    if let Expr::Assign(assign) = expr {
                        if assign.name.lexeme != "a" { return Err("Assign target wrong".to_string()); }
                    } else { return Err("Expected assign expression".to_string()); }
                }
                _ => return Err("Expected expression statement for assignment".to_string()),
            }
        }
        _ => return Err("Expected block statement".to_string()),
    }

    Ok(())
}


#[test]
fn parser_logical_ops() -> TestResult {
    let content = "true and false or true;\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p8.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }

    match parsed {
        Stmt::Expression(expr) => {
            if let Expr::Logical(l) = expr {
                // top operator should be Or
                if l.operator.get_type() != TokenType::Or { return Err("Expected 'or' as top operator".to_string()); }
                // left should itself be Logical (and)
                if let Expr::Logical(_) = *l.left { /* ok */ } else { return Err("Expected left side to be 'and' logical".to_string()); }
            } else { return Err("Expected logical expression".to_string()); }
        }
        _ => return Err("Expected expression statement".to_string()),
    }

    Ok(())
}


#[test]
fn parser_chained_calls_and_unary_grouping() -> TestResult {
    // chained calls and unary with grouping
    let content = "f()(1)(2); -(1 + 2);\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p9.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    // First top-level stmt: f()(1)(2);
    let parsed1 = parser.parse().ok_or_else(|| "Parser returned None for first stmt".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }
    match parsed1 {
        Stmt::Expression(expr) => {
            // Expect a call whose callee is a call whose callee is a call or var
            if let Expr::Call(outer) = expr {
                if let Expr::Call(mid) = *outer.callee {
                    // mid.callee should be a call or variable 'f'
                    match *mid.callee {
                        Expr::Call(_) => { /* possible but accept */ }
                        Expr::Variable(tok) => if tok.lexeme != "f" { return Err("Expected callee 'f' in chained calls".to_string()); },
                        _ => return Err("Unexpected callee shape in chained calls".to_string()),
                    }
                } else { return Err("Expected middle call in chained calls".to_string()); }
            } else { return Err("Expected call expression".to_string()); }
        }
        _ => return Err("Expected expression statement".to_string()),
    }

    // Parse second stmt: -(1 + 2);
    let parsed2 = parser.parse().ok_or_else(|| "Parser returned None for second stmt".to_string())?;
    match parsed2 {
        Stmt::Expression(expr) => {
            if let Expr::Unary(_u) = expr { /* unary ok */ } else { return Err("Expected unary expression".to_string()); }
        }
        _ => return Err("Expected expression statement for unary".to_string()),
    }

    Ok(())
}


#[test]
fn parser_function_no_params_empty_body() -> TestResult {
    let content = "fun g() { }\n";
    let temp_dir = tempfile::tempdir().map_err(|e| format!("TempDir creation failed: {}", e))?;
    let path = write_temp_file(&temp_dir, "p10.txt", content)?;
    let scanner = Scanner::new_from_file(&path).map_err(|e| format!("Reader init failed: {}", e))?;
    let mut parser = Parser::new(scanner);

    let parsed = parser.parse().ok_or_else(|| "Parser returned None".to_string())?;
    if parser.had_error() { return Err("Parser reported error".to_string()); }

    match parsed {
        Stmt::Function { name, params, body } => {
            if name.lexeme != "g" { return Err("Expected function name 'g'".to_string()); }
            if !params.is_empty() { return Err("Expected no params".to_string()); }
            if !body.is_empty() { return Err("Expected empty body".to_string()); }
        }
        _ => return Err("Expected Function declaration".to_string()),
    }

    Ok(())
}
