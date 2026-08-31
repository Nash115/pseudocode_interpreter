use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::ast::Stmt::*;
use crate::frontend::ast::{ExprNode, StmtNode};
use crate::frontend::errors::{
    InterpreterError::{self, *},
    RuntimeError,
};
use crate::frontend::span::Span;
use crate::runtime::environment::Environment;
use crate::runtime::eval::expressions;
use crate::runtime::interpreter::evaluate;
use crate::runtime::values::RuntimeVal;

pub fn eval_program(
    body: Vec<StmtNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut last_evaluated = RuntimeVal::Null;
    for statement in body {
        last_evaluated = evaluate(statement.clone(), env)?;
        if let RuntimeVal::ReturnValue(_) = last_evaluated {
            return Err(InterpreterError::with_span(
                UnexpectedReturn,
                statement.span,
            ));
        }
    }
    Ok(last_evaluated)
}

pub fn eval_var_declaration(
    constant: bool,
    identifier: String,
    value: Option<ExprNode>,
    span: Span,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let val = match value {
        Some(v) => evaluate(StmtNode::new(ExprStmt(v.clone()), v.span), env)?,
        None => RuntimeVal::Null,
    };
    Ok(env
        .borrow_mut()
        .declare_var(identifier, val, constant)
        .map_err(|err| err.with_span(span))?)
}

pub fn eval_fn_declaration(
    name: String,
    parameters: Vec<String>,
    body: Vec<StmtNode>,
    span: Span,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let f = RuntimeVal::Fn {
        name: name.clone(),
        parameters: parameters,
        declaration_env: env.clone(),
        body: body.clone(),
    };

    Ok(env
        .borrow_mut()
        .declare_var(name, f, true)
        .map_err(|err| err.with_span(span))?)
}

pub fn eval_condition(
    test: ExprNode,
    body: Vec<StmtNode>,
    alternate: Option<Vec<StmtNode>>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut last_evaluated = RuntimeVal::Null;
    let test_val = evaluate(StmtNode::new(ExprStmt(test.clone()), test.span), env)?;
    if expressions::eval_val_as_boolean(test_val) {
        let scope = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
        for statement in body {
            last_evaluated = evaluate(statement, &scope)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                return Ok(last_evaluated);
            }
        }
    } else if let Some(alt) = alternate {
        let scope = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
        for statement in alt {
            last_evaluated = evaluate(statement, &scope)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                return Ok(last_evaluated);
            }
        }
    }
    Ok(last_evaluated)
}

pub fn eval_while_loop(
    test: ExprNode,
    body: Vec<StmtNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut last_evaluated = RuntimeVal::Null;
    while expressions::eval_val_as_boolean(evaluate(
        StmtNode::new(ExprStmt(test.clone()), test.span),
        env,
    )?) {
        let scope = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));
        let body_clone = body.clone();
        for statement in body_clone {
            last_evaluated = evaluate(statement, &scope)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                return Ok(last_evaluated);
            }
        }
    }
    Ok(last_evaluated)
}

pub fn eval_for_loop(
    iterable: ExprNode,
    identifier: String,
    body: Vec<StmtNode>,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    let mut last_evaluated = RuntimeVal::Null;
    let mut i = 0;

    let parsed_iterable = evaluate(
        StmtNode::new(ExprStmt(iterable.clone()), iterable.span),
        env,
    )?;
    let iterable_size: usize = match parsed_iterable.clone() {
        RuntimeVal::List(v) => v.borrow().len(),
        RuntimeVal::Object(o) => o.borrow().len(),
        RuntimeVal::String(s) => s.chars().count(),
        v => {
            return Err(InterpreterError::with_span(
                NotIterable(format!("{}", v)),
                iterable.span,
            ));
        }
    };

    while i < iterable_size {
        let scope = Rc::new(RefCell::new(Environment::new(Some(env.clone()))));

        scope
            .borrow_mut()
            .assign_var(
                identifier.clone(),
                match parsed_iterable.clone() {
                    RuntimeVal::List(v) => {
                        let b = v.borrow();
                        b[i].clone()
                    }
                    RuntimeVal::Object(o) => {
                        let b = o.borrow();
                        let mut keys: Vec<String> = b.clone().into_keys().collect();
                        keys.sort();
                        RuntimeVal::String(keys[i].clone())
                    }
                    RuntimeVal::String(s) => RuntimeVal::String(format!(
                        "{}",
                        match s.chars().nth(i) {
                            Some(c) => c,
                            None => 'c',
                        }
                    )),
                    _ => RuntimeVal::Null,
                },
            )
            .map_err(|err| err.with_span(iterable.span))?;

        let body_clone = body.clone();
        for statement in body_clone {
            last_evaluated = evaluate(statement, &scope)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                return Ok(last_evaluated);
            }
        }

        i += 1;
    }
    Ok(last_evaluated)
}
