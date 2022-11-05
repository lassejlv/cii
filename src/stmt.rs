use crate::expr::Expr;
use crate::scanner::Token;


#[derive(Debug)]
pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr},
    Var { name: Token, initializer: Expr },
}
