use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::parse::expr::LiteralValue;
use crate::token::token::Token;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<LiteralValue>>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// Create a new global (root) environment with no enclosing scope.
    pub fn new() -> Self {
        Environment { values: HashMap::new(), enclosing: None }
    }

    /// Create a new environment that encloses the given outer environment.
    pub fn new_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { values: HashMap::new(), enclosing: Some(enclosing) }
    }

    /// Define or redefine a variable in the current environment. This always
    /// affects only the current (innermost) scope.
    pub fn define(&mut self, name: &str, value: Option<LiteralValue>) {
        self.values.insert(name.to_string(), value);
    }

    /// Get a variable's value by token. Walks the chain of enclosing
    /// environments outward until the variable is found or we reach the root.
    pub fn get(&self, name: &Token) -> Result<Option<LiteralValue>, String> {
        if let Some(val) = self.values.get(&name.lexeme) {
            return Ok(val.clone());
        }

        if let Some(enclos) = &self.enclosing {
            return enclos.borrow().get(name);
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }

    /// Assign to an existing variable, walking enclosing environments if
    /// necessary. Returns Err if the variable doesn't exist in any enclosing
    /// scope.
    pub fn assign(&mut self, name: &Token, value: Option<LiteralValue>) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclos) = &self.enclosing {
            return enclos.borrow_mut().assign(name, value);
        }

        Err(format!("Undefined variable '{}'.", name.lexeme))
    }
}
