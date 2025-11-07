use crate::token::token::Token;
use std::sync::atomic::{AtomicBool, Ordering};

static HAD_RUNTIME_ERROR: AtomicBool = AtomicBool::new(false);

pub fn runtime_error(token: &Token, message: &str) {
    eprintln!("{}\n[line {}]", message, token.line);
    HAD_RUNTIME_ERROR.store(true, Ordering::SeqCst);
}

pub fn had_runtime_error() -> bool {
    HAD_RUNTIME_ERROR.load(Ordering::SeqCst)
}
