use crate::frontend::ast::{Expr, Stmt};
use crate::runtime::environment::Environment;
use crate::runtime::values::RuntimeVal;

use crate::runtime::eval;

pub fn evaluate(statement: Stmt, env: &mut Environment) -> RuntimeVal {
    match statement {
        // Expr
        Stmt::ExprStmt(Expr::NumericLiteral(n)) => RuntimeVal::Number(n),
        Stmt::ExprStmt(Expr::Identifier(i)) => eval::expressions::eval_identifier(i, env),
        Stmt::ExprStmt(Expr::BinaryExpr {
            left,
            right,
            operator,
        }) => eval::expressions::eval_binary_expr(left, right, operator, env),
        Stmt::ExprStmt(Expr::ObjectLiteral(properties)) => {
            eval::expressions::eval_object_expr(properties, env)
        }
        Stmt::ExprStmt(Expr::AssignmentExpr { assigne, value }) => {
            eval::expressions::eval_assignment(assigne, value, env)
        }
        Stmt::ExprStmt(Expr::MemberExpr {
            object,
            property,
            computed,
        }) => eval::expressions::eval_member_expr(object, property, computed, env),
        Stmt::ExprStmt(Expr::CallExpr { args, caller }) => {
            eval::expressions::eval_call_expr(args, caller, env)
        }
        // Stmt
        Stmt::Program(body) => eval::statements::eval_program(body, env),
        Stmt::VarDeclaration {
            constant,
            identifier,
            value,
        } => eval::statements::eval_var_declaration(constant, identifier, value, env),
    }
}
