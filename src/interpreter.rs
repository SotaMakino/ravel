use crate::env::Env;
use crate::parser::ast::AstNode;
use crate::parser::ast::BinaryOp;
use crate::parser::ast::UnaryOp;
use crate::value::Value;
use std::collections::HashMap;

pub struct Interpreter<'a> {
    env: &'a mut Env,
}

impl<'a> Interpreter<'a> {
    pub fn new(env: &'a mut Env) -> Self {
        Self { env }
    }

    pub fn execute(&mut self, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::NumberLit(n) => Ok(Value::Number(*n)),
            AstNode::StrLit(s) => Ok(Value::Str(s.clone())),
            AstNode::BoolLit(b) => Ok(Value::Bool(*b)),
            AstNode::NullLit => Ok(Value::Null),
            AstNode::UndefinedLit => Ok(Value::Undefined),
            AstNode::Ident(name) => self
                .env
                .get(name)
                .ok_or_else(|| format!("Undefined variable: {}", name)),

            AstNode::BinaryOp { left, op, right } => {
                let l = self.execute(left)?;
                let r = self.execute(right)?;
                self.eval_binary(op, &l, &r)
            }

            AstNode::UnaryOp { op, operand } => {
                let v = self.execute(operand)?;
                self.eval_unary(op, &v)
            }

            AstNode::VarDecl {
                name,
                kind: _,
                init,
            } => {
                let value = match init {
                    Some(i) => self.execute(i)?,
                    None => Value::Undefined,
                };
                self.env.define(name, value);
                Ok(Value::Undefined)
            }

            AstNode::Assign { name, value } => {
                let v = self.execute(value)?;
                self.env.set(name, v.clone())?;
                Ok(v)
            }

            AstNode::Block(stmts) => {
                let mut result = Value::Undefined;
                for stmt in stmts {
                    result = self.execute(stmt)?;
                }
                Ok(result)
            }

            AstNode::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let c = self.execute(cond)?;
                if self.is_truthy(&c) {
                    self.execute(then_branch)
                } else if let Some(e) = else_branch {
                    self.execute(e)
                } else {
                    Ok(Value::Undefined)
                }
            }

            AstNode::While { cond, body } => {
                loop {
                    let c = self.execute(cond)?;
                    if !self.is_truthy(&c) {
                        break;
                    }
                    self.execute(body)?;
                }
                Ok(Value::Undefined)
            }

            AstNode::For {
                init,
                cond,
                update,
                body,
            } => {
                if let Some(i) = init {
                    self.execute(i)?;
                }
                loop {
                    if let Some(c) = cond {
                        let v = self.execute(c)?;
                        if !self.is_truthy(&v) {
                            break;
                        }
                    } else {
                        break;
                    }
                    self.execute(body)?;
                    if let Some(u) = update {
                        self.execute(u)?;
                    }
                }
                Ok(Value::Undefined)
            }

            AstNode::FuncDecl { name, params, body } => {
                let func = Value::Func {
                    params: params.clone(),
                    body: (**body).clone(),
                };
                self.env.define(name, func);
                Ok(Value::Undefined)
            }

            AstNode::Call { callee, args } => {
                let func = self.execute(callee)?;
                let arg_values: Result<Vec<Value>, String> =
                    args.iter().map(|a| self.execute(a)).collect();
                let arg_values = arg_values?;
                self.call_func(&func, &arg_values)
            }

            AstNode::Return(expr) => match expr {
                Some(e) => self.execute(e),
                None => Ok(Value::Undefined),
            },

            AstNode::MemberAccess { object, property } => {
                let obj = self.execute(object)?;
                match obj {
                    Value::Object(map) => map
                        .get(property)
                        .cloned()
                        .ok_or_else(|| format!("Property '{}' not found", property)),
                    _ => Err("Cannot access property of non-object".into()),
                }
            }

            AstNode::IndexAccess { object, index } => {
                let obj = self.execute(object)?;
                let idx = self.execute(index)?;
                match (obj, idx) {
                    (Value::Array(arr), Value::Number(n)) => {
                        let i = n as usize;
                        if i < arr.len() {
                            Ok(arr[i].clone())
                        } else {
                            Ok(Value::Undefined)
                        }
                    }
                    (Value::Object(map), Value::Str(s)) => map
                        .get(&s)
                        .cloned()
                        .ok_or_else(|| format!("Property '{}' not found", s)),
                    _ => Err("Cannot index into this value".into()),
                }
            }

            AstNode::ObjectLit(entries) => {
                let mut map = HashMap::new();
                for (k, v) in entries {
                    map.insert(k.clone(), self.execute(v)?);
                }
                Ok(Value::Object(map))
            }

            AstNode::ArrayLit(items) => {
                let arr: Result<Vec<Value>, String> =
                    items.iter().map(|i| self.execute(i)).collect();
                Ok(Value::Array(arr?))
            }

            AstNode::Expr(e) => self.execute(e),
        }
    }

    fn eval_binary(&self, op: &BinaryOp, left: &Value, right: &Value) -> Result<Value, String> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::Str(a), b) => Ok(Value::Str(format!("{}{}", a, b))),
                (a, Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                _ => Err(format!("Cannot add {:?} and {:?}", left, right)),
            },
            BinaryOp::Sub => self.binary_num(left, right, |a, b| a - b),
            BinaryOp::Mul => self.binary_num(left, right, |a, b| a * b),
            BinaryOp::Div => self.binary_num(left, right, |a, b| a / b),
            BinaryOp::Mod => self.binary_num(left, right, |a, b| a % b),
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::StrictEq => Ok(Value::Bool(left == right)),
            BinaryOp::NotEq => Ok(Value::Bool(left != right)),
            BinaryOp::StrictNotEq => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.binary_cmp(left, right, |a, b| a < b),
            BinaryOp::Gt => self.binary_cmp(left, right, |a, b| a > b),
            BinaryOp::LtEq => self.binary_cmp(left, right, |a, b| a <= b),
            BinaryOp::GtEq => self.binary_cmp(left, right, |a, b| a >= b),
            BinaryOp::And => Ok(Value::Bool(self.is_truthy(left) && self.is_truthy(right))),
            BinaryOp::Or => Ok(Value::Bool(self.is_truthy(left) || self.is_truthy(right))),
        }
    }

    fn eval_unary(&self, op: &UnaryOp, operand: &Value) -> Result<Value, String> {
        match op {
            UnaryOp::Not => Ok(Value::Bool(!self.is_truthy(operand))),
            UnaryOp::Neg => match operand {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err("Cannot negate non-number".into()),
            },
        }
    }

    fn binary_num<F>(&self, left: &Value, right: &Value, f: F) -> Result<Value, String>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(*a, *b))),
            _ => Err(format!(
                "Cannot apply numeric op to {:?} and {:?}",
                left, right
            )),
        }
    }

    fn binary_cmp<F>(&self, left: &Value, right: &Value, f: F) -> Result<Value, String>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(*a, *b))),
            _ => Err(format!("Cannot compare {:?} and {:?}", left, right)),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Null => false,
            Value::Undefined => false,
            _ => true,
        }
    }

    fn call_func(&mut self, func: &Value, args: &[Value]) -> Result<Value, String> {
        match func {
            Value::Func { params, body } => {
                let mut child_env = Env::with_parent(self.env as *mut Env);
                for (param, arg) in params.iter().zip(args.iter()) {
                    child_env.define(param, arg.clone());
                }
                let mut child_interp = Interpreter::new(&mut child_env);
                child_interp.execute(body)
            }
            Value::Builtin(f) => f(args),
            _ => Err("Cannot call non-function".into()),
        }
    }
}
