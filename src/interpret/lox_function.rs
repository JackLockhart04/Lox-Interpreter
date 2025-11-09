use crate::token::token::Token;
use crate::parse::stmt::Stmt;
use crate::interpret::environment::Environment;
use crate::interpret::value::Value;
use crate::interpret::interpreter::RuntimeError;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct LoxFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>, closure: Rc<RefCell<Environment>>) -> Self {
        LoxFunction { name, params, body, closure }
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn call(&self, interpreter: &mut crate::interpret::interpreter::Interpreter, arguments: &Vec<Value>) -> Result<Option<Value>, RuntimeError> {
        // Create a new environment for the function execution, enclosing the closure
        let env = Rc::new(RefCell::new(Environment::new_enclosing(self.closure.clone())));
        // Bind parameters
        for (i, param) in self.params.iter().enumerate() {
            let arg = arguments.get(i).cloned().unwrap_or(Value::Nil);
            env.borrow_mut().define(&param.lexeme, Some(arg));
        }

        // Execute body in the new environment
        interpreter.execute_block(&self.body, env)?;
        // No return implementation yet -> functions return nil
        Ok(Some(Value::Nil))
    }
}
