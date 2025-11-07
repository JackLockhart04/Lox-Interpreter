use crate::parse::expr::{Expr, Visitor, BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr, LiteralValue};
use crate::token::token::TokenType;
use crate::util::logger::{global_logger, LogLevel};

/// The Interpreter evaluates expressions and returns runtime values.
/// For now it implements the Visitor trait with return type Option<LiteralValue>
/// and only provides the literal evaluation as described (returns the literal's value).
pub struct Interpreter;

impl Interpreter {
	pub fn new() -> Self {
		Interpreter
	}

	/// Helper to interpret an expression tree.
	pub fn interpret(&mut self, expr: &Expr) -> Option<LiteralValue> {
		expr.accept(self)
	}
}

impl Visitor<Option<LiteralValue>> for Interpreter {
	fn visit_binary_expr(&mut self, _expr: &BinaryExpr) -> Option<LiteralValue> {
		// Evaluate operands
		let left_val = self.evaluate(&_expr.left);
		let right_val = self.evaluate(&_expr.right);

		let logger = global_logger();

		// Helper to extract number
		let as_number = |v: &Option<LiteralValue>| -> Option<f64> {
			match v {
				Some(LiteralValue::Number(n)) => Some(*n),
				_ => None,
			}
		};

		match _expr.operator.get_type() {
			TokenType::Minus => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Number(a - b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '-'\n");
				return None;
			}
			TokenType::Slash => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Number(a / b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '/'\n");
				return None;
			}
			TokenType::Star => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Number(a * b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '*'\n");
				return None;
			}
			TokenType::Plus => {
				// Number + Number
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Number(a + b));
				}
				// Str + Str
				match (&left_val, &right_val) {
					(Some(LiteralValue::Str(a)), Some(LiteralValue::Str(b))) => {
						let mut s = a.clone();
						s.push_str(b);
						return Some(LiteralValue::Str(s));
					}
					_ => {
						logger.log(LogLevel::Error, "Operands must be two numbers or two strings for '+'.\n");
						return None;
					}
				}
			}
			TokenType::Greater => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Bool(a > b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '>'.\n");
				return None;
			}
			TokenType::GreaterEqual => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Bool(a >= b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '>='.\n");
				return None;
			}
			TokenType::Less => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Bool(a < b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '<'.\n");
				return None;
			}
			TokenType::LessEqual => {
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Some(LiteralValue::Bool(a <= b));
				}
				logger.log(LogLevel::Error, "Operands must be numbers for '<='.");
				return None;
			}
			TokenType::BangEqual => {
				return Some(LiteralValue::Bool(!Interpreter::is_equal(&left_val, &right_val)));
			}
			TokenType::EqualEqual => {
				return Some(LiteralValue::Bool(Interpreter::is_equal(&left_val, &right_val)));
			}
			_ => {
				// Unsupported operator
				logger.log(LogLevel::Error, "Unsupported binary operator.");
				return None;
			}
		}
	}

	fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Option<LiteralValue> {
		// Evaluate the inner expression
		self.evaluate(&expr.expression)
	}

	fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Option<LiteralValue> {
		// Return the literal's runtime value directly, per the spec.
		expr.value.clone()
	}

	fn visit_unary_expr(&mut self, _expr: &UnaryExpr) -> Option<LiteralValue> {
		let right = self.evaluate(&_expr.right);
		let logger = global_logger();
		match _expr.operator.get_type() {
			TokenType::Minus => {
				match right {
					Some(LiteralValue::Number(n)) => return Some(LiteralValue::Number(-n)),
					_ => {
						logger.log(LogLevel::Error, "Operand must be a number for unary '-'.");
						return None;
					}
				}
			}
			TokenType::Bang => {
				return Some(LiteralValue::Bool(!Interpreter::is_truthy(&right)));
			}
			_ => {
				logger.log(LogLevel::Error, "Unsupported unary operator.");
				return None;
			}
		}
	}
}

impl Interpreter {
	fn evaluate(&mut self, expr: &Expr) -> Option<LiteralValue> {
		expr.accept(self)
	}

	fn is_truthy(val: &Option<LiteralValue>) -> bool {
		match val {
			None => false,
			Some(LiteralValue::Bool(b)) => *b,
			_ => true,
		}
	}

	fn is_equal(a: &Option<LiteralValue>, b: &Option<LiteralValue>) -> bool {
		if a.is_none() && b.is_none() {
			return true;
		}
		if a.is_none() || b.is_none() {
			return false;
		}
		// Both Some -> compare
		return a == b;
	}
}

