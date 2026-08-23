use std::collections::{HashMap, HashSet};

use crate::frontend::errors::InterpreterError;
use crate::runtime::default_env;
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
        default_env::load_default_variables(&mut env)?;

        // Define natives functions
        default_env::load_default_functions(&mut env)?;

        // Define aliases (natives variables and functions)
        default_env::load_default_aliases(&mut env)?;

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

    pub fn alias(&mut self, varname: &str, alias: &str) -> Result<RuntimeVal, InterpreterError> {
        let source_value = self.lookup_var(varname)?;
        Ok(self.assign_var(String::from(alias), source_value)?)
    }
}
