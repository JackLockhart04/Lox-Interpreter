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
    // The environment that was active when the function was declared.
    // This is the "closure" that lets the function access surrounding locals.
    pub closure: std::rc::Rc<std::cell::RefCell<crate::interpret::environment::Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: std::rc::Rc<std::cell::RefCell<crate::interpret::environment::Environment>>) -> Self {
        LoxFunction { declaration, closure }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => 0,
        }
    }

    pub fn call(&self, interpreter: &mut crate::interpret::interpreter::Interpreter, arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError> {
    // Create a new environment for the function execution, enclosing the closure
    // captured when the function was declared.
    let env = Rc::new(RefCell::new(Environment::new_enclosing(self.closure.clone())));

        // Bind parameters from the function declaration
        if let Stmt::Function { params, body, .. } = &self.declaration {
            for (i, param) in params.iter().enumerate() {
                let arg = arguments.get(i).cloned().unwrap_or(Value::Nil);
                env.borrow_mut().define(&param.lexeme, Some(arg));
            }

            // Execute the function body in the new environment, catching return panics
            use std::panic::{catch_unwind, resume_unwind, take_hook, set_hook};

            // Temporarily install a no-op panic hook so the unwind doesn't print to stderr
            let prev_hook = take_hook();
            set_hook(Box::new(|_info| {}));

            let res = catch_unwind(std::panic::AssertUnwindSafe(|| {
                interpreter.execute_block(body, env)
            }));

            // Restore previous panic hook
            set_hook(prev_hook);

            match res {
                Ok(inner_res) => {
                    // Normal completion (no return)
                    inner_res?;
                }
                Err(payload) => {
                    // If this was our return marker, extract the stored return value
                    if let Some(s) = payload.downcast_ref::<&str>() {
                        if *s == "__LOX_RETURN__" {
                            let rv = crate::interpret::return_value::take_return();
                            return Ok(rv);
                        }
                    }
                    // Otherwise, resume unwinding
                    resume_unwind(payload);
                }
            }
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
