use crate::parse::stmt::Stmt;
use crate::interpret::environment::Environment;
use crate::interpret::value::Value;
use crate::interpret::interpreter::RuntimeError;
use std::rc::Rc;
use std::cell::RefCell;
use crate::interpret::callable::LoxCallable;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub declaration: Stmt,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Self {
        LoxFunction { declaration }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => 0,
        }
    }

    pub fn call(&self, interpreter: &mut crate::interpret::interpreter::Interpreter, arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError> {
        // Create a new environment for the function execution, enclosing the global environment
        let env = Rc::new(RefCell::new(Environment::new_enclosing(interpreter.globals.clone())));

        // Bind parameters from the function declaration
        if let Stmt::Function { params, body, .. } = &self.declaration {
            for (i, param) in params.iter().enumerate() {
                let arg = arguments.get(i).cloned().unwrap_or(Value::Nil);
                env.borrow_mut().define(&param.lexeme, Some(arg));
            }

            // Execute the function body in the new environment
            interpreter.execute_block(body, env)?;
        }

        // No return implementation yet -> functions return nil
        Ok(Some(Value::Nil))
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize { self.arity() }

    fn call(&self, interpreter: &mut crate::interpret::interpreter::Interpreter, arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError> {
        self.call(interpreter, arguments)
    }

    fn to_string(&self) -> String {
        match &self.declaration {
            Stmt::Function { name, .. } => format!("<fn {}>", name.lexeme),
            _ => "<fn>".to_string(),
        }
    }
}
