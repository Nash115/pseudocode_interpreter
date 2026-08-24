use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::frontend::errors::InterpreterError;
use crate::runtime::default_env;
use crate::runtime::values::RuntimeVal;

#[derive(PartialEq)]
pub struct Environment {
    pub variables: HashMap<String, RuntimeVal>,
    pub constants: HashSet<String>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}
impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            variables: HashMap::new(),
            constants: HashSet::new(),
            parent,
        }
    }

    pub fn create_global() -> Result<Rc<RefCell<Self>>, InterpreterError> {
        let env = Rc::new(RefCell::new(Environment::new(None)));

        default_env::load_default_variables(&mut env.borrow_mut())?;
        default_env::load_default_functions(&mut env.borrow_mut())?;
        default_env::load_default_aliases(&mut env.borrow_mut())?;

        Ok(env)
    }

    pub fn declare_var(
        &mut self,
        varname: String,
        value: RuntimeVal,
        constant: bool,
    ) -> Result<RuntimeVal, InterpreterError> {
        if self.variables.contains_key(&varname) {
            return Err(InterpreterError::VarAlreadyDeclared(varname));
        }

        self.variables.insert(varname.clone(), value.clone());
        if constant {
            self.constants.insert(varname);
        }
        Ok(value)
    }

    pub fn lookup_var(&self, varname: &str) -> Result<RuntimeVal, InterpreterError> {
        if let Some(val) = self.variables.get(varname) {
            return Ok(val.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().lookup_var(varname);
        }

        Err(InterpreterError::VarUnresolvable(varname.to_string()))
    }

    pub fn assign_var(
        &mut self,
        varname: String,
        value: RuntimeVal,
    ) -> Result<RuntimeVal, InterpreterError> {
        if self.variables.contains_key(&varname) {
            if self.constants.contains(&varname) {
                return Err(InterpreterError::EditConst(varname));
            }
            self.variables.insert(varname.clone(), value.clone());
            return Ok(value);
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign_var(varname, value);
        }

        // Declare var if not assignable (= not declared)
        self.declare_var(varname, value, false)
    }

    pub fn alias(&mut self, varname: &str, alias: &str) -> Result<RuntimeVal, InterpreterError> {
        let source_value = self.lookup_var(varname)?;
        Ok(self.assign_var(String::from(alias), source_value)?)
    }
}
