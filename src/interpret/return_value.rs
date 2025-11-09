use std::cell::RefCell;
use crate::interpret::value::Value;

thread_local! {
    static RETURN_VALUE: RefCell<Option<Value>> = RefCell::new(None);
}

pub fn set_return(val: Option<Value>) {
    RETURN_VALUE.with(|c| {
        *c.borrow_mut() = val;
    });
}

pub fn take_return() -> Option<Value> {
    RETURN_VALUE.with(|c| c.borrow_mut().take())
}
