use crate::interpret::expr::{Expr, Visitor, BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr, LiteralValue};

/// The AstPrinter implements the Visitor trait to produce a string representation of the AST.
pub struct AstPrinter;

impl AstPrinter {
    /// Convenience method to print a full expression.
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }

    /// Helper function to generate Lisp-style parenthesized output.
    fn parenthesize(&mut self, name: &str, parts: &[&Expr]) -> String {
        let mut output = String::new();
        output.push_str("(");
        output.push_str(name);

        for part in parts {
            output.push_str(" ");
            // Recursive call to accept() on the sub-expression
            output.push_str(&part.accept(self)); 
        }
        output.push_str(")");
        output
    }
}

// Implement the Visitor trait, setting the return type R to String.
impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> String {
        // Uses the operator's lexeme, and recursively calls print on left and right
        self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> String {
        self.parenthesize("group", &[&expr.expression])
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> String {
        match &expr.value {
            Some(LiteralValue::Number(n)) => format!("{}", n),
            Some(LiteralValue::Str(s)) => s.clone(),
            Some(LiteralValue::Bool(b)) => format!("{}", b),
            None => "nil".to_string(),
        }
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> String {
        // Uses the operator's lexeme, and recursively calls print on the right
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }
}
