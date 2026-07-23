use std::env;
use std::fs;
use std::io::{Write, stdin, stdout};
use std::process::exit;

mod frontend;
mod runtime;

use frontend::lexer::tokenize;
use frontend::parser::Parser;
use runtime::environment::Environment;
use runtime::interpreter;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut env = Environment::new();

    if args.len() >= 2 {
        let filename = &args[1];
        let code = fs::read_to_string(filename).expect(&format!("Error reading file {}", filename));
        let tokens = match tokenize(&code) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer Error : {}", e);
                exit(1);
            }
        };
        let mut parser = Parser::new(tokens);
        let program = parser.produce_ast();
        let result = interpreter::evaluate(program, &mut env);
        match result {
            RuntimeVal::Null => {}
            _ => println!("-> {}", result),
        }
        return;
    }

    let mut u_input = get_user_input();
    while u_input != "exit" {
        let tokens = match tokenize(&u_input) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer Error : {}", e);
                exit(1);
            }
        };
        let mut parser = Parser::new(tokens);
        let program = parser.produce_ast();
        let result = interpreter::evaluate(program, &mut env);
        match result {
            RuntimeVal::Null => {}
            _ => println!("-> {}", result),
        }

        u_input = get_user_input();
    }
}
