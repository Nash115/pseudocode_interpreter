use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::ast::StmtNode;
use crate::frontend::ast::{Expr::*, Stmt::*};
use crate::frontend::errors::RuntimeError;
use crate::runtime::environment::Environment;
use crate::runtime::eval;
use crate::runtime::values::RuntimeVal;

pub fn evaluate(
    statement: StmtNode,
    env: &Rc<RefCell<Environment>>,
) -> Result<RuntimeVal, RuntimeError> {
    match statement.node {
        // Expr
        ExprStmt(s) => match s.node {
            NumericLiteral(n) => Ok(RuntimeVal::Number(n)),
            StringLiteral(s) => Ok(RuntimeVal::String(s)),
            Identifier(i) => Ok(eval::expressions::eval_identifier(i, statement.span, env)?),
            BinaryExpr {
                left,
                right,
                operator,
            } => Ok(eval::expressions::eval_binary_expr(
                left, right, operator, env,
            )?),
            UnaryExpr { right, operator } => {
                Ok(eval::expressions::eval_unary_expr(right, operator, env)?)
            }
            LogicalExpr {
                left,
                right,
                operator,
            } => Ok(eval::expressions::eval_logical_expr(
                left, right, operator, env,
            )?),
            ObjectLiteral(properties) => Ok(eval::expressions::eval_object_expr(
                properties,
                statement.span,
                env,
            )?),
            ListLiteral(values) => Ok(eval::expressions::eval_list_expr(values, env)?),
            AssignmentExpr { assigne, value } => {
                Ok(eval::expressions::eval_assignment(assigne, value, env)?)
            }
            MemberExpr {
                object,
                property,
                computed,
            } => Ok(eval::expressions::eval_member_expr(
                object, property, computed, env,
            )?),
            CallExpr { args, caller } => Ok(eval::expressions::eval_call_expr(args, caller, env)?),
        },
        // Stmt
        Program(body) => Ok(eval::statements::eval_program(body, env)?),
        VarDeclaration {
            constant,
            identifier,
            value,
        } => Ok(eval::statements::eval_var_declaration(
            constant,
            identifier,
            value,
            statement.span,
            env,
        )?),
        FnDeclaration {
            name,
            parameters,
            body,
        } => Ok(eval::statements::eval_fn_declaration(
            name,
            parameters,
            body,
            statement.span,
            env,
        )?),
        Return(e) => {
            let val = evaluate(StmtNode::new(ExprStmt(e.clone()), e.span), env)?;
            Ok(RuntimeVal::ReturnValue(Box::new(val)))
        }
        Condition {
            test,
            body,
            alternate,
        } => Ok(eval::statements::eval_condition(
            test, body, alternate, env,
        )?),
        WhileLoop { test, body } => Ok(eval::statements::eval_while_loop(test, body, env)?),
        ForLoop {
            iterable,
            identifier,
            body,
        } => Ok(eval::statements::eval_for_loop(
            iterable, identifier, body, env,
        )?),
    }
}
