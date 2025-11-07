use crate::parse::expr::{Expr, Visitor, BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr, LiteralValue};
use crate::token::token::{TokenType, Token};
use crate::util::logger::{global_logger, LogLevel};

/// The Interpreter evaluates expressions and returns runtime values.
/// For now it implements the Visitor trait with return type Option<LiteralValue>
/// and only provides the literal evaluation as described (returns the literal's value).
pub struct Interpreter;

#[derive(Debug, Clone)]
pub struct RuntimeError {
	pub token: Token,
	pub message: String,
}

impl RuntimeError {
	pub fn new(token: Token, message: &str) -> Self {
		RuntimeError { token, message: message.to_string() }
	}
}

impl Interpreter {
	pub fn new() -> Self {
		Interpreter
	}
}


impl Interpreter {
	/// Public API: evaluate the expression and print the result. If a runtime
	/// error occurs, report it via the Lox runtime_error handler and continue.
	pub fn interpret(&mut self, expr: &Expr) {
		match self.evaluate(expr) {
			Ok(value) => {
				println!("{}", self.stringify(&value));
			}
			Err(e) => {
				crate::lox::runtime_error(&e.token, &e.message);
			}
		}
	}

	fn stringify(&self, object: &Option<LiteralValue>) -> String {
		match object {
			None => "nil".to_string(),
			Some(LiteralValue::Number(n)) => {
				let mut text = format!("{}", n);
				if text.ends_with(".0") {
					text.truncate(text.len() - 2);
				}
				text
			}
			Some(LiteralValue::Str(s)) => s.clone(),
			Some(LiteralValue::Bool(b)) => b.to_string(),
		}
	}
}

impl Visitor<Result<Option<LiteralValue>, RuntimeError>> for Interpreter {
	fn visit_binary_expr(&mut self, _expr: &BinaryExpr) -> Result<Option<LiteralValue>, RuntimeError> {
		// Evaluate operands
		let left_val = self.evaluate(&_expr.left)?;
		let right_val = self.evaluate(&_expr.right)?;

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
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Number(a - b)));
			}
			TokenType::Slash => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Number(a / b)));
			}
			TokenType::Star => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Number(a * b)));
			}
			TokenType::Plus => {
				// Number + Number
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Ok(Some(LiteralValue::Number(a + b)));
				}
				// Str + Str
				match (&left_val, &right_val) {
					(Some(LiteralValue::Str(a)), Some(LiteralValue::Str(b))) => {
						let mut s = a.clone();
						s.push_str(b);
						return Ok(Some(LiteralValue::Str(s)));
					}
					_ => {
						return Err(RuntimeError::new(_expr.operator.clone(), "Operands must be two numbers or two strings."));
					}
				}
			}
			TokenType::Greater => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Bool(a > b)));
			}
			TokenType::GreaterEqual => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Bool(a >= b)));
			}
			TokenType::Less => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Bool(a < b)));
			}
			TokenType::LessEqual => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(LiteralValue::Bool(a <= b)));
			}
			TokenType::BangEqual => {
				return Ok(Some(LiteralValue::Bool(!Interpreter::is_equal(&left_val, &right_val))));
			}
			TokenType::EqualEqual => {
				return Ok(Some(LiteralValue::Bool(Interpreter::is_equal(&left_val, &right_val))));
			}
			_ => {
				// Unsupported operator
				logger.log(LogLevel::Error, "Unsupported binary operator.");
				return Ok(None);
			}
		}
	}

	fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Result<Option<LiteralValue>, RuntimeError> {
		// Evaluate the inner expression
		self.evaluate(&expr.expression)
	}

	fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Result<Option<LiteralValue>, RuntimeError> {
		// Return the literal's runtime value directly, per the spec.
		Ok(expr.value.clone())
	}

	fn visit_unary_expr(&mut self, _expr: &UnaryExpr) -> Result<Option<LiteralValue>, RuntimeError> {
		let right = self.evaluate(&_expr.right)?;
		match _expr.operator.get_type() {
			TokenType::Minus => {
				self.check_number_operand(&_expr.operator, &right)?;
				if let Some(LiteralValue::Number(n)) = right {
					return Ok(Some(LiteralValue::Number(-n)));
				}
				// Should never reach here because check_number_operand returns Err otherwise
				return Ok(None);
			}
			TokenType::Bang => {
				return Ok(Some(LiteralValue::Bool(!Interpreter::is_truthy(&right))));
			}
			_ => {
				let logger = global_logger();
				logger.log(LogLevel::Error, "Unsupported unary operator.");
				return Ok(None);
			}
		}
	}
}
impl Interpreter {
	fn evaluate(&mut self, expr: &Expr) -> Result<Option<LiteralValue>, RuntimeError> {
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

	fn check_number_operand(&self, operator: &Token, operand: &Option<LiteralValue>) -> Result<(), RuntimeError> {
		match operand {
			Some(LiteralValue::Number(_)) => Ok(()),
			_ => Err(RuntimeError::new(operator.clone(), "Operand must be a number.")),
		}
	}

	fn check_number_operands(&self, operator: &Token, left: &Option<LiteralValue>, right: &Option<LiteralValue>) -> Result<(), RuntimeError> {
		match (left, right) {
			(Some(LiteralValue::Number(_)), Some(LiteralValue::Number(_))) => Ok(()),
			_ => Err(RuntimeError::new(operator.clone(), "Operands must be numbers.")),
		}
	}
}

