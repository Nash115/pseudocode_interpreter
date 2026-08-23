use crate::frontend::ast::{Expr, ObjectProperty, Stmt};
use crate::frontend::errors::ParserError;
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

    fn expect(&mut self, expected_type: TokenType, err_msg: &str) -> Result<Token, ParserError> {
        let prev = self.eat();
        if prev.token_type != expected_type {
            return Err(ParserError::TokenExpected {
                expected: expected_type,
                found: prev,
                hint: err_msg.to_string(),
            });
        }
        Ok(prev)
    }

    pub fn produce_ast(&mut self) -> Result<Stmt, ParserError> {
        let mut program_body = Vec::new();

        while self.not_eof() {
            program_body.push(self.parse_statement()?);
        }

        Ok(Stmt::Program(program_body))
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        match self.at().token_type {
            TokenType::Let | TokenType::Const => self.parse_var_declaration(),
            TokenType::FnStart => self.parse_fn_declaration(),
            TokenType::Return => self.parse_return(),
            TokenType::If => self.parse_condition(),
            TokenType::While => self.parse_while_loop(),
            _ => Ok(Stmt::ExprStmt(self.parse_expression()?)),
        }
    }

    fn parse_return(&mut self) -> Result<Stmt, ParserError> {
        self.eat();
        let right = self.parse_expression()?;
        Ok(Stmt::Return(right))
    }

    fn parse_fn_declaration(&mut self) -> Result<Stmt, ParserError> {
        self.eat();

        let name = self
            .expect(TokenType::Identifier, "Expect functionn name")?
            .value;

        let args = self.parse_args()?;
        let mut params: Vec<String> = Vec::new();
        for arg in args {
            match arg {
                Expr::Identifier(s) => {
                    params.push(s);
                }
                _ => {
                    return Err(ParserError::Unexpected {
                        expected: String::from("identifier"),
                        hint: String::from("Expecting identifiers as arguments to the function"),
                    });
                }
            }
        }

        let mut body: Vec<Stmt> = Vec::new();

        while self.at().token_type != TokenType::EOF && self.at().token_type != TokenType::FnEnd {
            body.push(self.parse_statement()?);
        }

        self.expect(
            TokenType::FnEnd,
            "End function definition keyword expected at the end of the defined function.",
        )?;

        Ok(Stmt::FnDeclaration {
            name,
            parameters: params,
            body: body,
        })
    }

    fn parse_var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let is_constant = self.eat().token_type == TokenType::Const;
        let identifier = self
            .expect(
                TokenType::Identifier,
                "Expecting name (identifier) after variable declaration keyword.",
            )?
            .value;

        self.expect(
            TokenType::Equals,
            "Expecting '=' after identifier for a variable declaration.",
        )?;

        let declaration = Stmt::VarDeclaration {
            constant: is_constant,
            identifier,
            value: Some(self.parse_expression()?),
        };

        Ok(declaration)
    }

    fn parse_condition(&mut self) -> Result<Stmt, ParserError> {
        self.eat();
        let stmt = self.parse_condition_internal()?;
        self.expect(TokenType::IfEnd, "Except 'endIf' after the condition body")?;
        Ok(stmt)
    }

    fn parse_condition_internal(&mut self) -> Result<Stmt, ParserError> {
        let condition = self.parse_expression()?;
        self.expect(
            TokenType::Then,
            "Missing 'then' keyword after condition expression",
        )?;
        let mut body: Vec<Stmt> = Vec::new();
        while self.not_eof()
            && self.at().token_type != TokenType::IfEnd
            && self.at().token_type != TokenType::Else
        {
            body.push(self.parse_statement()?);
        }

        let mut alternate: Option<Vec<Stmt>> = None;
        if self.at().token_type == TokenType::Else {
            self.eat();
            if self.at().token_type == TokenType::If {
                self.eat();
                let else_if_stmt = self.parse_condition_internal()?;
                alternate = Some(vec![else_if_stmt]);
            } else {
                let mut alt_body = Vec::new();
                while self.not_eof() && self.at().token_type != TokenType::IfEnd {
                    alt_body.push(self.parse_statement()?);
                }
                alternate = Some(alt_body);
            }
        }

        Ok(Stmt::Condition {
            test: condition,
            body,
            alternate,
        })
    }

    fn parse_while_loop(&mut self) -> Result<Stmt, ParserError> {
        self.eat();
        let condition = self.parse_expression()?;
        self.expect(
            TokenType::Then,
            "Missing 'then' keyword after condition expression",
        )?;
        let mut body: Vec<Stmt> = Vec::new();
        while self.not_eof() && self.at().token_type != TokenType::WhileEnd {
            body.push(self.parse_statement()?);
        }
        self.expect(
            TokenType::WhileEnd,
            "Except 'endWhile' after the while loop body",
        )?;
        Ok(Stmt::WhileLoop {
            test: condition,
            body,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParserError> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_logical_expr()?;

        if self.at().token_type == TokenType::Equals {
            self.eat();
            let value = self.parse_assignment_expr()?;
            return Ok(Expr::AssignmentExpr {
                assigne: Box::new(left),
                value: Box::new(value),
            });
        }

        Ok(left)
    }

    fn parse_logical_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_unary_expr()?;

        while self.at().token_type == TokenType::LogicalOperator {
            let operator = self.eat().value.to_string();
            let right = self.parse_unary_expr()?;
            left = Expr::LogicalExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParserError> {
        if self.at().token_type == TokenType::UnaryOperator {
            let operator = self.eat().value;
            let right = self.parse_unary_expr()?;
            return Ok(Expr::UnaryExpr {
                right: Box::new(right),
                operator,
            });
        }

        self.parse_object_expr()
    }

    fn parse_object_expr(&mut self) -> Result<Expr, ParserError> {
        if self.at().token_type != TokenType::OpenBrace {
            return self.parse_additive_expr();
        }

        self.eat();
        let mut properties: Vec<ObjectProperty> = Vec::new();

        while self.not_eof() && self.at().token_type != TokenType::CloseBrace {
            let key = self
                .expect(TokenType::Identifier, "Object litteral key expected")?
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
            )?;
            let value = self.parse_expression()?;

            properties.push(ObjectProperty {
                key: key,
                value: Some(value),
            });

            if self.at().token_type != TokenType::CloseBrace {
                self.expect(TokenType::Comma, "Expecting ',' or '}' following property")?;
            }
        }

        self.expect(TokenType::CloseBrace, "Object litteral missing '}'")?;

        Ok(Expr::ObjectLiteral(properties))
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_multiplicative_expr()?;

        while self.at().value == "+" || self.at().value == "-" {
            let operator = self.eat().value.to_string();
            let right = self.parse_multiplicative_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_call_member_expr()?;

        while self.at().value == "*" || self.at().value == "/" || self.at().value == "%" {
            let operator = self.eat().value.to_string();
            let right = self.parse_call_member_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            }
        }

        Ok(left)
    }

    fn parse_call_member_expr(&mut self) -> Result<Expr, ParserError> {
        let member = self.parse_member_expr()?;
        if self.at().token_type == TokenType::OpenParen {
            return self.parse_call_expr(member);
        }
        Ok(member)
    }

    fn parse_call_expr(&mut self, caller: Expr) -> Result<Expr, ParserError> {
        let mut call_expr = Expr::CallExpr {
            args: self.parse_args()?,
            caller: Box::new(caller),
        };
        if self.at().token_type == TokenType::OpenParen {
            call_expr = self.parse_call_expr(call_expr)?;
        }
        Ok(call_expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        self.expect(TokenType::OpenParen, "Expect '(' before arguments")?;
        let args = match self.at().token_type {
            TokenType::CloseParen => Vec::new(),
            _ => self.parse_arguments_list()?,
        };
        self.expect(TokenType::CloseParen, "Expect ')' after arguments")?;
        Ok(args)
    }

    fn parse_arguments_list(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();
        args.push(self.parse_assignment_expr()?);
        while self.not_eof() && self.at().token_type == TokenType::Comma && self.eatable() {
            args.push(self.parse_assignment_expr()?);
        }
        Ok(args)
    }

    fn parse_member_expr(&mut self) -> Result<Expr, ParserError> {
        let mut object = self.parse_primary_expr()?;
        while self.at().token_type == TokenType::Dot
            || self.at().token_type == TokenType::OpenBracket
        {
            let operator = self.eat();
            let property: Expr;
            let computed: bool;

            if operator.token_type == TokenType::Dot {
                computed = false;
                property = self.parse_primary_expr()?;
                match property {
                    Expr::Identifier(_) => {}
                    _ => {
                        return Err(ParserError::Unexpected {
                            expected: String::from("identifier"),
                            hint: String::from("Cannot use '.' without an identifier on the right"),
                        });
                    }
                }
            } else {
                computed = true;
                property = self.parse_expression()?;
                self.expect(TokenType::CloseBracket, "Missing ']'")?;
            }

            object = Expr::MemberExpr {
                object: Box::new(object),
                property: Box::new(property),
                computed,
            };
        }

        Ok(object)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParserError> {
        match self.at().token_type {
            TokenType::Identifier => Ok(Expr::Identifier(self.eat().value.clone())),
            TokenType::Number => Ok(Expr::NumericLiteral(
                match self.eat().value.clone().parse() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(ParserError::ParsingNumber(e));
                    }
                },
            )),
            TokenType::String => Ok(Expr::StringLiteral(self.eat().value.clone())),
            TokenType::OpenParen => {
                self.eat();
                let value = self.parse_expression();
                self.expect(TokenType::CloseParen, "Expecting closing parenthesis")?;
                value
            }
            TokenType::BinaryOperator => {
                let operator = self.eat().value;
                let v = self.parse_expression()?;
                match v {
                    Expr::NumericLiteral(_) | Expr::Identifier(_) | Expr::BinaryExpr { .. } => {
                        Ok(Expr::BinaryExpr {
                            left: Box::new(Expr::NumericLiteral(0.0)),
                            right: Box::new(v),
                            operator,
                        })
                    }
                    _ => Err(ParserError::UnexpectedToken {
                        found: self.at().clone(),
                        hint: String::from("Unexpected token after a binary operator"),
                    }),
                }
            }
            _ => Err(ParserError::UnexpectedToken {
                found: self.at().clone(),
                hint: String::from("Unexpected token during parsing"),
            }),
        }
    }
}
