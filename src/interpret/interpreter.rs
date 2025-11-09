use crate::parse::expr::{Expr, Visitor, BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr, AssignExpr, LogicalExpr};
use crate::parse::stmt::{Stmt, Visitor as StmtVisitor};
use crate::token::token::{TokenType, Token};
use crate::interpret::environment::Environment;
use crate::util::logger::{global_logger, LogLevel};
use std::rc::Rc;
use std::cell::RefCell;
use crate::interpret::value::Value;

/// The Interpreter evaluates expressions and returns runtime values.
/// It keeps a simple global environment (flat scope) for variable declarations.
pub struct Interpreter {
	pub(crate) globals: Rc<RefCell<Environment>>,
	environment: Rc<RefCell<Environment>>,
}

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
		let globals = Rc::new(RefCell::new(Environment::new()));
		// Put native functions into globals
		// Register clock native function
		let clock = crate::interpret::callable::NativeClock;
		globals.borrow_mut().define("clock", Some(crate::interpret::value::Value::Native(std::rc::Rc::new(clock))));

		Interpreter { globals: globals.clone(), environment: globals }
	}
}


impl Interpreter {
	fn stringify(&self, object: &Option<Value>) -> String {
		match object {
			None => "nil".to_string(),
			Some(Value::Nil) => "nil".to_string(),
			Some(Value::Number(n)) => {
				let mut text = format!("{}", n);
				if text.ends_with(".0") {
					text.truncate(text.len() - 2);
				}
				text
			}
			Some(Value::Str(s)) => s.clone(),
			Some(Value::Bool(b)) => b.to_string(),
			Some(Value::Function(f)) => match &f.declaration {
				Stmt::Function { name, .. } => format!("<fn {}>", name.lexeme),
				_ => "<fn>".to_string(),
			},
			Some(Value::Native(n)) => n.to_string(),
		}
	}

	/// Execute a list of statements (a program). Errors are reported via
	/// crate::lox::runtime_error but the interpreter continues executing
	/// subsequent statements.
	pub fn interpret(&mut self, statements: &Vec<Stmt>) {
		for stmt in statements {
			if let Err(e) = self.execute(stmt) {
				crate::lox::runtime_error(&e.token, &e.message);
			}
		}
	}

	fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
		stmt.accept(self)
	}

}

impl Interpreter {
	/// Execute a single statement and report runtime errors via Lox runtime_error.
	pub fn interpret_stmt(&mut self, stmt: &Stmt) {
		if let Err(e) = self.execute(stmt) {
			crate::lox::runtime_error(&e.token, &e.message);
		}
	}
}

