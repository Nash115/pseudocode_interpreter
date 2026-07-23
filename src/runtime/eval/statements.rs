use crate::frontend::ast::{Expr, Stmt};
use crate::runtime::environment::Environment;
use crate::runtime::values::RuntimeVal;

use crate::runtime::interpreter;

pub fn eval_program(body: Vec<Stmt>, env: &mut Environment) -> RuntimeVal {
    let mut last_evaluated = RuntimeVal::Null;
    for statement in body {
        last_evaluated = interpreter::evaluate(statement, env);
    }
    last_evaluated
}

pub fn eval_var_declaration(
    constant: bool,
    identifier: String,
    value: Option<Expr>,
    env: &mut Environment,
) -> RuntimeVal {
    let val = match value {
        Some(v) => interpreter::evaluate(Stmt::ExprStmt(v), env),
        None => RuntimeVal::Null,
    };
    env.declare_var(identifier, val, constant)
}
