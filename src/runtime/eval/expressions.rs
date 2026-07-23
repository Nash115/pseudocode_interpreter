use std::cell::RefCell;
use std::collections::HashMap;
use std::process::exit;
use std::rc::Rc;

use crate::frontend::ast::{Expr, ObjectProperty, Stmt};
use crate::runtime::environment::Environment;
use crate::runtime::values::RuntimeVal::{self, Object};

use crate::runtime::interpreter::{self, evaluate};

fn eval_numeric_binary_expr(lhs: f64, rhs: f64, operator: String) -> RuntimeVal {
    let result: f64 = match operator.as_str() {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => lhs / rhs,
        "%" => lhs % rhs,
        _ => {
            println!("Binop eval impossible : unknown operator '{:?}'.", operator);
            exit(1);
        }
    };
    RuntimeVal::Number(result)
}

pub fn eval_binary_expr(
    left: Box<Expr>,
    right: Box<Expr>,
    operator: String,
    env: &mut Environment,
) -> RuntimeVal {
    let lhs = interpreter::evaluate(Stmt::ExprStmt(*left), env);
    let rhs = interpreter::evaluate(Stmt::ExprStmt(*right), env);

    if let RuntimeVal::Number(lhsv) = lhs
        && let RuntimeVal::Number(rhsv) = rhs
    {
        return eval_numeric_binary_expr(lhsv, rhsv, operator);
    }

    RuntimeVal::Null
}

pub fn eval_identifier(identifier: String, env: &mut Environment) -> RuntimeVal {
    env.lookup_var(&identifier)
}

pub fn eval_assignment(assigne: Box<Expr>, value: Box<Expr>, env: &mut Environment) -> RuntimeVal {
    match *assigne {
        Expr::Identifier(i) => {
            let v = interpreter::evaluate(Stmt::ExprStmt((*value).clone()), env);
            env.assign_var(i, v)
        }
        Expr::MemberExpr {
            object,
            property,
            computed,
        } => {
            let obj = evaluate(Stmt::ExprStmt((*object).clone()), env);

            if let RuntimeVal::Object(map) = obj {
                let key = get_member_key(&property, computed, env);
                let val = evaluate(Stmt::ExprStmt((*value).clone()), env);
                map.borrow_mut().insert(key, val.clone());
                return val;
            } else {
                println!("Assignment error : {} is not an object", obj);
                exit(1);
            }
        }
        expr => {
            println!("Incorrect assignment found unexpected {:?}", expr);
            exit(1);
        }
    }
}

pub fn eval_object_expr(properties: Vec<ObjectProperty>, env: &mut Environment) -> RuntimeVal {
    let mut object_properties = HashMap::new();

    for prop in properties {
        let key = prop.key;
        let value = prop.value;

        let runtime_val = match value {
            Some(val) => evaluate(Stmt::ExprStmt(val), env),
            None => env.lookup_var(&key),
        };

        object_properties.insert(key, runtime_val);
    }

    Object(Rc::new(RefCell::new(object_properties)))
}

fn get_member_key(property: &Expr, computed: bool, env: &mut Environment) -> String {
    if !computed {
        match property {
            Expr::Identifier(ident) => ident.clone(),
            _ => {
                println!("Access to an uncomputed property requires an identifier.");
                exit(1);
            }
        }
    } else {
        let evaluated_prop = evaluate(Stmt::ExprStmt(property.clone()), env);
        match evaluated_prop {
            RuntimeVal::Number(n) => n.to_string(),
            _ => {
                println!("Invalid type for object key : {:?}", evaluated_prop);
                exit(1);
            }
        }
    }
}

pub fn eval_member_expr(
    object: Box<Expr>,
    property: Box<Expr>,
    computed: bool,
    env: &mut Environment,
) -> RuntimeVal {
    let obj = evaluate(Stmt::ExprStmt((*object).clone()), env);
    match obj {
        RuntimeVal::Object(map) => {
            let key = get_member_key(&property, computed, env);
            map.borrow().get(&key).cloned().unwrap_or(RuntimeVal::Null)
        }
        _ => {
            println!(
                "Impossible to acces a property of {:?} : not an object.",
                obj
            );
            exit(1);
        }
    }
}

pub fn eval_call_expr(args: Vec<Expr>, caller: Box<Expr>, env: &mut Environment) -> RuntimeVal {
    let mut evaluated_args = Vec::new();
    for arg in args {
        evaluated_args.push(evaluate(Stmt::ExprStmt(arg), env))
    }
    let f = evaluate(Stmt::ExprStmt(*caller), env);
    match f {
        RuntimeVal::NativeFn(call) => call(evaluated_args, env),
        v => {
            println!("Cannot call a {} : not a function", v);
            exit(1);
        }
    }
}
