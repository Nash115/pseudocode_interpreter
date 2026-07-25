use crate::frontend::ast::{Expr, Stmt};
use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter;
use crate::runtime::values::RuntimeVal;

pub fn eval_program(
    body: Vec<Stmt>,
    env: &mut Environment,
) -> Result<RuntimeVal, InterpreterError> {
    let mut last_evaluated = RuntimeVal::Null;
    for statement in body {
        last_evaluated = interpreter::evaluate(statement, env)?;
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
