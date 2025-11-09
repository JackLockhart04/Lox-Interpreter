use std::rc::Rc;
use std::fmt;
use crate::interpret::callable::LoxCallable;
use crate::parse::stmt::Stmt;

#[derive(Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Str(String),
    Bool(bool),
    // User-defined function
    Function(Rc<crate::interpret::lox_function::LoxFunction>),
    // Native or other callable implemented in Rust
    Native(Rc<dyn LoxCallable>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Str(s) => write!(f, "Str({})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Function(func) => match &func.declaration {
                Stmt::Function { name, .. } => write!(f, "Function({})", name.lexeme),
                _ => write!(f, "Function(<fn>)"),
            },
            Value::Native(_) => write!(f, "Native(<native fn>)"),
        }
    }
}
