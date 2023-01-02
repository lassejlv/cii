use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[allow(dead_code)]
pub struct Resolver {
    pub interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: vec![],
        }
    }

    pub fn resolve(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements: _ } => self.resolve_block(stmt)?,
            Stmt::Var {
                name: _,
                initializer: _,
            } => self.resolve_var(stmt)?,
            Stmt::Function {
                name: _,
                params: _,
                body: _,
            } => self.resolve_function(stmt)?,
            Stmt::Expression { expression } => self.resolve_expr(expression, None)?,
            Stmt::IfStmt {
                predicate: _,
                then: _,
                els: _,
            } => self.resolve_if_stmt(stmt)?,
            Stmt::Print { expression } => self.resolve_expr(expression, None)?,
            Stmt::ReturnStmt {
                keyword: _,
                value: None,
            } => (),
            Stmt::ReturnStmt {
                keyword: _,
                value: Some(value),
            } => self.resolve_expr(value, None)?,
            Stmt::WhileStmt { condition, body } => {
                self.resolve_expr(condition, Some(condition.get_id()))?;
                self.resolve(body.as_ref())?;
            }
        }
        Ok(())
    }

    pub fn resolve_many(&mut self, stmts: &Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            self.resolve(stmt)?;
        }

        Ok(())
    }

    fn resolve_block(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_many(&statements.iter().map(|b| b.as_ref()).collect())?;
                self.end_scope();
            }
            _ => panic!("Wrong type"),
        }

        Ok(())
    }

    fn resolve_var(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::Var { name, initializer } = stmt {
            self.declare(name);
            self.resolve_expr(initializer, Some(initializer.get_id()))?;
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

            self.resolve_function_helper(params, &body.iter().map(|b| b.as_ref()).collect(), None)
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
            self.resolve_expr(predicate, None)?;
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
        body: &Vec<&Stmt>,
        _resolve_id: Option<usize>,
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
        let size = self.scopes.len();
        if self.scopes.is_empty() {
            return;
        }

        self.scopes[size - 1].insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let size = self.scopes.len();
        self.scopes[size - 1].insert(name.lexeme.clone(), true);
    }

    fn resolve_expr(&mut self, expr: &Expr, resolve_id: Option<usize>) -> Result<(), String> {
        match expr {
            Expr::Variable { id: _, name: _ } => self.resolve_expr_var(expr, resolve_id),
            Expr::Assign {
                id: _,
                name: _,
                value: _,
            } => self.resolve_expr_assign(expr, resolve_id),
            Expr::Binary {
                id: _,
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left, resolve_id)?;
                self.resolve_expr(right, resolve_id)
            }
            Expr::Call {
                id: _,
                callee: _,
                paren: _,
                arguments,
            } => {
                //self.resolve_expr(callee.as_ref())?;
                self.resolve_expr_var(expr, resolve_id)?;
                for arg in arguments {
                    self.resolve_expr(arg, resolve_id)?;
                }

                Ok(())
            }
            Expr::Grouping { id: _, expression } => self.resolve_expr(expression, resolve_id),
            Expr::Literal { id: _, value: _ } => Ok(()),
            Expr::Logical {
                id: _,
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left, resolve_id)?;
                self.resolve_expr(right, resolve_id)
            }
            Expr::Unary {
                id: _,
                operator: _,
                right,
            } => self.resolve_expr(right, resolve_id),
            Expr::AnonFunction {
                id: _,
                paren: _,
                arguments,
                body,
            } => self.resolve_function_helper(
                arguments,
                &body.iter().map(|b| b.as_ref()).collect(),
                resolve_id,
            ),
        }
    }

    fn resolve_expr_var(&mut self, expr: &Expr, resolve_id: Option<usize>) -> Result<(), String> {
        match expr {
            Expr::Variable { id: _, name } => {
                if !self.scopes.is_empty() {
                    if let Some(false) = self.scopes[self.scopes.len() - 1].get(&name.lexeme) {
                        return Err("Can't read local variable in its own initializer".to_string());
                    }
                }

                self.resolve_local(expr, name, resolve_id)
            }
            Expr::Call {
                id: _,
                callee,
                paren: _,
                arguments: _,
            } => match callee.as_ref() {
                Expr::Variable { id: _, name } => self.resolve_local(expr, &name, resolve_id),
                _ => panic!("Wrong type in resolve_expr_var"),
            },
            _ => panic!("Wrong type in resolve_expr_var"),
        }
    }

    fn resolve_local(
        &mut self,
        expr: &Expr,
        name: &Token,
        resolve_id: Option<usize>,
    ) -> Result<(), String> {
        let size = self.scopes.len();
        if size == 0 {
            return Ok(());
        }

        for i in (0..=(size - 1)).rev() {
            let scope = &self.scopes[i];
            if scope.contains_key(&name.lexeme) {
                let id_to_use = match resolve_id {
                    None => expr.get_id(),
                    Some(id) => id,
                };

                // println!("Name: {}, ID: {}, Dist: {}", name.lexeme, id_to_use, size-1-i);
                self.interpreter
                    .borrow_mut()
                    .resolve(id_to_use, size - 1 - i)?;
                return Ok(());
            }
        }

        // Assume it's global
        Ok(())
    }

    fn resolve_expr_assign(
        &mut self,
        expr: &Expr,
        resolve_id: Option<usize>,
    ) -> Result<(), String> {
        if let Expr::Assign { id: _, name, value } = expr {
            self.resolve_expr(value.as_ref(), resolve_id)?;
            self.resolve_local(expr, name, resolve_id)?;
        } else {
            panic!("Wrong type in resolve assign");
        }

        Ok(())
    }
}
