use std::env;
use std::fs;
use std::io::{Write, stdin, stdout};
use std::process::exit;

mod frontend;
mod runtime;

use crate::frontend::errors::{InterpreterError, LexerError, ParserError};
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

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut env = match Environment::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error creating Environment : {}", e);
            exit(1);
        }
    };

    if args.len() >= 2 {
        let filename = &args[1];
        let code = fs::read_to_string(filename).expect(&format!("Error reading file {}", filename));
        let result = match tokenize(&code) {
            Ok(tokens) => match Parser::new(tokens).produce_ast() {
                Ok(program) => match interpreter::evaluate(program, &mut env) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            },
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
        let mut lexer_error: Option<LexerError> = None;
        let mut parser_error: Option<ParserError> = None;
        let mut interpreter_error: Option<InterpreterError> = None;
        let result = match tokenize(&u_input) {
            Ok(tokens) => match Parser::new(tokens).produce_ast() {
                Ok(program) => match interpreter::evaluate(program, &mut env) {
                    Ok(r) => Ok(r),
                    Err(e) => {
                        interpreter_error = Some(e);
                        Err(())
                    }
                },
                Err(e) => {
                    parser_error = Some(e);
                    Err(())
                }
            },
            Err(e) => {
                lexer_error = Some(e);
                Err(())
            }
        };
        match result {
            Ok(r) => match r {
                RuntimeVal::Null => {}
                _ => println!("-> {}", r),
            },
            Err(_) => {
                if let Some(e) = lexer_error {
                    eprintln!("{}", e);
                }
                if let Some(e) = parser_error {
                    eprintln!("{}", e);
                }
                if let Some(e) = interpreter_error {
                    eprintln!("{}", e);
                }
            }
        }

        u_input = get_user_input();
    }
}
