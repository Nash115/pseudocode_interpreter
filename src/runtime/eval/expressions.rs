use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::frontend::ast::ObjectProperty;
use crate::frontend::ast::{Expr::*, Stmt::*};
use crate::frontend::ast::{ExprNode, StmtNode};
use crate::frontend::errors::{
    InterpreterError::{self, *},
    RuntimeError,
};
use crate::frontend::span::{Position, Span};
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::evaluate;
use crate::runtime::values::RuntimeVal::{self, Object};

fn eval_numeric_binary_expr(
    lhs: f64,
    rhs: f64,
    span_left: Span,
    span_right: Span,
    operator: String,
) -> Result<RuntimeVal, RuntimeError> {
    let result: f64 = match operator.as_str() {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => {
            if rhs == 0.0 {
                return Err(InterpreterError::with_span(DivBy0, span_right));
            } else {
                lhs / rhs
            }
        }
        "%" => lhs % rhs,
        _ => {
            return Err(InterpreterError::with_span(
                UnknownBinaryOperator(operator),
                span_left.merge(&span_right),
            ));
        }
    };
    Ok(RuntimeVal::Number(result))
}

fn eval_string_binary_expr(
    lhs: String,
    rhs: String,
    span_left: Span,
    span_right: Span,
    operator: String,
) -> Result<RuntimeVal, RuntimeError> {
    let result: String = match operator.as_str() {
        "+" => format!("{}{}", lhs, rhs),
        "-" | "*" | "/" | "%" => {
            return Err(InterpreterError::with_span(
                UnpermittedBinaryOperation { lhs, rhs, operator },
                span_left.merge(&span_right),
            ));
        }
        _ => {
            return Err(InterpreterError::with_span(
                UnknownBinaryOperator(operator),
                span_left.merge(&span_right),
            ));
        }
    };
    Ok(RuntimeVal::String(result))
}

fn eval_list_binary_expr(
    lhs: Vec<RuntimeVal>,
    rhs: Vec<RuntimeVal>,
    span_left: Span,
    span_right: Span,
    operator: String,
) -> Result<RuntimeVal, RuntimeError> {
    let result: Vec<RuntimeVal> = match operator.as_str() {
        "+" => {
            let mut t = Vec::new();
            for v in lhs.clone() {
                t.push(v);
            }
            for v in rhs {
                t.push(v);
            }
            t
        }
        "-" | "*" | "/" | "%" => {
            return Err(InterpreterError::with_span(
                UnpermittedBinaryOperation {
                    lhs: format!("{:?}", lhs),
                    rhs: format!("{:?}", rhs),
                    operator,
                },
                span_left.merge(&span_right),
            ));
        }
        _ => {
            return Err(InterpreterError::with_span(
                UnknownBinaryOperator(operator),
                span_left.merge(&span_right),
            ));
        }
    };
    Ok(RuntimeVal::List(Rc::new(RefCell::new(result))))
}

fn eval_val_as_number(e: RuntimeVal, span: Span) -> Result<f64, RuntimeError> {
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
        RuntimeVal::ReturnValue(v) => eval_val_as_number(*v, span),
        v => Err(InterpreterError::with_span(NumberInterpretation(v), span)),
    }
}

pub fn eval_binary_expr(
    left: Box<ExprNode>,
    right: Box<ExprNode>,
    operator: String,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let lhs = evaluate(StmtNode::new(ExprStmt(*left.clone()), left.span), env)?;
    let rhs = evaluate(StmtNode::new(ExprStmt(*right.clone()), right.span), env)?;

    if let (RuntimeVal::String(ls), RuntimeVal::String(rs)) = (lhs.clone(), rhs.clone()) {
        return Ok(eval_string_binary_expr(
            ls, rs, left.span, right.span, operator,
        )?);
    }
    if let (RuntimeVal::List(ls), RuntimeVal::List(rs)) = (lhs.clone(), rhs.clone()) {
        let ll = ls.borrow().clone();
        let rl = rs.borrow().clone();
        return Ok(eval_list_binary_expr(
            ll, rl, left.span, right.span, operator,
        )?);
    }

    let lhsv = eval_val_as_number(lhs, left.span)?;
    let rhsv = eval_val_as_number(rhs, right.span)?;

    Ok(eval_numeric_binary_expr(
        lhsv, rhsv, left.span, right.span, operator,
    )?)
}

pub fn eval_val_as_boolean(e: RuntimeVal) -> bool {
    match e {
        RuntimeVal::Null => false,
        RuntimeVal::Number(n) => !(n == 0.0),
        RuntimeVal::String(s) => !s.is_empty(),
        RuntimeVal::Boolean(b) => b,
        RuntimeVal::Object(map) => {
            let borrowed = map.borrow();
            !borrowed.is_empty()
        }
        RuntimeVal::List(v) => {
            let borrowed = v.borrow();
            !borrowed.is_empty()
        }
        RuntimeVal::NativeFn(_) => true,
        RuntimeVal::Fn { .. } => true,
        RuntimeVal::ReturnValue(v) => eval_val_as_boolean(*v),
    }
}

