use crate::expr::Expr;
use crate::scanner::Token;


#[derive(Debug)]
pub enum Stmt<'a> {
    Expression { expression: Expr<'a> },
    Print { expression: Expr<'a> },
    Var { name: Token<'a>, initializer: Expr<'a> },
}

impl<'a> Stmt<'a> {
    pub fn tostring(&self) -> String {
        use Stmt::*;
        match self {
            Expression { expression } => expression.to_string(),
            Print { expression } => format!("(print {})", expression.to_string()),
            Var { name, initializer } => format!("(var {})", name.lexeme),
        }
    }
}