impl Visitor<Result<Option<Value>, RuntimeError>> for Interpreter {
	fn visit_binary_expr(&mut self, _expr: &BinaryExpr) -> Result<Option<Value>, RuntimeError> {
		// Evaluate operands
		let left_val = self.evaluate(&_expr.left)?;
		let right_val = self.evaluate(&_expr.right)?;

		let logger = global_logger();

		// Helper to extract number
		let as_number = |v: &Option<Value>| -> Option<f64> {
			match v {
				Some(Value::Number(n)) => Some(*n),
				_ => None,
			}
		};

		match _expr.operator.get_type() {
			TokenType::Minus => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Number(a - b)));
			}
			TokenType::Slash => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Number(a / b)));
			}
			TokenType::Star => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Number(a * b)));
			}
			TokenType::Plus => {
				// Number + Number
				if let (Some(a), Some(b)) = (as_number(&left_val), as_number(&right_val)) {
					return Ok(Some(Value::Number(a + b)));
				}

				// If either operand is a string, convert both to strings and concatenate.
				if matches!(left_val, Some(Value::Str(_))) || matches!(right_val, Some(Value::Str(_))) {
					let left_s = self.stringify(&left_val);
					let right_s = self.stringify(&right_val);
					return Ok(Some(Value::Str(format!("{}{}", left_s, right_s))));
				}

				// Otherwise it's a type error
				return Err(RuntimeError::new(_expr.operator.clone(), "Operands must be two numbers or two strings."));
			}
			TokenType::Greater => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Bool(a > b)));
			}
			TokenType::GreaterEqual => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Bool(a >= b)));
			}
			TokenType::Less => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Bool(a < b)));
			}
			TokenType::LessEqual => {
				self.check_number_operands(&_expr.operator, &left_val, &right_val)?;
				let a = as_number(&left_val).unwrap();
				let b = as_number(&right_val).unwrap();
				return Ok(Some(Value::Bool(a <= b)));
			}
			TokenType::BangEqual => {
				return Ok(Some(Value::Bool(!Interpreter::is_equal(&left_val, &right_val))));
			}
			TokenType::EqualEqual => {
				return Ok(Some(Value::Bool(Interpreter::is_equal(&left_val, &right_val))));
			}
			_ => {
				// Unsupported operator
				logger.log(LogLevel::Error, "Unsupported binary operator.");
				return Ok(None);
			}
		}
	}


	fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Result<Option<Value>, RuntimeError> {
		// Evaluate the right-hand side
		let value = self.evaluate(&expr.value)?;
		// Try to assign into the environment. If the variable is undefined, return a runtime error.
		match self.environment.borrow_mut().assign(&expr.name, value.clone()) {
			Ok(()) => Ok(value),
			Err(msg) => Err(RuntimeError::new(expr.name.clone(), &msg)),
		}
	}

	fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Result<Option<Value>, RuntimeError> {
		let left = self.evaluate(&expr.left)?;
		match expr.operator.get_type() {
			TokenType::Or => {
				if Interpreter::is_truthy(&left) {
					return Ok(left);
				}
			}
			TokenType::And => {
				if !Interpreter::is_truthy(&left) {
					return Ok(left);
				}
			}
			_ => {}
		}
		// Not short-circuited; evaluate and return right
		let right = self.evaluate(&expr.right)?;
		Ok(right)
	}

	fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Result<Option<Value>, RuntimeError> {
		// Evaluate the inner expression
		self.evaluate(&expr.expression)
	}

	fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Result<Option<Value>, RuntimeError> {
		// Convert AST literal into runtime Value
		match &expr.value {
			None => Ok(Some(Value::Nil)),
			Some(crate::parse::expr::LiteralValue::Number(n)) => Ok(Some(Value::Number(*n))),
			Some(crate::parse::expr::LiteralValue::Str(s)) => Ok(Some(Value::Str(s.clone()))),
			Some(crate::parse::expr::LiteralValue::Bool(b)) => Ok(Some(Value::Bool(*b))),
		}
	}

	fn visit_unary_expr(&mut self, _expr: &UnaryExpr) -> Result<Option<Value>, RuntimeError> {
		let right = self.evaluate(&_expr.right)?;
		match _expr.operator.get_type() {
			TokenType::Minus => {
				self.check_number_operand(&_expr.operator, &right)?;
				if let Some(Value::Number(n)) = right {
					return Ok(Some(Value::Number(-n)));
				}
				// Should never reach here because check_number_operand returns Err otherwise
				return Ok(None);
			}
			TokenType::Bang => {
				return Ok(Some(Value::Bool(!Interpreter::is_truthy(&right))));
			}
			_ => {
				let logger = global_logger();
				logger.log(LogLevel::Error, "Unsupported unary operator.");
				return Ok(None);
			}
		}
	}

		fn visit_variable_expr(&mut self, name: &Token) -> Result<Option<Value>, RuntimeError> {
			match self.environment.borrow().get(name) {
				Ok(val) => Ok(val),
				Err(msg) => Err(RuntimeError::new(name.clone(), &msg)),
			}
		}

		fn visit_call_expr(&mut self, expr: &crate::parse::expr::CallExpr) -> Result<Option<Value>, RuntimeError> {
			// Evaluate callee
			let callee_val = self.evaluate(&expr.callee)?;

			// Evaluate arguments
			let mut arguments: Vec<Value> = Vec::new();
			for arg_expr in &expr.arguments {
				let v = self.evaluate(arg_expr)?;
				// convert Option<Value> to Value, treating None as Nil
				let val = match v {
					Some(vv) => vv,
					None => Value::Nil,
				};
				arguments.push(val);
			}

			// Ensure callee is callable (user-defined or native)
			match callee_val {
				Some(Value::Function(func_rc)) => {
					let func = func_rc.as_ref();
					// arity check
					if arguments.len() != func.arity() {
						return Err(RuntimeError::new(expr.paren.clone(), &format!("Expected {} arguments but got {}.", func.arity(), arguments.len())));
					}
					// Call the function
					return func.call(self, &arguments);
				}
				Some(Value::Native(native_rc)) => {
					// arity check
					if arguments.len() != native_rc.arity() {
						return Err(RuntimeError::new(expr.paren.clone(), &format!("Expected {} arguments but got {}.", native_rc.arity(), arguments.len())));
					}
					return native_rc.call(self, &arguments);
				}
				_ => {
					return Err(RuntimeError::new(expr.paren.clone(), "Can only call functions and classes."));
				}
			}
		}
}
impl StmtVisitor<Result<(), RuntimeError>> for Interpreter {
	fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
		// Evaluate and discard the value
		let _ = self.evaluate(expr)?;
		Ok(())
	}

	fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
		let val = self.evaluate(expr)?;
		println!("{}", self.stringify(&val));
		Ok(())
	}
	fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), RuntimeError> {
		let value = match initializer {
			Some(expr) => self.evaluate(expr)?,
			None => None,
		};
		self.environment.borrow_mut().define(&name.lexeme, value);
		Ok(())
	}

	fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) -> Result<(), RuntimeError> {
		// Wrap the parsed function declaration into a runtime LoxFunction object
		let decl = Stmt::Function { name: name.clone(), params: params.clone(), body: body.clone() };
		let func = crate::interpret::lox_function::LoxFunction::new(decl);
		let rc = Rc::new(func);
		self.environment.borrow_mut().define(&name.lexeme, Some(Value::Function(rc)));
		Ok(())
	}

	fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Box<Stmt>, else_branch: &Option<Box<Stmt>>) -> Result<(), RuntimeError> {
		let cond_val = self.evaluate(condition)?;
		if Interpreter::is_truthy(&cond_val) {
			self.execute(&*then_branch)?;
		} else if let Some(eb) = else_branch {
			self.execute(&**eb)?;
		}
		Ok(())
	}

	fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> Result<(), RuntimeError> {
		loop {
			let cond_val = self.evaluate(condition)?;
			if !Interpreter::is_truthy(&cond_val) {
				break;
			}
			self.execute(&*body)?;
		}
		Ok(())
	}

	fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), RuntimeError> {
		// Create a new environment that encloses the current one and execute the block
		let new_env = Rc::new(RefCell::new(Environment::new_enclosing(self.environment.clone())));
		self.execute_block(statements, new_env)
	}
}

