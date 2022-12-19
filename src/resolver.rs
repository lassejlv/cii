use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            scopes: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn resolve(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements: _ } => self.resolve_block(stmt)?,
            Stmt::Var {
                name: _,
                initializer: _,
            } => self.resolve_var(stmt)?,
            Stmt::Function { name: _, params: _, body: _ } => self.resolve_function(stmt)?,
            Stmt::Expression { expression } => self.resolve_expr(expression)?,
            Stmt::IfStmt {
                predicate: _,
                then: _,
                els: _,
            } => self.resolve_if_stmt(stmt)?,
            Stmt::Print { expression } => self.resolve_expr(expression)?,
            Stmt::ReturnStmt {
                keyword: _,
                value: None,
            } => (),
            Stmt::ReturnStmt {
                keyword: _,
                value: Some(value),
            } => self.resolve_expr(value)?,
            Stmt::WhileStmt { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve(body.as_ref())?;
            }
        }
        Ok(())
    }

    fn resolve_many(&mut self, stmts: &Vec<Box<Stmt>>) -> Result<(), String> {
        for stmt in stmts {
            self.resolve(stmt.as_ref())?;
        }

        Ok(())
    }

    fn resolve_block(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_many(statements)?;
                self.end_scope();
            }
            _ => panic!("Wrong type"),
        }

        Ok(())
    }

    fn resolve_var(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::Var { name, initializer } = stmt {
            self.declare(name);
            self.resolve_expr(initializer)?;
            self.define(name);
        } else {
            panic!("Wrong type in resolve var");
        }

        Ok(())
    }

    fn resolve_function(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::Function { name, params, body } = stmt {
            self.declare(name);
            self.define(name);

            self.resolve_function_helper(params, body)
        } else {
            panic!("Wrong type in resolve function");
        }
    }

    fn resolve_if_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::IfStmt {
            predicate,
            then,
            els,
        } = stmt
        {
            self.resolve_expr(predicate)?;
            self.resolve(then.as_ref())?;
            if let Some(els) = els {
                self.resolve(els.as_ref())?;
            }

            Ok(())
        } else {
            panic!("Wrong type in resolve if stmt");
        }
    }

    fn resolve_function_helper(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> Result<(), String> {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_many(body)?;
        self.end_scope();

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Stack underflow");
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let size = self.scopes.len();
        self.scopes[size - 1].insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let size = self.scopes.len();
        self.scopes[size - 1].insert(name.lexeme.clone(), true);
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Variable { name: _ } => self.resolve_expr_var(expr),
            Expr::Assign { name: _, value: _ } => self.resolve_expr_assign(expr),
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(callee.as_ref())?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }

                Ok(())
            }
            Expr::Grouping { expression } => self.resolve_expr(expression),
            Expr::Literal { value: _ } => Ok(()),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Unary { operator: _, right } => self.resolve_expr(right),
            Expr::AnonFunction {
                paren: _,
                arguments,
                body,
            } => self.resolve_function_helper(arguments, body),
        }
    }

    fn resolve_expr_var(&mut self, expr: &Expr) -> Result<(), String> {
        if let Expr::Variable { name } = expr {
            if !self.scopes.is_empty()
                && *self.scopes[self.scopes.len() - 1]
                    .get(&name.lexeme)
                    .unwrap()
                    == false
            {
                return Err("Can't read local variable in its own initializer".to_string());
            }

            self.resolve_local(expr, name)
        } else {
            panic!("Wrong type in resolve_expr_var");
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result<(), String> {
        let size = self.scopes.len();
        for i in (0..=(size - 1)).rev() {
            let scope = &self.scopes[i];
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, size - 1 - i)?;
                return Ok(());
            }
        }

        // Assume it's global
        Ok(())
    }

    fn resolve_expr_assign(&mut self, expr: &Expr) -> Result<(), String> {
        if let Expr::Assign { name, value } = expr {
            self.resolve_expr(value.as_ref())?;
            self.resolve_local(expr, name)?;
        } else {
            panic!("Wrong type in resolve assign");
        }

        Ok(())
    }
}
