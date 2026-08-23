use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::*;
use crate::runtime::values::RuntimeVal;

pub fn load_default_variables(env: &mut Environment) -> Result<(), InterpreterError> {
    env.declare_var(
        String::from("PI"),
        RuntimeVal::Number(3.14159265358979323846264338327950288),
        true,
    )?;
    env.declare_var(String::from("true"), RuntimeVal::Boolean(true), true)?;
    env.declare_var(String::from("false"), RuntimeVal::Boolean(false), true)?;
    env.declare_var(String::from("null"), RuntimeVal::Null, true)?;
    Ok(())
}

pub fn load_default_functions(env: &mut Environment) -> Result<(), InterpreterError> {
    env.declare_var(
        String::from("print"),
        RuntimeVal::NativeFn(|_args, _env| {
            let mut i: usize = 0;
            for v in _args.clone() {
                i += 1;
                let space = if i == _args.len() { "" } else { " " };
                print!("{}{}", v, space);
            }
            println!("");
            return Ok(RuntimeVal::Null);
        }),
        true,
    )?;
    env.declare_var(
        String::from("time"),
        RuntimeVal::NativeFn(|_args, _env| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
            Ok(RuntimeVal::Number(since_the_epoch.as_millis() as f64))
        }),
        true,
    )?;
    env.declare_var(
        String::from("len"),
        RuntimeVal::NativeFn(|_args, _env| {
            if _args.len() != 1 {
                return Err(InterpreterError::FunctionCallArguments {
                    name: "len (NATIVE FUNCTION)".to_string(),
                    expected: 1,
                    given: _args.len(),
                });
            }
            match _args[0].clone() {
                RuntimeVal::String(ref s) => Ok(RuntimeVal::Number(s.chars().count() as f64)),
                RuntimeVal::Object(ref o) => Ok(RuntimeVal::Number(o.borrow().len() as f64)),
                RuntimeVal::List(ref v) => Ok(RuntimeVal::Number(v.borrow().len() as f64)),
                e => Err(InterpreterError::NativeFunctionWrongArgument {
                    name: "len".to_string(),
                    index: 1,
                    expected: "String / Object / List".to_string(),
                    given: format!("{}", e),
                }),
            }
        }),
        true,
    )?;
    env.declare_var(
        String::from("push"),
        RuntimeVal::NativeFn(|_args, _env| {
            if _args.len() != 2 {
                return Err(InterpreterError::FunctionCallArguments {
                    name: "push (NATIVE FUNCTION)".to_string(),
                    expected: 1,
                    given: _args.len(),
                });
            }
            if let RuntimeVal::List(ref v) = _args[0] {
                let mut borrowed = v.borrow_mut();
                borrowed.push(_args[1].clone());
                Ok(_args[1].clone())
            } else {
                Err(InterpreterError::NativeFunctionWrongArgument {
                    name: "push".to_string(),
                    index: 1,
                    expected: "List".to_string(),
                    given: format!("{}", _args[0]),
                })
            }
        }),
        true,
    )?;
    env.declare_var(
        String::from("pop"),
        RuntimeVal::NativeFn(|_args, _env| {
            if _args.len() > 2 {
                return Err(InterpreterError::FunctionCallArguments {
                    name: "pop (NATIVE FUNCTION)".to_string(),
                    expected: 2,
                    given: _args.len(),
                });
            }
            if let RuntimeVal::List(ref v) = _args[0] {
                let mut borrowed = v.borrow_mut();
                if borrowed.len() == 0 {
                    return Ok(RuntimeVal::Null);
                }
                let index = if _args.len() == 1 {
                    borrowed.len() - 1
                } else {
                    if let RuntimeVal::Number(n) = _args[1] {
                        n as usize
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "pop".to_string(),
                            index: 0,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                };
                let v = borrowed.remove(index);
                Ok(v)
            } else {
                Err(InterpreterError::NativeFunctionWrongArgument {
                    name: "pop".to_string(),
                    index: 1,
                    expected: "List".to_string(),
                    given: format!("{}", _args[0]),
                })
            }
        }),
        true,
    )?;
    env.declare_var(
        String::from("range"),
        RuntimeVal::NativeFn(|_args, _env| {
            if _args.len() > 3 {
                return Err(InterpreterError::FunctionCallArguments {
                    name: "range (NATIVE FUNCTION)".to_string(),
                    expected: 3,
                    given: _args.len(),
                });
            }
            if _args.len() < 1 {
                return Err(InterpreterError::FunctionCallArguments {
                    name: "range (NATIVE FUNCTION)".to_string(),
                    expected: 1,
                    given: _args.len(),
                });
            }
            let mut start = 0;
            let mut stop = 0;
            let mut step = 1;
            match _args.len() {
                1 => {
                    if let RuntimeVal::Number(n) = _args[0] {
                        stop = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 1,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                }
                2 => {
                    if let RuntimeVal::Number(n) = _args[0] {
                        start = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 1,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                    if let RuntimeVal::Number(n) = _args[1] {
                        stop = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 2,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                }
                3 => {
                    if let RuntimeVal::Number(n) = _args[0] {
                        start = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 1,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                    if let RuntimeVal::Number(n) = _args[1] {
                        stop = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 2,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                    if let RuntimeVal::Number(n) = _args[2] {
                        step = n as i32;
                    } else {
                        return Err(InterpreterError::NativeFunctionWrongArgument {
                            name: "range".to_string(),
                            index: 3,
                            expected: "Number".to_string(),
                            given: format!("{}", _args[0]),
                        });
                    }
                }
                _ => {}
            }
            let mut range: Vec<RuntimeVal> = Vec::new();
            if step == 0 || (start < stop && step < 0) || (start > stop && step > 0) {
                return Ok(RuntimeVal::List(Rc::new(RefCell::new(range))));
            }
            let mut c = start;
            if step > 0 {
                while c < stop {
                    range.push(RuntimeVal::Number(c as f64));
                    c += step;
                }
            } else {
                while c > stop {
                    range.push(RuntimeVal::Number(c as f64));
                    c += step;
                }
            }
            return Ok(RuntimeVal::List(Rc::new(RefCell::new(range))));
        }),
        true,
    )?;
    Ok(())
}

pub fn load_default_aliases(env: &mut Environment) -> Result<(), InterpreterError> {
    env.alias("true", "vrai")?;
    env.alias("false", "faux")?;
    env.alias("null", "nul")?;
    env.alias("print", "affiche")?;
    env.alias("time", "temps")?;
    env.alias("len", "taille")?;
    env.alias("push", "append")?;
    env.alias("push", "empiler")?;
    env.alias("pop", "depiler")?;
    env.alias("range", "plage")?;
    Ok(())
}