pub fn eval_unary_expr(
    right: Box<ExprNode>,
    operator: String,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let rhs = evaluate(StmtNode::new(ExprStmt(*right.clone()), right.span), env)?;

    match operator.as_str() {
        "!" => Ok(RuntimeVal::Boolean(!eval_val_as_boolean(rhs))),
        _ => {
            return Err(InterpreterError::with_span(
                UnknownUnaryOperator(operator),
                Span {
                    start: Position {
                        line: right.span.start.line,
                        col: right.span.start.col - 1,
                    },
                    end: Position {
                        line: right.span.start.line,
                        col: right.span.start.col - 1,
                    },
                },
            ));
        }
    }
}

pub fn eval_logical_expr(
    left: Box<ExprNode>,
    right: Box<ExprNode>,
    operator: String,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let lhs = evaluate(StmtNode::new(ExprStmt(*left.clone()), left.span), env)?;
    let rhs = evaluate(StmtNode::new(ExprStmt(*right.clone()), right.span), env)?;

    match operator.as_str() {
        "||" => Ok(RuntimeVal::Boolean(
            eval_val_as_boolean(lhs) || eval_val_as_boolean(rhs),
        )),
        "&&" => Ok(RuntimeVal::Boolean(
            eval_val_as_boolean(lhs) && eval_val_as_boolean(rhs),
        )),
        "==" => Ok(RuntimeVal::Boolean(lhs == rhs)),
        "!=" => Ok(RuntimeVal::Boolean(lhs != rhs)),
        "<=" => Ok(RuntimeVal::Boolean(
            eval_val_as_number(lhs, left.span)? <= eval_val_as_number(rhs, right.span)?,
        )),
        "<" => Ok(RuntimeVal::Boolean(
            eval_val_as_number(lhs, left.span)? < eval_val_as_number(rhs, right.span)?,
        )),
        ">=" => Ok(RuntimeVal::Boolean(
            eval_val_as_number(lhs, left.span)? >= eval_val_as_number(rhs, right.span)?,
        )),
        ">" => Ok(RuntimeVal::Boolean(
            eval_val_as_number(lhs, left.span)? > eval_val_as_number(rhs, right.span)?,
        )),
        _ => {
            return Err(InterpreterError::with_span(
                UnknownLogicalOperator(operator),
                left.span.merge(&right.span),
            ));
        }
    }
}

pub fn eval_identifier(
    identifier: String,
    span: Span,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    env.borrow()
        .lookup_var(&identifier)
        .map_err(|err| err.with_span(span))
}

pub fn eval_assignment(
    assigne: Box<ExprNode>,
    value: Box<ExprNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match assigne.clone().node {
        Identifier(i) => {
            let v = evaluate(StmtNode::new(ExprStmt((*value).clone()), value.span), env)?;
            env.borrow_mut()
                .assign_var(i, v)
                .map_err(|err| err.with_span(assigne.span.merge(&value.span)))
        }
        MemberExpr {
            object,
            property,
            computed,
        } => {
            let obj = evaluate(StmtNode::new(ExprStmt((*object).clone()), object.span), env)?;

            if let RuntimeVal::Object(map) = obj {
                let key = get_member_key(&property, computed, env)?;
                let val = evaluate(StmtNode::new(ExprStmt((*value).clone()), value.span), env)?;
                map.borrow_mut().insert(key, val.clone());
                return Ok(val);
            } else if let RuntimeVal::List(v) = obj {
                let key = get_member_key(&property, computed, env)?;
                let val = evaluate(StmtNode::new(ExprStmt((*value).clone()), value.span), env)?;
                let index: i64 = match key.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(InterpreterError::with_span(
                            InvalidIndex(key),
                            property.span,
                        ));
                    }
                };
                let mut borrowed = v.borrow_mut();
                if index < -(borrowed.len() as i64) {
                    return Err(InterpreterError::with_span(
                        OutOfBounds(index),
                        property.span,
                    ));
                }
                let mut uindex = index as usize;
                if index < 0 {
                    uindex = borrowed.len() - ((-1 * index) as usize);
                }
                if uindex >= borrowed.len() {
                    return Err(InterpreterError::with_span(
                        OutOfBounds(index),
                        property.span,
                    ));
                }
                borrowed[uindex] = val.clone();
                return Ok(val);
            } else {
                return Err(InterpreterError::with_span(
                    MemberNotAccessible {
                        action: String::from("Assignment"),
                        value: obj,
                    },
                    value.span,
                ));
            }
        }
        expr => {
            return Err(InterpreterError::with_span(
                Assignment(expr),
                assigne.span.merge(&value.span),
            ));
        }
    }
}

