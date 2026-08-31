use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::frontend::ast::StmtNode;
use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::Environment;

pub type FunctionCall =
    fn(Vec<RuntimeVal>, &Rc<RefCell<Environment>>) -> Result<RuntimeVal, InterpreterError>;

#[derive(Clone)]
pub enum RuntimeVal {
    Null,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(Rc<RefCell<HashMap<String, RuntimeVal>>>),
    List(Rc<RefCell<Vec<RuntimeVal>>>),
    NativeFn(FunctionCall),
    Fn {
        name: String,
        parameters: Vec<String>,
        declaration_env: Rc<RefCell<Environment>>,
        body: Vec<StmtNode>,
    },
    ReturnValue(Box<RuntimeVal>),
}
impl PartialEq for RuntimeVal {
    fn eq(&self, other: &RuntimeVal) -> bool {
        match (self, other) {
            (RuntimeVal::Null, RuntimeVal::Null) => true,
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => n1 == n2,
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => s1 == s2,
            (RuntimeVal::Boolean(b1), RuntimeVal::Boolean(b2)) => b1 == b2,
            (RuntimeVal::ReturnValue(v1), RuntimeVal::ReturnValue(v2)) => v1 == v2,
            (RuntimeVal::Object(o1), RuntimeVal::Object(o2)) => {
                if Rc::ptr_eq(o1, o2) {
                    return true;
                }
                *o1.borrow() == *o2.borrow()
            }
            (
                RuntimeVal::Fn {
                    name: name1,
                    parameters: parameters1,
                    declaration_env: declaration_env1,
                    body: body1,
                },
                RuntimeVal::Fn {
                    name: name2,
                    parameters: parameters2,
                    declaration_env: declaration_env2,
                    body: body2,
                },
            ) => {
                name1 == name2
                    && parameters1 == parameters2
                    && Rc::ptr_eq(declaration_env1, declaration_env2)
                    && body1 == body2
            }
            _ => false,
        }
    }
}

impl std::fmt::Debug for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeVal::Null => write!(f, "null"),
            RuntimeVal::Number(n) => write!(f, "{}", n),
            RuntimeVal::String(s) => write!(f, "{}", s),
            RuntimeVal::Boolean(b) => write!(f, "{}", b),
            RuntimeVal::Object(o) => write!(f, "{:?}", o),
            RuntimeVal::List(v) => write!(f, "{:?}", v),
            RuntimeVal::NativeFn(_) => write!(f, "[Native Function]"),
            RuntimeVal::Fn { .. } => write!(f, "[Function]"),
            RuntimeVal::ReturnValue(v) => write!(f, "{:?}", v),
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
            RuntimeVal::List(v) => {
                let borrowed = v.borrow();
                let mut tab: Vec<String> = Vec::new();
                for val in borrowed.iter() {
                    tab.push(format!("{}", val));
                }
                write!(f, "[{}]", tab.join(", "))
            }
            RuntimeVal::NativeFn(_) => write!(f, "[Native Function]"),
            RuntimeVal::Fn { .. } => write!(f, "[Function]"),
            RuntimeVal::ReturnValue(v) => write!(f, "{}", v),
        }
    }
}
