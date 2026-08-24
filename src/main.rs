use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{Write, stdin, stdout};
use std::process::exit;
use std::rc::Rc;

mod frontend;
mod runtime;

use crate::frontend::lexer::tokenize;
use crate::frontend::parser::Parser;
use crate::runtime::environment::Environment;
use crate::runtime::interpreter;
use crate::runtime::values::RuntimeVal;

fn get_user_input() -> String {
    let mut user_input = String::new();
    print!("> ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut user_input)
        .expect("Did not enter a correct string");
    if let Some('\n') = user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r') = user_input.chars().next_back() {
        user_input.pop();
    }
    user_input
}

fn run_code(
    code: &str,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, Box<dyn std::error::Error>> {
    let tokens = tokenize(code)?;
    let program = Parser::new(tokens).produce_ast()?;
    let result = interpreter::evaluate(program, env)?;
    Ok(result)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut env = match Environment::create_global() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error creating Environment : {}", e);
            exit(1);
        }
    };

    if args.len() >= 2 {
        let filename = &args[1];
        let code = fs::read_to_string(filename).expect(&format!("Error reading file {}", filename));
        let result = match run_code(&code, &mut env) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };
        match result {
            RuntimeVal::Null => {}
            _ => println!("-> {}", result),
        }
        return;
    }

    let mut u_input = get_user_input();
    while u_input != "exit" {
        match run_code(&u_input, &mut env) {
            Ok(r) => match r {
                RuntimeVal::Null => {}
                _ => println!("-> {}", r),
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        };
        u_input = get_user_input();
    }
}
