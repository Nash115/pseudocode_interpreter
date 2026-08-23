use crate::frontend::ast::{Expr, Stmt};
use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::Environment;
use crate::runtime::eval::expressions;
use crate::runtime::interpreter;
use crate::runtime::values::RuntimeVal;

pub fn eval_program(
    body: Vec<Stmt>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut last_evaluated = RuntimeVal::Null;
    for statement in body {
        last_evaluated = interpreter::evaluate(statement, env)?;
        if let RuntimeVal::ReturnValue(_) = last_evaluated {
            return Err(InterpreterError::UnexpectedReturn);
        }
    }
    Ok(last_evaluated)
}

pub fn eval_var_declaration(
    constant: bool,
    identifier: String,
    value: Option<Expr>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let val = match value {
        Some(v) => interpreter::evaluate(Stmt::ExprStmt(v), env)?,
        None => RuntimeVal::Null,
    };
    Ok(env.declare_var(identifier, val, constant)?)
}

pub fn eval_fn_declaration(
    name: String,
    parameters: Vec<String>,
    body: Vec<Stmt>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let f = RuntimeVal::Fn {
        name: name.clone(),
        parameters: parameters,
        declaration_env: env.this_scope(),
        body,
    };

    Ok(env.declare_var(name, f, true)?)
}

pub fn eval_condition(
    test: Expr,
    body: Vec<Stmt>,
    alternate: Option<Vec<Stmt>>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut last_evaluated = RuntimeVal::Null;
    let test_val = interpreter::evaluate(Stmt::ExprStmt(test), env)?;
    if expressions::eval_val_as_boolean(test_val) {
        let current_scope = env.this_scope();
        let previous_scope = env.push_scope(Some(current_scope));
        for statement in body {
            last_evaluated = interpreter::evaluate(statement, env)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                env.pop_scope(previous_scope);
                return Ok(last_evaluated);
            }
        }
        env.pop_scope(previous_scope);
    } else if let Some(alt) = alternate {
        let current_scope = env.this_scope();
        let previous_scope = env.push_scope(Some(current_scope));
        for statement in alt {
            last_evaluated = interpreter::evaluate(statement, env)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                env.pop_scope(previous_scope);
                return Ok(last_evaluated);
            }
        }
        env.pop_scope(previous_scope);
    }
    Ok(last_evaluated)
}

pub fn eval_while_loop(
    test: Expr,
    body: Vec<Stmt>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut last_evaluated = RuntimeVal::Null;
    while expressions::eval_val_as_boolean(interpreter::evaluate(
        Stmt::ExprStmt(test.clone()),
        env,
    )?) {
        let current_scope = env.this_scope();
        let previous_scope = env.push_scope(Some(current_scope));
        let body_clone = body.clone();
        for statement in body_clone {
            last_evaluated = interpreter::evaluate(statement, env)?;
            if let RuntimeVal::ReturnValue(_) = last_evaluated {
                env.pop_scope(previous_scope);
                return Ok(last_evaluated);
            }
        }
        env.pop_scope(previous_scope);
    }
    Ok(last_evaluated)
}
