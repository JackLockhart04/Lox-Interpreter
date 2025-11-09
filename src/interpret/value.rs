use std::rc::Rc;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Str(String),
    Bool(bool),
    // Function will be stored as a reference counted pointer to a LoxFunction
    Function(Rc<crate::interpret::lox_function::LoxFunction>),
}
