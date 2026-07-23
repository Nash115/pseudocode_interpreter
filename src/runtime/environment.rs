use crate::runtime::values::RuntimeVal;
use std::collections::{HashMap, HashSet};
use std::println;
use std::process::exit;

pub struct Environment {
    scopes: Vec<(HashMap<String, RuntimeVal>, HashSet<String>)>,
}
impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            scopes: vec![(HashMap::new(), HashSet::new())],
        };

        // Define natives variables
        env.declare_var(
            String::from("PI"),
            RuntimeVal::Number(3.14159265358979323846264338327950288),
            true,
        );
        env.declare_var(String::from("vrai"), RuntimeVal::Boolean(true), true);
        env.declare_var(String::from("faux"), RuntimeVal::Boolean(false), true);
        env.declare_var(String::from("nul"), RuntimeVal::Null, true);

        // Define natives functions
        env.declare_var(
            String::from("AFFICHE"),
            RuntimeVal::NativeFn(|_args, _scope| {
                let mut i: usize = 0;
                for v in _args.clone() {
                    i += 1;
                    let space = if i == _args.len() { "" } else { " " };
                    print!("{}{}", v, space);
                }
                println!("");
                return RuntimeVal::Null;
            }),
            true,
        );
        env.declare_var(
            String::from("TEMPS"),
            RuntimeVal::NativeFn(|_args, _env| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let start = SystemTime::now();
                let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
                RuntimeVal::Number(since_the_epoch.as_millis() as f64)
            }),
            true,
        );

        env
    }

    // pub fn push_scope(&mut self) {
    //     self.scopes.push((HashMap::new(), HashSet::new()));
    // }

    // pub fn pop_scope(&mut self) {
    //     self.scopes.pop();
    // }

    pub fn declare_var(
        &mut self,
        varname: String,
        value: RuntimeVal,
        constant: bool,
    ) -> RuntimeVal {
        let current_scope = self.scopes.last_mut().unwrap();

        if current_scope.0.contains_key(&varname) {
            println!(
                "Cannot declare var {} : already defined in this scope.",
                varname
            );
            exit(1);
        }

        current_scope.0.insert(varname.clone(), value.clone());
        if constant {
            current_scope.1.insert(varname);
        }
        value
    }

    pub fn lookup_var(&self, varname: &str) -> RuntimeVal {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.0.get(varname) {
                return value.clone();
            }
        }

        println!("Unable to resolve var '{}'", varname);
        exit(1);
    }

    pub fn assign_var(&mut self, varname: String, value: RuntimeVal) -> RuntimeVal {
        for scope in self.scopes.iter_mut().rev() {
            if scope.0.contains_key(&varname) {
                if scope.1.contains(&varname) {
                    println!("Cannot assign const {} : Cannot edit const.", varname);
                    exit(1);
                }
                scope.0.insert(varname.clone(), value.clone());
                return value;
            }
        }

        // Declare var if not assignable (= not declared)
        self.declare_var(varname, value, false)
    }
}
