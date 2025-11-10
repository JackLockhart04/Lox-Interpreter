use lox_interpreter::input::scanner::Scanner;
use lox_interpreter::parse::parser::Parser;
use lox_interpreter::interpret::interpreter::Interpreter;
use lox_interpreter::interpret::value::Value;

use std::fs;
use tempfile;

type TestResult = Result<(), String>;

fn write_temp_file(temp_dir: &tempfile::TempDir, filename: &str, content: &str) -> Result<String, String> {
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).map_err(|e| format!("File Write Error: {}", e))?;
    Ok(file_path.to_str().ok_or_else(|| "Invalid Path".to_string())?.to_string())
}

// Helper to run a file through parser+interpreter loop (like main())
fn run_file_and_return_interpreter(path: &str) -> Result<Interpreter, String> {
    let scanner = Scanner::new_from_file(path).map_err(|e| format!("Scanner init failed: {}", e))?;
    let mut parser = Parser::new(scanner);
    let mut interpreter = Interpreter::new();

    while !parser.is_at_end() {
        match parser.parse() {
            Some(stmt) => interpreter.interpret_stmt(&stmt),
            None => {
                if parser.had_error() {
                    parser.report_errors();
                    parser.clear_errors();
                }
            }
        }
    }

    Ok(interpreter)
}

#[test]
fn interpret_var_and_lookup() -> TestResult {
    let content = "var x = 99;\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "i1.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let val = interp.get_global("x");
    match val {
        Some(Value::Number(n)) => if (n - 99.0).abs() > std::f64::EPSILON { return Err(format!("Expected 99 got {}", n)); },
        _ => return Err("Expected numeric global 'x'".to_string()),
    }
    Ok(())
}

#[test]
fn interpret_function_and_call() -> TestResult {
    let content = "fun f() { return 1; } var r = f();\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "i2.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let val = interp.get_global("r");
    match val {
        Some(Value::Number(n)) => if (n - 1.0).abs() > std::f64::EPSILON { return Err(format!("Expected 1 got {}", n)); },
        _ => return Err("Expected numeric global 'r'".to_string()),
    }
    Ok(())
}

#[test]
fn interpret_closure_makecounter() -> TestResult {
    let content = r#"
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    return i;
  }
  return count;
}
var c = makeCounter();
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "i3.txt", content)?;

    let mut interp = run_file_and_return_interpreter(&path)?;

    // Get the function object "c" and call it twice via API to assert closure behavior.
    let vc = interp.get_global("c");
    match vc {
        Some(Value::Function(rcf)) => {
            // call once
            let res1 = rcf.call(&mut interp, &Vec::new()).map_err(|e| format!("Runtime error: {}", e.message))?;
            match res1 {
                Some(Value::Number(n)) => if (n - 1.0).abs() > std::f64::EPSILON { return Err(format!("Expected 1 got {}", n)); },
                other => return Err(format!("Expected numeric return from c() first call, got {:?}", other)),
            }

            // call twice
            let res2 = rcf.call(&mut interp, &Vec::new()).map_err(|e| format!("Runtime error: {}", e.message))?;
            match res2 {
                Some(Value::Number(n)) => if (n - 2.0).abs() > std::f64::EPSILON { return Err(format!("Expected 2 got {}", n)); },
                other => return Err(format!("Expected numeric return from c() second call, got {:?}", other)),
            }
        }
        other => return Err(format!("Expected function value for c, got {:?}", other)),
    }

    Ok(())
}

#[test]
fn interpret_native_clock_smoke() -> TestResult {
    let content = "var t = clock();\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "i4.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let vt = interp.get_global("t");
    match vt {
        Some(Value::Number(_n)) => Ok(()),
        _ => Err("Expected numeric global 't' from clock()".to_string()),
    }
}

