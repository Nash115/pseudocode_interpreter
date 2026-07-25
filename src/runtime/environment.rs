use std::collections::{HashMap, HashSet};

use crate::frontend::errors::InterpreterError;
use crate::runtime::values::RuntimeVal;

pub struct Scope {
    pub variables: HashMap<String, RuntimeVal>,
    pub constants: HashSet<String>,
    pub parent_idx: Option<usize>,
}

pub struct Environment {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
}
impl Environment {
    pub fn new() -> Result<Self, InterpreterError> {
        let mut env = Environment {
            scopes: vec![Scope {
                variables: HashMap::new(),
                constants: HashSet::new(),
                parent_idx: None,
            }],
            current_scope: 0,
        };

        // Define natives variables
        env.declare_var(
            String::from("PI"),
            RuntimeVal::Number(3.14159265358979323846264338327950288),
            true,
        )?;
        env.declare_var(String::from("true"), RuntimeVal::Boolean(true), true)?;
        env.declare_var(String::from("false"), RuntimeVal::Boolean(false), true)?;
        env.declare_var(String::from("null"), RuntimeVal::Null, true)?;

        // Define natives functions
        env.declare_var(
            String::from("print"),
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
        )?;
        env.declare_var(
            String::from("time"),
            RuntimeVal::NativeFn(|_args, _env| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let start = SystemTime::now();
                let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
                RuntimeVal::Number(since_the_epoch.as_millis() as f64)
            }),
            true,
        )?;

        // Define aliases (natives variables and functions)
        env.alias("true", "vrai")?;
        env.alias("false", "faux")?;
        env.alias("null", "nul")?;
        env.alias("print", "affiche")?;
        env.alias("time", "temps")?;

        Ok(env)
    }

    pub fn push_scope(&mut self, parent_idx: Option<usize>) -> usize {
        let new_idx = self.scopes.len();
        self.scopes.push(Scope {
            variables: HashMap::new(),
            constants: HashSet::new(),
            parent_idx,
        });
        let previous = self.current_scope;
        self.current_scope = new_idx;
        previous
    }

    pub fn pop_scope(&mut self, previous_scope: usize) {
        self.current_scope = previous_scope;
    }

    pub fn this_scope(&mut self) -> usize {
        self.current_scope
    }

    pub fn declare_var(
        &mut self,
        varname: String,
        value: RuntimeVal,
        constant: bool,
    ) -> Result<RuntimeVal, InterpreterError> {
        let scope = &mut self.scopes[self.current_scope];

        if scope.variables.contains_key(&varname) {
            return Err(InterpreterError::VarAlreadyDeclared(varname));
        }

        scope.variables.insert(varname.clone(), value.clone());
        if constant {
            scope.constants.insert(varname);
        }
        Ok(value)
    }

    pub fn lookup_var(&self, varname: &str) -> Result<RuntimeVal, InterpreterError> {
        let mut current_idx = Some(self.current_scope);

        while let Some(idx) = current_idx {
            let scope = &self.scopes[idx];
            if let Some(val) = scope.variables.get(varname) {
                return Ok(val.clone());
            }
            current_idx = scope.parent_idx;
        }

        Err(InterpreterError::VarUnresolvable(varname.to_string()))
    }

    pub fn assign_var(
        &mut self,
        varname: String,
        value: RuntimeVal,
    ) -> Result<RuntimeVal, InterpreterError> {
        let mut current_idx = Some(self.current_scope);

        while let Some(idx) = current_idx {
            if self.scopes[idx].variables.contains_key(&varname) {
                if self.scopes[idx].constants.contains(&varname) {
                    return Err(InterpreterError::EditConst(varname));
                }
                self.scopes[idx]
                    .variables
                    .insert(varname.clone(), value.clone());
                return Ok(value);
            }
            current_idx = self.scopes[idx].parent_idx;
        }

        // Declare var if not assignable (= not declared)
        self.declare_var(varname, value, false)
    }

    fn alias(&mut self, varname: &str, alias: &str) -> Result<RuntimeVal, InterpreterError> {
        let source_value = self.lookup_var(varname)?;
        Ok(self.assign_var(String::from(alias), source_value)?)
    }
}
