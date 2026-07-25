use crate::frontend::ast::{Expr, Stmt};
use crate::frontend::errors::InterpreterError;
use crate::runtime::environment::Environment;
use crate::runtime::eval;
use crate::runtime::values::RuntimeVal;

pub fn evaluate(statement: Stmt, env: &mut Environment) -> Result<RuntimeVal, InterpreterError> {
    match statement {
        // Expr
        Stmt::ExprStmt(Expr::NumericLiteral(n)) => Ok(RuntimeVal::Number(n)),
        Stmt::ExprStmt(Expr::Identifier(i)) => Ok(eval::expressions::eval_identifier(i, env)?),
        Stmt::ExprStmt(Expr::BinaryExpr {
            left,
            right,
            operator,
        }) => Ok(eval::expressions::eval_binary_expr(
            left, right, operator, env,
        )?),
        Stmt::ExprStmt(Expr::UnaryExpr { right, operator }) => {
            Ok(eval::expressions::eval_unary_expr(right, operator, env)?)
        }
        Stmt::ExprStmt(Expr::LogicalExpr {
            left,
            right,
            operator,
        }) => Ok(eval::expressions::eval_logical_expr(
            left, right, operator, env,
        )?),
        Stmt::ExprStmt(Expr::ObjectLiteral(properties)) => {
            Ok(eval::expressions::eval_object_expr(properties, env)?)
        }
        Stmt::ExprStmt(Expr::AssignmentExpr { assigne, value }) => {
            Ok(eval::expressions::eval_assignment(assigne, value, env)?)
        }
        Stmt::ExprStmt(Expr::MemberExpr {
            object,
            property,
            computed,
        }) => Ok(eval::expressions::eval_member_expr(
            object, property, computed, env,
        )?),
        Stmt::ExprStmt(Expr::CallExpr { args, caller }) => {
            Ok(eval::expressions::eval_call_expr(args, caller, env)?)
        }
        // Stmt
        Stmt::Program(body) => Ok(eval::statements::eval_program(body, env)?),
        Stmt::VarDeclaration {
            constant,
            identifier,
            value,
        } => Ok(eval::statements::eval_var_declaration(
            constant, identifier, value, env,
        )?),
        Stmt::FnDeclaration {
            name,
            parameters,
            body,
        } => Ok(eval::statements::eval_fn_declaration(
            name, parameters, body, env,
        )?),
        Stmt::Return(_) => Err(InterpreterError::UnexpectedReturn),
    }
}
