use crate::environment::Environment;
use crate::expr::LiteralValue;
use crate::scanner::Token;
use crate::stmt::Stmt;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub specials: HashMap<String, LiteralValue>,
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            specials: HashMap::new(),
            environment: Environment::new(HashMap::new()),
        }
    }

    pub fn resolve(&mut self, locals: HashMap<usize, usize>) {
        self.environment.resolve(locals);
    }

    fn for_closure(
        parent: Environment,
    ) -> Self {
        let environment = parent.enclose(); 
        Self {
            specials: HashMap::new(),
            environment,
        }
    }

    pub fn for_anon(
        parent: Environment,
    ) -> Self {
        let env = parent.enclose();
        Self {
            specials: HashMap::new(),
            environment: env,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone())?;
                }
                Stmt::Print { expression } => {
                    let value =
                        expression.evaluate(self.environment.clone())?;
                    println!("{}", value.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let value =
                        initializer.evaluate(self.environment.clone())?;
                    self.environment
                        .define(name.lexeme.clone(), value);
                }
                Stmt::Block { statements } => {
                    let new_environment = self.environment.enclose();
                    
                    //     Environment::new();
                    // new_environment.enclosing = Some(Box::new(self.environment.clone()));
                    let old_environment = self.environment.clone();
                    self.environment = new_environment;
                    let block_result =
                        self.interpret((*statements).iter().map(|b| b.as_ref()).collect());
                    self.environment = old_environment;
                    // self.environment = self.environment.enclosing.unwrap();
                    block_result?;
                }
                Stmt::Class { name, methods } => {
                    self.environment
                        .define(name.lexeme.clone(), LiteralValue::Nil);

                    let mut methods_map = HashMap::new();
                    for method in methods {
                        if let Stmt::Function { name, params: _, body: _ } = method.as_ref() {
                            let function = self.make_function(method);
                            methods_map.insert(name.lexeme.clone(), function);
                        } else {
                            panic!("Something that was not a function was in the methods of a class");
                        }
                    }

                    let klass = LiteralValue::LoxClass {
                        name: name.lexeme.clone(),
                        methods: methods_map,
                    };

                    if !self
                        .environment
                        .assign_global(&name.lexeme, klass)
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
                        predicate.evaluate(self.environment.clone())?;
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
                        condition.evaluate(self.environment.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        let statements = vec![body.as_ref()];
                        self.interpret(statements)?;
                        flag = condition.evaluate(self.environment.clone())?;
                    }
                }
                Stmt::Function { name, params: _, body: _ } => {
                    let callable = self.make_function(stmt);
                    self.environment
                        .define(name.lexeme.clone(), callable);
                }
                Stmt::ReturnStmt { keyword: _, value } => {
                    let eval_val;
                    if let Some(value) = value {
                        eval_val = value.evaluate(self.environment.clone())?;
                    } else {
                        eval_val = LiteralValue::Nil;
                    }
                    self.specials
                        .insert("return".to_string(), eval_val);
                }
            };
        }

        Ok(())
    }

    fn make_function(&self, fn_stmt: &Stmt) -> LiteralValue {
        if let Stmt::Function { name, params, body } = fn_stmt {
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
            let fun_impl = move |args: &Vec<LiteralValue>| {
                let mut clos_int =
                    Interpreter::for_closure(parent_env.clone());

                for (i, arg) in args.iter().enumerate() {
                    clos_int
                        .environment
                        .define(params[i].lexeme.clone(), (*arg).clone());
                }

                for i in 0..(body.len()) {
                    clos_int
                        .interpret(vec![body[i].as_ref()])
                        .expect(&format!("Evaluating failed inside {}", name_clone));

                    if let Some(value) = clos_int.specials.get("return") {
                        return value.clone();
                    }
                }

                LiteralValue::Nil
            };

            LiteralValue::Callable {
                name: name.lexeme.clone(),
                arity,
                fun: Rc::new(fun_impl),
            }
        } else {
            panic!("Tried to make a function from a non-function statement");
        }
    }
}
