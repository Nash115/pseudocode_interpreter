use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::frontend::ast::Stmt;
use crate::runtime::environment::Environment;

pub type FunctionCall = fn(Vec<RuntimeVal>, &mut Environment) -> RuntimeVal;

#[derive(Clone)]
pub enum RuntimeVal {
    Null,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(Rc<RefCell<HashMap<String, RuntimeVal>>>),
    NativeFn(FunctionCall),
    Fn {
        name: String,
        parameters: Vec<String>,
        declaration_env: usize,
        body: Vec<Stmt>,
    },
}

impl std::fmt::Debug for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeVal::Null => write!(f, "null"),
            RuntimeVal::Number(n) => write!(f, "{}", n),
            RuntimeVal::String(s) => write!(f, "{}", s),
            RuntimeVal::Boolean(b) => write!(f, "{}", b),
            RuntimeVal::Object(o) => write!(f, "{:?}", o),
            RuntimeVal::NativeFn(_) => write!(f, "[Native Function]"),
            RuntimeVal::Fn { .. } => write!(f, "[Function]"),
        }
    }
}

impl std::fmt::Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeVal::Null => write!(f, "null"),
            RuntimeVal::Number(n) => write!(f, "{}", n),
            RuntimeVal::String(s) => write!(f, "{}", s),
            RuntimeVal::Boolean(true) => write!(f, "true"),
            RuntimeVal::Boolean(false) => write!(f, "false"),
            RuntimeVal::Object(map) => {
                let borrowed = map.borrow();
                if borrowed.is_empty() {
                    return write!(f, "{{}}");
                }
                let mut entries = Vec::new();
                for (key, val) in borrowed.iter() {
                    entries.push(format!("{}: {}", key, val));
                }
                return write!(f, "{{{}}}", entries.join(", "));
            }
            RuntimeVal::NativeFn(_) => write!(f, "[Native Function]"),
            RuntimeVal::Fn { .. } => write!(f, "[Function]"),
        }
    }
}
