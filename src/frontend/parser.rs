use crate::frontend::ast::{Expr, ExprNode, ObjectProperty, Stmt, StmtNode};
use crate::frontend::errors::{
    ParserError::{self, *},
    ParserErrorWithSpan,
};
use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenType::{self, *};
use crate::frontend::span::Span;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // Utils for parsing

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
        self.at().token_type != EOF
    }

    fn expect(
        &mut self,
        expected_type: TokenType,
        err_msg: &str,
    ) -> Result<Token, ParserErrorWithSpan> {
        let prev = self.eat();
        if prev.token_type != expected_type {
            return Err(ParserError::with_span(
                TokenExpected {
                    expected: expected_type,
                    found: prev.clone(),
                    hint: err_msg.to_string(),
                },
                prev.span,
            ));
        }
        Ok(prev)
    }

    // Parsing entrypoint

    pub fn produce_ast(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let mut program_body = Vec::new();

        while self.not_eof() {
            program_body.push(self.parse_statement()?);
        }

        Ok(StmtNode::new(Stmt::Program(program_body), Span::null()))
    }

    // Statements path

    fn parse_statement(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        match self.at().token_type {
            Let | Const => self.parse_var_declaration(),
            FnStart => self.parse_fn_declaration(),
            Return => self.parse_return(),
            If => self.parse_condition(),
            While => self.parse_while_loop(),
            For => self.parse_for_loop(),
            _ => {
                let e = self.parse_expression()?;
                Ok(StmtNode::new(Stmt::ExprStmt(e.clone()), e.span))
            }
        }
    }

    fn parse_return(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;
        let right = self.parse_expression()?;
        Ok(StmtNode::new(
            Stmt::Return(right.clone()),
            span_start.merge(&right.span),
        ))
    }

    fn parse_fn_declaration(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;

        let name = self.expect(Identifier, "Expect functionn name")?.value;

        let args = self.parse_args()?;
        let mut params: Vec<String> = Vec::new();
        for arg in args {
            match arg.node {
                Expr::Identifier(s) => {
                    params.push(s);
                }
                _ => {
                    return Err(ParserError::with_span(
                        Unexpected {
                            expected: String::from("identifier"),
                            hint: String::from(
                                "Expecting identifiers as arguments to the function",
                            ),
                        },
                        arg.span,
                    ));
                }
            }
        }

        let mut body: Vec<StmtNode> = Vec::new();

        while self.at().token_type != EOF && self.at().token_type != FnEnd {
            body.push(self.parse_statement()?);
        }

        let span_end = self
            .expect(
                FnEnd,
                "End function definition keyword expected at the end of the defined function.",
            )?
            .span;

        Ok(StmtNode::new(
            Stmt::FnDeclaration {
                name,
                parameters: params,
                body: body,
            },
            span_start.merge(&span_end),
        ))
    }

    fn parse_var_declaration(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let eat = self.eat();
        let is_constant = eat.token_type == Const;
        let identifier = self
            .expect(
                Identifier,
                "Expecting name (identifier) after variable declaration keyword.",
            )?
            .value;

        self.expect(
            Equals,
            "Expecting '=' after identifier for a variable declaration.",
        )?;

        let value = self.parse_expression()?;

        Ok(StmtNode::new(
            Stmt::VarDeclaration {
                constant: is_constant,
                identifier,
                value: Some(value.clone()),
            },
            eat.span.merge(&value.span),
        ))
    }

    fn parse_condition(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;

        let condition = self.parse_expression()?;
        self.expect(Then, "Missing 'then' keyword after condition expression")?;
        let mut body: Vec<StmtNode> = Vec::new();
        while self.not_eof() && self.at().token_type != IfEnd && self.at().token_type != Else {
            body.push(self.parse_statement()?);
        }

        let mut alternate: Option<Vec<StmtNode>> = None;
        if self.at().token_type == Else {
            self.eat();
            if self.at().token_type == If {
                let else_if_stmt = self.parse_condition()?;
                alternate = Some(vec![else_if_stmt]);
            } else {
                let mut alt_body = Vec::new();
                while self.not_eof() && self.at().token_type != IfEnd {
                    alt_body.push(self.parse_statement()?);
                }
                alternate = Some(alt_body);
            }
        }

        let span_end = self
            .expect(IfEnd, "Except 'endIf' after the condition body")?
            .span;

        Ok(StmtNode::new(
            Stmt::Condition {
                test: condition,
                body,
                alternate,
            },
            span_start.merge(&span_end),
        ))
    }

    fn parse_while_loop(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;
        let condition = self.parse_expression()?;
        self.expect(Then, "Missing 'then' keyword after condition expression")?;
        let mut body: Vec<StmtNode> = Vec::new();
        while self.not_eof() && self.at().token_type != WhileEnd {
            body.push(self.parse_statement()?);
        }
        let span_end = self
            .expect(WhileEnd, "Except 'endWhile' after the while loop body")?
            .span;
        Ok(StmtNode::new(
            Stmt::WhileLoop {
                test: condition,
                body,
            },
            span_start.merge(&span_end),
        ))
    }

    fn parse_for_loop(&mut self) -> Result<StmtNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;
        let identifier = self
            .expect(Identifier, "Identifier required after 'for' keyword")?
            .value;
        self.expect(In, "'in' keyword required after 'for' and identifier")?;
        let iterable = self.parse_expression()?;
        let mut body: Vec<StmtNode> = Vec::new();
        while self.not_eof() && self.at().token_type != ForEnd {
            body.push(self.parse_statement()?);
        }
        let span_end = self
            .expect(ForEnd, "Except 'endFor' after the for loop body")?
            .span;
        Ok(StmtNode::new(
            Stmt::ForLoop {
                iterable,
                identifier,
                body,
            },
            span_start.merge(&span_end),
        ))
    }

    fn parse_expression(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let left = self.parse_logical_expr()?;

        if self.at().token_type == Equals {
            self.eat();
            let value = self.parse_assignment_expr()?;
            return Ok(ExprNode::new(
                Expr::AssignmentExpr {
                    assigne: Box::new(left.clone()),
                    value: Box::new(value.clone()),
                },
                left.span.merge(&value.span),
            ));
        }

        Ok(left)
    }

    fn parse_logical_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let mut left = self.parse_additive_expr()?;

        while self.at().token_type == LogicalOperator {
            let operator = self.eat().value.to_string();
            let right = self.parse_additive_expr()?;
            left = ExprNode::new(
                Expr::LogicalExpr {
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                    operator,
                },
                left.span.merge(&right.span),
            )
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let mut left = self.parse_multiplicative_expr()?;

        while self.at().value == "+" || self.at().value == "-" {
            let operator = self.eat().value.to_string();
            let right = self.parse_multiplicative_expr()?;
            left = ExprNode::new(
                Expr::BinaryExpr {
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                    operator,
                },
                left.span.merge(&right.span),
            )
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let mut left = self.parse_unary_expr()?;

        while self.at().value == "*" || self.at().value == "/" || self.at().value == "%" {
            let operator = self.eat().value.to_string();
            let right = self.parse_unary_expr()?;
            left = ExprNode::new(
                Expr::BinaryExpr {
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                    operator,
                },
                left.span.merge(&right.span),
            )
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let span_start = self.at().span;
        if self.at().token_type == UnaryOperator {
            let operator = self.eat().value;
            let right = self.parse_unary_expr()?;
            return Ok(ExprNode::new(
                Expr::UnaryExpr {
                    right: Box::new(right.clone()),
                    operator,
                },
                span_start.merge(&right.span),
            ));
        }

        self.parse_call_member_expr()
    }

    fn parse_call_member_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let member = self.parse_member_expr()?;
        if self.at().token_type == OpenParen {
            return self.parse_call_expr(member);
        }
        Ok(member)
    }

    fn parse_member_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let mut object = self.parse_primary_expr()?;
        while self.at().token_type == Dot || self.at().token_type == OpenBracket {
            let operator = self.eat();
            let property: ExprNode;
            let computed: bool;

            let span_end = if operator.token_type == Dot {
                computed = false;
                property = self.parse_primary_expr()?;
                match property.node {
                    Expr::Identifier(_) => property.span,
                    _ => {
                        return Err(ParserError::with_span(
                            Unexpected {
                                expected: String::from("identifier"),
                                hint: String::from(
                                    "Cannot use '.' without an identifier on the right",
                                ),
                            },
                            property.span,
                        ));
                    }
                }
            } else {
                computed = true;
                property = self.parse_expression()?;
                self.expect(CloseBracket, "Missing ']'")?.span
            };

            object = ExprNode::new(
                Expr::MemberExpr {
                    object: Box::new(object.clone()),
                    property: Box::new(property),
                    computed,
                },
                object.span.merge(&span_end),
            );
        }

        Ok(object)
    }

    fn parse_call_expr(&mut self, caller: ExprNode) -> Result<ExprNode, ParserErrorWithSpan> {
        let args = self.parse_args()?;
        let mut call_expr = ExprNode::new(
            Expr::CallExpr {
                args,
                caller: Box::new(caller.clone()),
            },
            caller.span,
        );
        if self.at().token_type == OpenParen {
            call_expr = self.parse_call_expr(call_expr)?;
        }
        Ok(call_expr)
    }

    fn parse_args(&mut self) -> Result<Vec<ExprNode>, ParserErrorWithSpan> {
        self.expect(OpenParen, "Expect '(' before arguments")?;
        let args = match self.at().token_type {
            CloseParen => Vec::new(),
            _ => self.parse_arguments_list()?,
        };
        self.expect(CloseParen, "Expect ')' after arguments")?;
        Ok(args)
    }

    fn parse_arguments_list(&mut self) -> Result<Vec<ExprNode>, ParserErrorWithSpan> {
        let mut args = Vec::new();
        args.push(self.parse_assignment_expr()?);
        while self.not_eof() && self.at().token_type == Comma && self.eatable() {
            args.push(self.parse_assignment_expr()?);
        }
        Ok(args)
    }

    // Primary expressions

    fn parse_primary_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let first_token = self.at().clone();
        match first_token.token_type {
            Identifier => Ok(ExprNode::new(
                Expr::Identifier(self.eat().value.clone()),
                first_token.span,
            )),
            Number => Ok(ExprNode::new(
                Expr::NumericLiteral(match self.eat().value.clone().parse() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(ParserError::with_span(ParsingNumber(e), first_token.span));
                    }
                }),
                first_token.span,
            )),
            StringToken => Ok(ExprNode::new(
                Expr::StringLiteral(self.eat().value.clone()),
                first_token.span,
            )),
            OpenBracket => self.parse_list_expr(),
            OpenBrace => self.parse_object_expr(),
            OpenParen => {
                self.eat();
                let value = self.parse_expression();
                self.expect(CloseParen, "Expecting closing parenthesis")?;
                value
            }
            BinaryOperator => {
                let operator = self.eat().value;
                let v = self.parse_expression()?;
                match v.node {
                    Expr::NumericLiteral(_) | Expr::Identifier(_) | Expr::BinaryExpr { .. } => {
                        Ok(ExprNode::new(
                            Expr::BinaryExpr {
                                left: Box::new(ExprNode::new(
                                    Expr::NumericLiteral(0.0),
                                    first_token.span,
                                )),
                                right: Box::new(v.clone()),
                                operator,
                            },
                            first_token.span.merge(&v.span),
                        ))
                    }
                    _ => Err(ParserError::with_span(
                        UnexpectedToken {
                            found: self.at().clone(),
                            hint: String::from("Unexpected token after a binary operator"),
                        },
                        self.at().span,
                    )),
                }
            }
            _ => Err(ParserError::with_span(
                UnexpectedToken {
                    found: self.at().clone(),
                    hint: String::from("Unexpected token during parsing"),
                },
                self.at().span,
            )),
        }
    }

    fn parse_list_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;
        let mut list: Vec<ExprNode> = Vec::new();

        while self.not_eof() && self.at().token_type != CloseBracket {
            let value = self.parse_expression()?;

            list.push(value);

            if self.at().token_type != CloseBracket {
                self.expect(Comma, "Expecting ',' or ']' following list value")?;
            }
        }

        let span_end = self.expect(CloseBracket, "List litteral missing ']'")?.span;

        Ok(ExprNode::new(
            Expr::ListLiteral(list),
            span_start.merge(&span_end),
        ))
    }

    fn parse_object_expr(&mut self) -> Result<ExprNode, ParserErrorWithSpan> {
        let span_start = self.eat().span;
        let mut properties: Vec<ObjectProperty> = Vec::new();

        while self.not_eof() && self.at().token_type != CloseBrace {
            let key = self
                .expect(Identifier, "Object litteral key expected")?
                .value;

            if self.at().token_type == Comma {
                self.eat();
                properties.push(ObjectProperty {
                    key: key,
                    value: None,
                });
                continue;
            } else if self.at().token_type == CloseBrace {
                self.eat();
                properties.push(ObjectProperty {
                    key: key,
                    value: None,
                });
                continue;
            }

            self.expect(
                Colon,
                "Missing ':' following identifier in object expression",
            )?;
            let value = self.parse_expression()?;

            properties.push(ObjectProperty {
                key: key,
                value: Some(value),
            });

            if self.at().token_type != CloseBrace {
                self.expect(Comma, "Expecting ',' or '}' following property")?;
            }
        }

        let span_end = self.expect(CloseBrace, "Object litteral missing '}'")?.span;

        Ok(ExprNode::new(
            Expr::ObjectLiteral(properties),
            span_start.merge(&span_end),
        ))
    }
}