pub fn eval_object_expr(
    properties: Vec<ObjectProperty>,
    span: Span,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut object_properties = HashMap::new();

    for prop in properties {
        let key = prop.key;
        let value = prop.value;

        let runtime_val = match value {
            Some(val) => evaluate(StmtNode::new(ExprStmt(val.clone()), val.span), env)?,
            None => env
                .borrow()
                .lookup_var(&key)
                .map_err(|err| err.with_span(span))?,
        };

        object_properties.insert(key, runtime_val);
    }

    Ok(Object(Rc::new(RefCell::new(object_properties))))
}

pub fn eval_list_expr(
    values: Vec<ExprNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut list_values = Vec::new();

    for value in values {
        let runtime_val = evaluate(StmtNode::new(ExprStmt(value.clone()), value.span), env)?;
        list_values.push(runtime_val);
    }

    Ok(RuntimeVal::List(Rc::new(RefCell::new(list_values))))
}

fn get_member_key(
    property: &ExprNode,
    computed: bool,
    env: &Rc<RefCell<Environment>>,
) -> Result<String, RuntimeError> {
    if !computed {
        match property.clone().node {
            Identifier(ident) => Ok(ident.clone()),
            p => Err(InterpreterError::with_span(
                ObjectKeyUncomputedNotIdentifier(p.clone()),
                property.span,
            )),
        }
    } else {
        let evaluated_prop = evaluate(
            StmtNode::new(ExprStmt(property.clone()), property.span),
            env,
        )?;
        match evaluated_prop {
            RuntimeVal::Number(n) => Ok(n.to_string()),
            RuntimeVal::String(s) => Ok(s),
            v => Err(InterpreterError::with_span(
                ObjectKeyComputedType(v),
                property.span,
            )),
        }
    }
}

pub fn eval_member_expr(
    object: Box<ExprNode>,
    property: Box<ExprNode>,
    computed: bool,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let obj = evaluate(StmtNode::new(ExprStmt((*object).clone()), object.span), env)?;
    match obj {
        RuntimeVal::Object(map) => {
            let key = get_member_key(&property, computed, env)?;
            Ok(map.borrow().get(&key).cloned().unwrap_or(RuntimeVal::Null))
        }
        RuntimeVal::List(v) => {
            let key = get_member_key(&property, computed, env)?;
            let index: i64 = match key.parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(InterpreterError::with_span(
                        InvalidIndex(key),
                        property.span,
                    ));
                }
            };
            let borrowed = v.borrow_mut();
            if index < -(borrowed.len() as i64) {
                return Err(InterpreterError::with_span(
                    OutOfBounds(index),
                    property.span,
                ));
            }
            let mut uindex = index as usize;
            if index < 0 {
                uindex = borrowed.len() - ((-1 * index) as usize);
            }
            if uindex >= borrowed.len() {
                return Err(InterpreterError::with_span(
                    OutOfBounds(index),
                    property.span,
                ));
            }
            return Ok(borrowed[uindex].clone());
        }
        _ => Err(InterpreterError::with_span(
            MemberNotAccessible {
                action: String::from("Key access"),
                value: obj,
            },
            object.span.merge(&property.span),
        )),
    }
}

pub fn eval_call_expr(
    args: Vec<ExprNode>,
    caller: Box<ExprNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut evaluated_args = Vec::new();
    let mut args_span = Span::null();
    for arg in args {
        args_span = args_span.merge(&arg.span);
        evaluated_args.push(evaluate(
            StmtNode::new(ExprStmt(arg.clone()), arg.span),
            env,
        )?)
    }
    let f = evaluate(StmtNode::new(ExprStmt(*caller.clone()), caller.span), env)?;
    match f {
        RuntimeVal::NativeFn(call) => {
            call(evaluated_args, env).map_err(|err| err.with_span(caller.span.merge(&args_span)))
        }
        RuntimeVal::Fn {
            name,
            parameters,
            declaration_env,
            body,
        } => {
            let params_len = parameters.len();
            if params_len != evaluated_args.len() {
                return Err(InterpreterError::with_span(
                    FunctionCallArguments {
                        name,
                        expected: parameters.len(),
                        given: evaluated_args.len(),
                    },
                    args_span,
                ));
            }

            let scope = Rc::new(RefCell::new(Environment::new(Some(
                declaration_env.clone(),
            ))));

            for i in 0..params_len {
                scope
                    .borrow_mut()
                    .declare_var(parameters[i].clone(), evaluated_args[i].clone(), false)
                    .map_err(|err| err.with_span(caller.span.merge(&args_span)))?;
            }

            let mut result: RuntimeVal = RuntimeVal::Null;
            for s in body {
                result = evaluate(s, &scope)?;
                if let RuntimeVal::ReturnValue(val) = result {
                    result = *val;
                    break;
                }
            }

            Ok(result)
        }
        v => Err(InterpreterError::with_span(
            NotAFunction {
                action: String::from("call"),
                value: v,
            },
            caller.span,
        )),
    }
}
