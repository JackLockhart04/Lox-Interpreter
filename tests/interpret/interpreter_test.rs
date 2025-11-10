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
