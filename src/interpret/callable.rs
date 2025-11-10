use crate::interpret::value::Value;
use crate::interpret::interpreter::{Interpreter, RuntimeError};

// Trait representing any callable Lox value (native or user-defined).
pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError>;
    fn to_string(&self) -> String;
}

// A small native clock function implementation.
pub struct NativeClock;

impl LoxCallable for NativeClock {
    fn arity(&self) -> usize { 0 }

    fn call(&self, _interpreter: &mut Interpreter, _arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError> {
        let secs = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64) / 1000.0;
        Ok(Some(Value::Number(secs)))
    }

    fn to_string(&self) -> String { "<native fn>".to_string() }
}
