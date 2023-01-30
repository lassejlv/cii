use crate::environment::Environment;
use crate::expr::LiteralValue;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub specials: Rc<RefCell<HashMap<String, LiteralValue>>>,
    pub environment: Environment,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment: Environment::new(),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn for_closure(
        parent: Environment,
        locals: Rc<RefCell<HashMap<usize, usize>>>,
    ) -> Self {
        let mut environment = Environment::new();
        environment.enclosing = Some(Box::new(parent));

        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment,
            locals: locals,
        }
    }

    pub fn for_anon(
        parent: Environment,
        locals: Rc<RefCell<HashMap<usize, usize>>>,
    ) -> Self {
        let mut env = Environment::new();
        env.enclosing = Some(Box::new(parent));
        Self {
            specials: Rc::new(RefCell::new(HashMap::new())),
            environment: env,
            locals,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone(), self.locals.clone())?;
                }
                Stmt::Print { expression } => {
                    let value =
                        expression.evaluate(self.environment.clone(), self.locals.clone())?;
                    println!("{}", value.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let value =
                        initializer.evaluate(self.environment.clone(), self.locals.clone())?;
                    self.environment
                        .define(name.lexeme.clone(), value);
                }
                Stmt::Block { statements } => {
                    let mut new_environment = Environment::new();
                    new_environment.enclosing = Some(Box::new(self.environment.clone()));
                    let old_environment = self.environment.clone();
                    self.environment = new_environment;
                    let block_result =
                        self.interpret((*statements).iter().map(|b| b.as_ref()).collect());
                    self.environment = old_environment;

                    block_result?;
                }
                Stmt::Class { name, methods: _ } => {
                    self.environment
                        .define(name.lexeme.clone(), LiteralValue::Nil);
                    let klass = LiteralValue::LoxClass {
                        name: name.lexeme.clone(),
                    };

                    if !self
                        .environment
                        .assign(&name.lexeme, klass, None)
                    {
                        return Err(format!("Class definition failed for {}", name.lexeme));
                    }
                }
                Stmt::IfStmt {
                    predicate,
                    then,
                    els,
                } => {
                    let truth_value =
                        predicate.evaluate(self.environment.clone(), self.locals.clone())?;
                    if truth_value.is_truthy() == LiteralValue::True {
                        let statements = vec![then.as_ref()];
                        self.interpret(statements)?;
                    } else if let Some(els_stmt) = els {
                        let statements = vec![els_stmt.as_ref()];
                        self.interpret(statements)?;
                    }
                }
                Stmt::WhileStmt { condition, body } => {
                    let mut flag =
                        condition.evaluate(self.environment.clone(), self.locals.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        let statements = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone(), self.locals.clone())?;
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
                        .define(name.lexeme.clone(), callable);
                }
                Stmt::ReturnStmt { keyword: _, value } => {
                    let eval_val;
                    if let Some(value) = value {
                        eval_val = value.evaluate(self.environment.clone(), self.locals.clone())?;
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
}