impl Interpreter {
	fn evaluate(&mut self, expr: &Expr) -> Result<Option<Value>, RuntimeError> {
		expr.accept(self)
	}

	pub(crate) fn execute_block(&mut self, statements: &Vec<Stmt>, env: Rc<RefCell<Environment>>) -> Result<(), RuntimeError> {
		let previous = self.environment.clone();
		self.environment = env;
		let result = (|| -> Result<(), RuntimeError> {
			for stmt in statements {
				self.execute(stmt)?;
			}
			Ok(())
		})();
		self.environment = previous;
		result
	}

	fn is_truthy(val: &Option<Value>) -> bool {
		match val {
			None => false,
			Some(Value::Bool(b)) => *b,
			Some(Value::Nil) => false,
			_ => true,
		}
	}

	fn is_equal(a: &Option<Value>, b: &Option<Value>) -> bool {
		match (a, b) {
			(None, None) => true,
			(None, Some(_)) | (Some(_), None) => false,
			(Some(Value::Nil), Some(Value::Nil)) => true,
			(Some(Value::Number(x)), Some(Value::Number(y))) => x == y,
			(Some(Value::Str(x)), Some(Value::Str(y))) => x == y,
			(Some(Value::Bool(x)), Some(Value::Bool(y))) => x == y,
			(Some(Value::Function(f1)), Some(Value::Function(f2))) => std::rc::Rc::ptr_eq(f1, f2),
			_ => false,
		}
	}

	fn check_number_operand(&self, operator: &Token, operand: &Option<Value>) -> Result<(), RuntimeError> {
		match operand {
			Some(Value::Number(_)) => Ok(()),
			_ => Err(RuntimeError::new(operator.clone(), "Operand must be a number.")),
		}
	}

	fn check_number_operands(&self, operator: &Token, left: &Option<Value>, right: &Option<Value>) -> Result<(), RuntimeError> {
		match (left, right) {
			(Some(Value::Number(_)), Some(Value::Number(_))) => Ok(()),
			_ => Err(RuntimeError::new(operator.clone(), "Operands must be numbers.")),
		}
	}
}

