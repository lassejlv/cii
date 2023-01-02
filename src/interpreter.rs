use crate::environment::Environment;
use crate::expr::Expr;
use crate::expr::LiteralValue;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub specials: Rc<RefCell<HashMap<String, LiteralValue>>>,
    pub environment: Rc<RefCell<Environment>>,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment: Rc::new(RefCell::new(Environment::new())),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn for_closure(
        parent: Rc<RefCell<Environment>>,
        locals: Rc<RefCell<HashMap<usize, usize>>>,
    ) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment,
            locals: locals,
        }
    }

    pub fn for_anon(parent: Rc<RefCell<Environment>>) -> Self {
        let mut env = Environment::new();
        env.enclosing = Some(parent);
        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment: Rc::new(RefCell::new(env)),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    let distance = self.get_distance(&expression);
                    expression.evaluate(self.environment.clone(), distance)?;
                }
                Stmt::Print { expression } => {
                    let distance = self.get_distance(&expression);
                    let value = expression.evaluate(self.environment.clone(), distance)?;
                    println!("{}", value.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let distance = self.get_distance(&initializer);
                    let value = initializer.evaluate(self.environment.clone(), distance)?;
                    self.environment
                        .borrow_mut()
                        .define(name.lexeme.clone(), value);
                }
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(self.environment.clone());
                    let old_environment = self.environment.clone();
                    self.environment = Rc::new(RefCell::new(new_environment));
                    let block_result =
                        self.interpret((*statements).iter().map(|b| b.as_ref()).collect());
                    self.environment = old_environment;

                    block_result?;
                }
                Stmt::IfStmt {
                    predicate,
                    then,
                    els,
                } => {
                    let distance = self.get_distance(&predicate);
                    let truth_value = predicate.evaluate(self.environment.clone(), distance)?;
                    if truth_value.is_truthy() == LiteralValue::True {
                        let statements = vec![then.as_ref()];
                        self.interpret(statements)?;
                    } else if let Some(els_stmt) = els {
                        let statements = vec![els_stmt.as_ref()];
                        self.interpret(statements)?;
                    }
                }
                Stmt::WhileStmt { condition, body } => {
                    let distance = self.get_distance(&condition);
                    let mut flag = condition.evaluate(self.environment.clone(), distance)?;
                    while flag.is_truthy() == LiteralValue::True {
                        let statements = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone(), distance)?;
                    }
                }
                Stmt::Function { name, params, body } => {
                    // Function decl
                    let arity = params.len();
                    // Function impl:
                    // Bind list of input values to names in params
                    // Add those bindings to the environment used to execute body
                    // Then execute body

                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                    let name_clone = name.lexeme.clone();
                    // TODO Make a struct that contains data for evaluation
                    // and which implements Fn

                    let parent_env = self.environment.clone();
                    let parent_locals = self.locals.clone();
                    let fun_impl = move |args: &Vec<LiteralValue>| {
                        let mut clos_int =
                            Interpreter::for_closure(parent_env.clone(), parent_locals.clone());

                        for (i, arg) in args.iter().enumerate() {
                            clos_int
                                .environment
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), (*arg).clone());
                        }

                        for i in 0..(body.len()) {
                            clos_int
                                .interpret(vec![body[i].as_ref()])
                                .expect(&format!("Evaluating failed inside {}", name_clone));

                            if let Some(value) = clos_int.specials.borrow().get("return") {
                                return value.clone();
                            }
                        }

                        LiteralValue::Nil
                    };

                    let callable = LiteralValue::Callable {
                        name: name.lexeme.clone(),
                        arity,
                        fun: Rc::new(fun_impl),
                    };

                    self.environment
                        .borrow_mut()
                        .define(name.lexeme.clone(), callable);
                }
                Stmt::ReturnStmt { keyword: _, value } => {
                    let eval_val;
                    if let Some(value) = value {
                        let distance = self.get_distance(value);
                        eval_val = value.evaluate(self.environment.clone(), distance)?;
                    } else {
                        eval_val = LiteralValue::Nil;
                    }
                    self.specials
                        .borrow_mut()
                        .insert("return".to_string(), eval_val);
                }
            };
        }

        Ok(())
    }

    // TODO Try the trick with addresses again
    pub fn resolve(&mut self, id: usize, steps: usize) -> Result<(), String> {
        self.locals.borrow_mut().insert(id, steps);
        Ok(())
    }

    fn get_distance(&self, expr: &Expr) -> Option<usize> {
        let dist = self.locals.borrow().get(&expr.get_id()).copied();
        dist
    }
}
