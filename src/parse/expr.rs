use crate::token::token::Token; // Assuming your Token is defined in a 'token' module

// --- AST NODE STRUCTS ---

// All concrete expression types are defined as structs.
// We use Box<Expr> to hold sub-expressions, which is necessary for recursive
// data structures like trees in Rust.

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    // Use a concrete enum for literal values so the type is Clone + Debug
    // and easy to pattern-match later.
    pub value: Option<LiteralValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

// --- BASE EXPR ENUM ---

// The main Expr enum, which acts as the root of the expression hierarchy.
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Variable(Token),
    Assign(AssignExpr),
    Logical(LogicalExpr),
    Call(CallExpr),
    // You'll add more variants here as you expand Lox (e.g., Variable, Call, Assign)
}

// --- VISITOR TRAIT ---

// The Visitor trait defines the contract for all operations (interpreting, 
// printing, resolving) that can be performed on the AST.
// R is the generic return type of the operation.
pub trait Visitor<R> {
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> R;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> R;
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> R;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> R;
    fn visit_variable_expr(&mut self, name: &Token) -> R;
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> R;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> R;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> R;
}

impl Expr {
    // The "accept" method, which performs the double dispatch.
    // It matches on the specific expression type and calls the corresponding 
    // visit method on the provided visitor object.
    pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Expr::Binary(expr) => visitor.visit_binary_expr(expr),
            Expr::Grouping(expr) => visitor.visit_grouping_expr(expr),
            Expr::Literal(expr) => visitor.visit_literal_expr(expr),
            Expr::Unary(expr) => visitor.visit_unary_expr(expr),
            Expr::Variable(name) => visitor.visit_variable_expr(name),
            Expr::Assign(assign) => visitor.visit_assign_expr(assign),
            Expr::Logical(logical) => visitor.visit_logical_expr(logical),
            Expr::Call(call) => visitor.visit_call_expr(call),
        }
    }
}