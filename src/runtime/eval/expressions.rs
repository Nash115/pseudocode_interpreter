use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::frontend::ast::{Expr, ObjectProperty, Stmt};
use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::{self, evaluate};
use crate::runtime::values::RuntimeVal::{self, Object};

fn eval_numeric_binary_expr(
    lhs: f64,
    rhs: f64,
    operator: String,
) -> Result<RuntimeVal, InterpreterError> {
    let result: f64 = match operator.as_str() {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        "%" => lhs % rhs,
        _ => return Err(InterpreterError::UnknownBinaryOperator(operator)),
    };
    Ok(RuntimeVal::Number(result))
}

fn eval_val_as_number(e: RuntimeVal) -> Result<f64, InterpreterError> {
    match e {
        RuntimeVal::Null => Ok(0.0),
        RuntimeVal::Number(n) => Ok(n),
        RuntimeVal::Boolean(b) => {
            if b {
                Ok(1.0)
            } else {
                Ok(0.0)
            }
        }
        v => Err(InterpreterError::NumberInterpretation(v)),
    }
}

pub fn eval_binary_expr(
    left: Box<Expr>,
    right: Box<Expr>,
    operator: String,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let lhs = interpreter::evaluate(Stmt::ExprStmt(*left), env)?;
    let rhs = interpreter::evaluate(Stmt::ExprStmt(*right), env)?;

    let lhsv = eval_val_as_number(lhs)?;
    let rhsv = eval_val_as_number(rhs)?;

    Ok(eval_numeric_binary_expr(lhsv, rhsv, operator)?)
}

fn eval_val_as_boolean(e: RuntimeVal) -> bool {
    match e {
        RuntimeVal::Null => false,
        RuntimeVal::Number(n) => {
            if n == 0.0 {
                false
            } else {
                true
            }
        }
        RuntimeVal::Boolean(b) => b,
        RuntimeVal::Object(_) => true,
        RuntimeVal::NativeFn(_) => true,
        RuntimeVal::Fn { .. } => true,
    }
}

pub fn eval_unary_expr(
    right: Box<Expr>,
    operator: String,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let rhs = interpreter::evaluate(Stmt::ExprStmt(*right), env)?;

    match operator.as_str() {
        "!" => Ok(RuntimeVal::Boolean(!eval_val_as_boolean(rhs))),
        _ => return Err(InterpreterError::UnknownUnaryOperator(operator)),
    }
}

pub fn eval_logical_expr(
    left: Box<Expr>,
    right: Box<Expr>,
    operator: String,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let lhs = interpreter::evaluate(Stmt::ExprStmt(*left), env)?;
    let rhs = interpreter::evaluate(Stmt::ExprStmt(*right), env)?;

    match operator.as_str() {
        "||" => Ok(RuntimeVal::Boolean(
            eval_val_as_boolean(lhs) || eval_val_as_boolean(rhs),
        )),
        "&&" => Ok(RuntimeVal::Boolean(
            eval_val_as_boolean(lhs) && eval_val_as_boolean(rhs),
        )),
        _ => return Err(InterpreterError::UnknownLogicalOperator(operator)),
    }
}

pub fn eval_identifier(
    identifier: String,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    env.lookup_var(&identifier)
}

pub fn eval_assignment(
    assigne: Box<Expr>,
    value: Box<Expr>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    match *assigne {
        Expr::Identifier(i) => {
            let v = interpreter::evaluate(Stmt::ExprStmt((*value).clone()), env)?;
            env.assign_var(i, v)
        }
        Expr::MemberExpr {
            object,
            property,
            computed,
        } => {
            let obj = evaluate(Stmt::ExprStmt((*object).clone()), env)?;

            if let RuntimeVal::Object(map) = obj {
                let key = get_member_key(&property, computed, env)?;
                let val = evaluate(Stmt::ExprStmt((*value).clone()), env)?;
                map.borrow_mut().insert(key, val.clone());
                return Ok(val);
            } else {
                return Err(InterpreterError::NotAnObject {
                    action: String::from("Assignment"),
                    value: obj,
                });
            }
        }
        expr => {
            return Err(InterpreterError::Assignment(expr));
        }
    }
}

pub fn eval_object_expr(
    properties: Vec<ObjectProperty>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut object_properties = HashMap::new();

    for prop in properties {
        let key = prop.key;
        let value = prop.value;

        let runtime_val = match value {
            Some(val) => evaluate(Stmt::ExprStmt(val), env)?,
            None => env.lookup_var(&key)?,
        };

        object_properties.insert(key, runtime_val);
    }

    Ok(Object(Rc::new(RefCell::new(object_properties))))
}

fn get_member_key(
    property: &Expr,
    computed: bool,
    env: &mut Environment,
) -> Result<String, InterpreterError> {
    if !computed {
        match property {
            Expr::Identifier(ident) => Ok(ident.clone()),
            p => Err(InterpreterError::ObjectKeyUncomputedNotIdentifier(
                p.clone(),
            )),
        }
    } else {
        let evaluated_prop = evaluate(Stmt::ExprStmt(property.clone()), env)?;
        match evaluated_prop {
            RuntimeVal::Number(n) => Ok(n.to_string()),
            v => Err(InterpreterError::ObjectKeyComputedType(v)),
        }
    }
}

pub fn eval_member_expr(
    object: Box<Expr>,
    property: Box<Expr>,
    computed: bool,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let obj = evaluate(Stmt::ExprStmt((*object).clone()), env)?;
    match obj {
        RuntimeVal::Object(map) => {
            let key = get_member_key(&property, computed, env)?;
            Ok(map.borrow().get(&key).cloned().unwrap_or(RuntimeVal::Null))
        }
        _ => Err(InterpreterError::NotAnObject {
            action: String::from("Key access"),
            value: obj,
        }),
    }
}

pub fn eval_call_expr(
    args: Vec<Expr>,
    caller: Box<Expr>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut evaluated_args = Vec::new();
    for arg in args {
        evaluated_args.push(evaluate(Stmt::ExprStmt(arg), env)?)
    }
    let f = evaluate(Stmt::ExprStmt(*caller), env)?;
    match f {
        RuntimeVal::NativeFn(call) => Ok(call(evaluated_args, env)),
        RuntimeVal::Fn {
            name,
            parameters,
            declaration_env,
            body,
        } => {
            let params_len = parameters.len();
            if params_len != evaluated_args.len() {
                return Err(InterpreterError::FunctionCallArguments {
                    name,
                    expected: parameters.len(),
                    given: evaluated_args.len(),
                });
            }

            let previous_scope = env.push_scope(Some(declaration_env));

            for i in 0..params_len {
                env.declare_var(parameters[i].clone(), evaluated_args[i].clone(), false)?;
            }

            let mut result: RuntimeVal = RuntimeVal::Null;
            for s in body {
                match s {
                    Stmt::Return(e) => {
                        result = evaluate(Stmt::ExprStmt(e), env)?;
                        env.pop_scope(previous_scope);
                        return Ok(result);
                    }
                    _ => {
                        result = evaluate(s, env)?;
                    }
                }
            }

            env.pop_scope(previous_scope);

            Ok(result)
        }
        v => Err(InterpreterError::NotAFunction {
            action: String::from("call"),
            value: v,
        }),
    }
}
