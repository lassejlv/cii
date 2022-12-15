use crate::environment::Environment;
use crate::expr::LiteralValue;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    // globals: Environment,
    environment: Rc<RefCell<Environment>>,
}

fn clock_impl(_env: Rc<RefCell<Environment>>, _args: &Vec<LiteralValue>) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time")
        .as_millis();

    LiteralValue::Number(now as f64 / 1000.0)
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define(
            "clock".to_string(),
            LiteralValue::Callable {
                name: "clock".to_string(),
                arity: 0,
                fun: Rc::new(clock_impl),
            },
        );

        Self {
            // globals,
            //environment: Rc::new(RefCell::new(Environment::new())),
            environment: Rc::new(RefCell::new(globals)),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self { environment }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone())?;
                }
                Stmt::Print { expression } => {
                    let value = expression.evaluate(self.environment.clone())?;
                    println!("{}", value.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(self.environment.clone())?;

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
                    let truth_value = predicate.evaluate(self.environment.clone())?;
                    if truth_value.is_truthy() == LiteralValue::True {
                        let statements = vec![then.as_ref()];
                        self.interpret(statements)?;
                    } else if let Some(els_stmt) = els {
                        let statements = vec![els_stmt.as_ref()];
                        self.interpret(statements)?;
                    }
                }
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        let statements = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone())?;
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
                    let fun_impl = move |parent_env, args: &Vec<LiteralValue>| {
                        let mut clos_int = Interpreter::for_closure(parent_env);

                        for (i, arg) in args.iter().enumerate() {
                            clos_int
                                .environment
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), (*arg).clone());
                        }

                        for i in 0..(body.len() - 1) {
                            clos_int.interpret(vec![body[i].as_ref()]).expect(&format!(
                                "Evaluating failed inside {}",
                                name_clone
                            ));
                        }

                        let value;
                        match body[body.len() - 1].as_ref() {
                            Stmt::Expression { expression } => {
                                value = expression.evaluate(clos_int.environment.clone()).unwrap();
                            }
                            _ => todo!("Didnt get an expression"),
                        }

                        value
                    };

                    let callable = LiteralValue::Callable {
                        name: name.lexeme.clone(),
                        arity,
                        fun: Rc::new(fun_impl),
                    };

                    self.environment.borrow_mut().define(name.lexeme.clone(), callable);
                }
            };
        }

        Ok(())
    }
}
