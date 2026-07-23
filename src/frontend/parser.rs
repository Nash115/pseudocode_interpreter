use std::process::exit;

use crate::frontend::ast::{Expr, ObjectProperty, Stmt};
use crate::frontend::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn at(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn eat(&mut self) -> Token {
        let prev = self.tokens[self.pos].clone();
        self.pos += 1;
        prev
    }

    fn eatable(&mut self) -> bool {
        self.eat();
        true
    }

    fn not_eof(&self) -> bool {
        self.at().token_type != TokenType::EOF
    }

    fn expect(&mut self, expected_type: TokenType, err_msg: &str) -> Token {
        let prev = self.eat();
        if prev.token_type != expected_type {
            println!(
                "Parser error:\n{} - Expecting: {:?}, found: {:?}",
                err_msg, expected_type, prev.token_type
            );
            exit(1);
        }
        prev
    }

    pub fn produce_ast(&mut self) -> Stmt {
        let mut program_body = Vec::new();

        while self.not_eof() {
            program_body.push(self.parse_statement());
        }

        Stmt::Program(program_body)
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.at().token_type {
            TokenType::Let | TokenType::Const => self.parse_var_declaration(),
            _ => Stmt::ExprStmt(self.parse_expression()),
        }
    }

    fn parse_var_declaration(&mut self) -> Stmt {
        let is_constant = self.eat().token_type == TokenType::Const;
        let identifier = self
            .expect(
                TokenType::Identifier,
                "Expecting identifier name after variable declaration keyword.",
            )
            .value;

        if self.at().token_type == TokenType::Semicolon {
            self.eat();
            if is_constant {
                println!("Unexpected semicolon. A constant must be declared with a value.");
                exit(1);
            }
            return Stmt::VarDeclaration {
                constant: false,
                identifier: identifier,
                value: None,
            };
        }

        self.expect(
            TokenType::Equals,
            "Expecting '=' after identifier for a variable declaration.",
        );

        let declaration = Stmt::VarDeclaration {
            constant: is_constant,
            identifier,
            value: Some(self.parse_expression()),
        };

        if self.at().token_type == TokenType::Semicolon {
            self.eat();
        }

        declaration
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Expr {
        let left = self.parse_object_expr();

        if self.at().token_type == TokenType::Equals {
            self.eat();
            let value = self.parse_assignment_expr();
            return Expr::AssignmentExpr {
                assigne: Box::new(left),
                value: Box::new(value),
            };
        }

        left
    }

    fn parse_object_expr(&mut self) -> Expr {
        if self.at().token_type != TokenType::OpenBrace {
            return self.parse_additive_expr();
        }

        self.eat();
        let mut properties: Vec<ObjectProperty> = Vec::new();

        while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
            let key = self
                .expect(TokenType::Identifier, "Object litteral key expected")
                .value;

            if self.at().token_type == TokenType::Comma {
                self.eat();
                properties.push(ObjectProperty {
                    key: key,
                    value: None,
                });
                continue;
            } else if self.at().token_type == TokenType::CloseBrace {
                self.eat();
                properties.push(ObjectProperty {
                    key: key,
                    value: None,
                });
                continue;
            }

            self.expect(
                TokenType::Colon,
                "Missing ':' following identifier in object expression",
            );
            let value = self.parse_expression();

            properties.push(ObjectProperty {
                key: key,
                value: Some(value),
            });

            if self.at().token_type != TokenType::CloseBrace {
                self.expect(TokenType::Comma, "Expecting ',' or '}' following property");
            }
        }

        self.expect(TokenType::CloseBrace, "Object litteral missing '}'");

        Expr::ObjectLiteral(properties)
    }

    fn parse_additive_expr(&mut self) -> Expr {
        let mut left = self.parse_multiplicative_expr();

        while self.at().value == "+" || self.at().value == "-" {
            let operator = self.eat().value.to_string();
            let right = self.parse_multiplicative_expr();
            left = Expr::BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> Expr {
        let mut left = self.parse_call_member_expr();

        while self.at().value == "*" || self.at().value == "/" || self.at().value == "%" {
            let operator = self.eat().value.to_string();
            let right = self.parse_call_member_expr();
            left = Expr::BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        left
    }

    fn parse_call_member_expr(&mut self) -> Expr {
        let member = self.parse_member_expr();
        if self.at().token_type == TokenType::OpenParen {
            return self.parse_call_expr(member);
        }
        member
    }

    fn parse_call_expr(&mut self, caller: Expr) -> Expr {
        let mut call_expr = Expr::CallExpr {
            args: self.parse_args(),
            caller: Box::new(caller),
        };
        if self.at().token_type == TokenType::OpenParen {
            call_expr = self.parse_call_expr(call_expr);
        }
        call_expr
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        self.expect(TokenType::OpenParen, "Expect '(' before arguments");
        let args = match self.at().token_type {
            TokenType::CloseParen => Vec::new(),
            _ => self.parse_arguments_list(),
        };
        self.expect(TokenType::CloseParen, "Expect ')' after arguments");
        args
    }

    fn parse_arguments_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        args.push(self.parse_assignment_expr());
        while self.not_eof() && self.at().token_type == TokenType::Comma && self.eatable() {
            args.push(self.parse_assignment_expr());
        }
        args
    }

    fn parse_member_expr(&mut self) -> Expr {
        let mut object = self.parse_primary_expr();
        while self.at().token_type == TokenType::Dot
            || self.at().token_type == TokenType::OpenBracket
        {
            let operator = self.eat();
            let property: Expr;
            let computed: bool;

            if operator.token_type == TokenType::Dot {
                computed = false;
                property = self.parse_primary_expr();
                match property {
                    Expr::Identifier(_) => {}
                    _ => {
                        println!("Cannot use '.' without an identifier on the right");
                        exit(1);
                    }
                }
            } else {
                computed = true;
                property = self.parse_expression();
                self.expect(TokenType::CloseBracket, "Missing ']'");
            }

            object = Expr::MemberExpr {
                object: Box::new(object),
                property: Box::new(property),
                computed,
            };
        }

        object
    }

    fn parse_primary_expr(&mut self) -> Expr {
        match self.at().token_type {
            TokenType::Identifier => Expr::Identifier(self.eat().value.clone()),
            TokenType::Number => Expr::NumericLiteral(self.eat().value.clone().parse().unwrap()),
            TokenType::OpenParen => {
                self.eat();
                let value = self.parse_expression();
                self.expect(TokenType::CloseParen, "Expecting closing parenthesis");
                value
            }
            _ => {
                println!("Unexpected token: {:?} during parsing", self.at());
                exit(1);
            }
        }
    }
}
