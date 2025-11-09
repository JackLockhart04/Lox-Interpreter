use std::collections::HashMap;
use crate::parse::expr::LiteralValue;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Option<LiteralValue>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { values: HashMap::new() }
    }

    /// Define or redefine a variable in the current environment. We allow
    /// redefinition (useful in REPL sessions).
    pub fn define(&mut self, name: &str, value: Option<LiteralValue>) {
        self.values.insert(name.to_string(), value);
    }

    /// Get a variable's value by token. Returns Ok(Some(value)) if found,
    /// Ok(None) if the variable exists but is `nil`, or Err(message) if not
    /// defined.
    pub fn get(&self, name: &Token) -> Result<Option<LiteralValue>, String> {
        if let Some(val) = self.values.get(&name.lexeme) {
            Ok(val.clone())
        } else {
            Err(format!("Undefined variable '{}'.", name.lexeme))
        }
    }

    /// Assign to an existing variable. Returns Err if the variable is not defined.
    pub fn assign(&mut self, name: &Token, value: Option<LiteralValue>) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name.lexeme))
        }
    }
}