#[test]
fn interpret_assignment_expression_returns_value() -> TestResult {
    // Assignment is an expression and returns the assigned value.
    let content = "var a = 1; var r = (a = 5);\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "assign_expr.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let ra = interp.get_global("r");
    match ra {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global 'r' from assignment expression".to_string()),
    }

    let aa = interp.get_global("a");
    match aa {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global 'a' after assignment".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_function_arity_errors_prevent_assignment() -> TestResult {
    // Calling a function with wrong arity should produce a runtime error and
    // the variable initializer should not be set.
    let content = "fun f(a,b) { return a; } var r = f(1);\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "arity_err.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let vr = interp.get_global("r");
    if vr.is_some() { return Err("Expected 'r' to be unset due to arity error".to_string()); }
    Ok(())
}

#[test]
fn interpret_recursion_factorial() -> TestResult {
    let content = r#"
fun fact(n) {
  if (n <= 1) return 1;
  return n * fact(n - 1);
}
var r = fact(5);
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "recursion.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let vr = interp.get_global("r");
    match vr {
        Some(Value::Number(n)) => if (n - 120.0).abs() > std::f64::EPSILON { return Err(format!("Expected 120 got {}", n)); },
        _ => return Err("Expected numeric global 'r' from factorial".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_native_clock_wrong_arity() -> TestResult {
    // Calling native clock with arguments should produce a runtime error and
    // not set the target variable.
    let content = "var t = clock(1);\n";
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "native_arity.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let vt = interp.get_global("t");
    if vt.is_some() { return Err("Expected 't' to be unset due to arity error".to_string()); }
    Ok(())
}

// --- Additional interpreter tests merged in ---

#[test]
fn interpret_value_equality_cases() -> TestResult {
    let content = r#"
var s_eq = "a" == "a";
var n_eq = 1 == 1;
var mix_eq = 1 == "1";
var nil_eq = nil == nil;
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "eqs.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;

    match interp.get_global("s_eq") {
        Some(Value::Bool(b)) => if !b { return Err("Expected s_eq true".to_string()); },
        _ => return Err("Expected boolean global s_eq".to_string()),
    }

    match interp.get_global("n_eq") {
        Some(Value::Bool(b)) => if !b { return Err("Expected n_eq true".to_string()); },
        _ => return Err("Expected boolean global n_eq".to_string()),
    }

    match interp.get_global("mix_eq") {
        Some(Value::Bool(b)) => if b { return Err("Expected mix_eq false".to_string()); },
        _ => return Err("Expected boolean global mix_eq".to_string()),
    }

    match interp.get_global("nil_eq") {
        Some(Value::Bool(b)) => if !b { return Err("Expected nil_eq true".to_string()); },
        _ => return Err("Expected boolean global nil_eq".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_return_unwind_nested_blocks() -> TestResult {
    let content = r#"
fun outer() {
  { { return 5; } }
  return 0;
}
var r = outer();
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "return_unwind.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("r") {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global r from outer()".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_function_debug_string() -> TestResult {
    let content = r#"
fun greet() { }
var gf = greet;
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "fn_debug.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    let val = interp.get_global("gf");
    match val {
        Some(v) => {
            let s = format!("{:?}", v);
            if !s.contains("Function(greet)") { return Err(format!("Expected Function(greet) debug got {}", s)); }
        }
        None => return Err("Expected gf to be defined".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_calling_non_callable_reports_error_but_preserves_value() -> TestResult {
    let content = r#"
var vcall = 123;
vcall();
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "noncallable.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("vcall") {
        Some(Value::Number(n)) => if (n - 123.0).abs() > std::f64::EPSILON { return Err(format!("Expected 123 got {}", n)); },
        _ => return Err("Expected numeric global vcall to remain after non-callable call".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_comparisons_and_truthiness() -> TestResult {
    let content = r#"
var lt = 1 < 2;
var le = 2 <= 2;
var gt = 5 > 3;
var ge = 5 >= 6;

var sc = false;
var res1 = true or (sc = true);
var res2 = false or (sc = true);
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "cmp_truth.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;

    match interp.get_global("lt") {
        Some(Value::Bool(b)) => if !b { return Err("Expected lt true".to_string()); },
        _ => return Err("Expected boolean global lt".to_string()),
    }
    match interp.get_global("le") {
        Some(Value::Bool(b)) => if !b { return Err("Expected le true".to_string()); },
        _ => return Err("Expected boolean global le".to_string()),
    }
    match interp.get_global("gt") {
        Some(Value::Bool(b)) => if !b { return Err("Expected gt true".to_string()); },
        _ => return Err("Expected boolean global gt".to_string()),
    }
    match interp.get_global("ge") {
        Some(Value::Bool(b)) => if b { return Err("Expected ge false".to_string()); },
        _ => return Err("Expected boolean global ge".to_string()),
    }

    match interp.get_global("res1") {
        Some(Value::Bool(b)) => if !b { return Err("Expected res1 true".to_string()); },
        _ => return Err("Expected boolean global res1".to_string()),
    }
    match interp.get_global("res2") {
        Some(Value::Bool(b)) => if !b { return Err("Expected res2 true".to_string()); },
        _ => return Err("Expected boolean global res2".to_string()),
    }

    match interp.get_global("sc") {
        Some(Value::Bool(b)) => if !b { return Err("Expected sc true after res2".to_string()); },
        _ => return Err("Expected boolean global sc".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_string_number_concatenation() -> TestResult {
    let content = r#"
var s1 = "one " + 1;
var s2 = 1 + "two";
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "concat.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;

    match interp.get_global("s1") {
        Some(Value::Str(s)) => if s != "one 1" { return Err(format!("Expected 'one 1' got {}", s)); },
        _ => return Err("Expected string global s1".to_string()),
    }
    match interp.get_global("s2") {
        Some(Value::Str(s)) => if s != "1two" { return Err(format!("Expected '1two' got {}", s)); },
        _ => return Err("Expected string global s2".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_function_arity_success() -> TestResult {
    let content = r#"
fun inc(a) { return a + 1; }
var r = inc(4);
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "arity_ok.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("r") {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global r from inc(4)".to_string()),
    }

    Ok(())
}

#[test]
fn interpret_assign_to_undefined_variable_reports_error_and_does_not_create() -> TestResult {
    let content = r#"
a = 1;
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "assign_undef.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    if interp.get_global("a").is_some() { return Err("Expected 'a' to be undefined after assignment to undefined variable".to_string()); }
    Ok(())
}

#[test]
fn interpret_assign_to_outer_from_inner_block() -> TestResult {
    let content = r#"
var outer = 10;
{
  outer = 5;
}
var r = outer;
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "assign_outer.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("r") {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global r after assigning outer from block".to_string()),
    }
    Ok(())
}

#[test]
fn interpret_shadowing_inner_does_not_affect_outer() -> TestResult {
    let content = r#"
var v = "outer";
{
  var v = "inner";
}
var r = v;
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "shadow.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("r") {
        Some(Value::Str(s)) => if s != "outer" { return Err(format!("Expected 'outer' got {}", s)); },
        _ => return Err("Expected string global r after shadowing test".to_string()),
    }
    Ok(())
}

#[test]
fn interpret_higher_order_function_call() -> TestResult {
    let content = r#"
fun apply(f, x) { return f(x); }
fun inc(a) { return a + 1; }
var r = apply(inc, 4);
"#;
    let td = tempfile::tempdir().map_err(|e| format!("TempDir failed: {}", e))?;
    let path = write_temp_file(&td, "hof.txt", content)?;

    let interp = run_file_and_return_interpreter(&path)?;
    match interp.get_global("r") {
        Some(Value::Number(n)) => if (n - 5.0).abs() > std::f64::EPSILON { return Err(format!("Expected 5 got {}", n)); },
        _ => return Err("Expected numeric global r from apply(inc,4)".to_string()),
    }
    Ok(())
}
