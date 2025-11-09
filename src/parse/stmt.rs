use crate::parse::expr::Expr;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var { name: Token, initializer: Option<Expr> },
    Function { name: Token, params: Vec<Token>, body: Vec<Stmt> },
    Block(Vec<Stmt>),
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { condition: Expr, body: Box<Stmt> },
}

pub trait Visitor<R> {
    fn visit_expression_stmt(&mut self, expr: &Expr) -> R;
    fn visit_print_stmt(&mut self, expr: &Expr) -> R;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> R;
    fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) -> R;
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> R;
    fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Box<Stmt>, else_branch: &Option<Box<Stmt>>) -> R;
    fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> R;
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr),
            Stmt::Print(expr) => visitor.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(name, initializer),
            Stmt::Function { name, params, body } => visitor.visit_function_stmt(name, params, body),
            Stmt::Block(stmts) => visitor.visit_block_stmt(stmts),
            Stmt::If { condition, then_branch, else_branch } => visitor.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => visitor.visit_while_stmt(condition, body),
        }
    }
}
