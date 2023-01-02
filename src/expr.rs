use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::scanner;
use crate::scanner::{Token, TokenType};
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
    Callable {
        name: String,
        arity: usize,
        fun: Rc<dyn Fn(&Vec<LiteralValue>) -> LiteralValue>,
    },
}
use LiteralValue::*;

impl std::fmt::Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number(x), Number(y)) => x == y,
            (
                Callable {
                    name,
                    arity,
                    fun: _,
                },
                Callable {
                    name: name2,
                    arity: arity2,
                    fun: _,
                },
            ) => name == name2 && arity == arity2,
            (StringValue(x), StringValue(y)) => x == y,
            (True, True) => true,
            (False, False) => true,
            (Nil, Nil) => true,
            _ => false,
        }
    }
}

fn unwrap_as_f64(literal: Option<scanner::LiteralValue>) -> f64 {
    match literal {
        Some(scanner::LiteralValue::FValue(x)) => x as f64,
        _ => panic!("Could not unwrap as f64"),
    }
}

fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> String {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
        _ => panic!("Could not unwrap as string"),
    }
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(x) => format!("\"{}\"", x),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Nil => "nil".to_string(),
            LiteralValue::Callable {
                name,
                arity,
                fun: _,
            } => format!("{name}/{arity}"),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True => "Boolean",
            LiteralValue::False => "Boolean",
            LiteralValue::Nil => "nil",
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => "Callable",
        }
    }

    pub fn from_token(token: Token) -> Self {
        match token.token_type {
            TokenType::Number => Self::Number(unwrap_as_f64(token.literal)),
            TokenType::StringLit => Self::StringValue(unwrap_as_string(token.literal)),
            TokenType::False => Self::False,
            TokenType::True => Self::True,
            TokenType::Nil => Self::Nil,
            _ => panic!("Could not create LiteralValue from {:?}", token),
        }
    }

    pub fn from_bool(b: bool) -> Self {
        if b {
            True
        } else {
            False
        }
    }

    pub fn is_falsy(&self) -> LiteralValue {
        match self {
            Number(x) => {
                if *x == 0.0 as f64 {
                    True
                } else {
                    False
                }
            }
            StringValue(s) => {
                if s.len() == 0 {
                    True
                } else {
                    False
                }
            }
            True => False,
            False => True,
            Nil => True,
            Callable {
                name: _,
                arity: _,
                fun: _,
            } => panic!("Cannot use Callable as a falsy value"),
        }
    }

    pub fn is_truthy(&self) -> LiteralValue {
        match self {
            Number(x) => {
                if *x == 0.0 as f64 {
                    False
                } else {
                    True
                }
            }
            StringValue(s) => {
                if s.len() == 0 {
                    False
                } else {
                    True
                }
            }
            True => True,
            False => False,
            Nil => False,
            Callable {
                name: _,
                arity: _,
                fun: _,
            } => panic!("Can not use callable as a truthy value"),
        }
    }
}

use crate::stmt::Stmt;

#[derive(Clone)]
pub enum Expr {
    AnonFunction {
        id: usize,
        paren: Token,
        arguments: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    Assign {
        id: usize,
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        id: usize,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    // 2 + 2 |> f
    Call { // x |> f -> Call { id, f, paren (pipe), arguments: [x]}
        id: usize,
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        id: usize,
        expression: Box<Expr>,
    },
    Literal {
        id: usize,
        value: LiteralValue,
    },
    Logical {
        id: usize,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        id: usize,
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        id: usize,
        name: Token,
    },
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.get_id(), self.to_string())
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state)
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        let ptr = std::ptr::addr_of!(self);
        let ptr2 = std::ptr::addr_of!(other);
        ptr == ptr2
    }
}

impl Eq for Expr {}


impl Expr {
    pub fn get_id(&self) -> usize {
        match self {
            Expr::AnonFunction {
                id,
                paren: _,
                arguments: _,
                body: _,
            } => *id,
            Expr::Assign { id, name: _, value: _ } => *id,
            Expr::Binary {
                id,
                left: _,
                operator: _,
                right: _,
            } => *id,
            
            Expr::Call {
                id,
                callee: _,
                paren: _,
                arguments: _,
            } => *id,
            Expr::Grouping { id, expression: _, } => *id,
            Expr::Literal { id, value: _ } => *id,
            Expr::Logical {
                id,
                left: _,
                operator: _,
                right: _,
            } => *id,
            Expr::Unary {
                id,
                operator: _,
                right: _,
            } => *id,
            Expr::Variable { id, name: _ } => *id,
        }
    }

}

impl Expr {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            Expr::AnonFunction {
                id: _,
                paren: _,
                arguments,
                body: _,
            } => format!("anon/{}", arguments.len()),
            Expr::Assign { id: _, name, value } => format!("({name:?} = {}", value.to_string()),
            Expr::Binary {
                id: _,
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Call {
                id: _,
                callee,
                paren: _,
                arguments,
            } => format!("({} {:?})", (*callee).to_string(), arguments),
            Expr::Grouping { id: _, expression } => {
                format!("(group {})", (*expression).to_string())
            }
            Expr::Literal { id: _, value } => format!("{}", value.to_string()),
            Expr::Logical {
                id: _,
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.to_string(),
                left.to_string(),
                right.to_string()
            ),
            Expr::Unary {
                id: _,
                operator,
                right,
            } => {
                let operator_str = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {})", operator_str, right_str)
            }
            Expr::Variable { id: _, name } => format!("(var {})", name.lexeme),
        }
    }

    pub fn evaluate(
        &self,
        environment: Rc<RefCell<Environment>>,
        distance: Option<usize>,
    ) -> Result<LiteralValue, String> {
        match self {
            Expr::AnonFunction {
                id: _,
                paren,
                arguments,
                body,
            } => {
                // We have to clone everything so the borrow checker doesnt get scared about us taking ownership of the values in the Expr
                let arity = arguments.len();
                let env = environment.clone();
                let arguments: Vec<Token> = arguments.iter().map(|t| (*t).clone()).collect();
                let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                let paren = paren.clone();

                let fun_impl = move |args: &Vec<LiteralValue>| {
                    let mut anon_int = Interpreter::for_anon(env.clone());
                    for (i, arg) in args.iter().enumerate() {
                        anon_int
                            .environment
                            .borrow_mut()
                            .define(arguments[i].lexeme.clone(), (*arg).clone());
                    }

                    for i in 0..(body.len()) {
                        anon_int.interpret(vec![&body[i]]).expect(&format!(
                            "Evaluating failed inside anon function at line {}",
                            paren.line_number
                        ));

                        if let Some(value) = anon_int.specials.borrow().get("return") {
                            return value.clone();
                        }
                    }

                    LiteralValue::Nil
                };

                Ok(Callable {
                    name: "anon_function".to_string(),
                    arity,
                    fun: Rc::new(fun_impl),
                })
            }
            Expr::Assign { id: _, name, value } => {
                let new_value = (*value).evaluate(environment.clone(), distance)?;
                let assign_success =
                    environment
                        .borrow_mut()
                        .assign(&name.lexeme, new_value.clone(), distance);

                if assign_success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared", name.lexeme))
                }
            }
            Expr::Variable { id: _, name } => {
                match environment.borrow().get(&name.lexeme, distance) {
                    Some(value) => Ok(value.clone()),
                    None => Err(format!("Variable '{}' has not been declared", name.lexeme)),
                }
            }
            Expr::Call {
                id: _,
                callee,
                paren: _,
                arguments,
            } => {
                // Look up function definition in environment
                let callable = (*callee).evaluate(environment.clone(), distance)?;
                match callable {
                    Callable { name, arity, fun } => {
                        // Do some checking (correct number of args?)
                        if arguments.len() != arity {
                            return Err(format!(
                                "Callable {} expected {} arguments but got {}",
                                name,
                                arity,
                                arguments.len()
                            ));
                        }
                        // Evaluate arguments
                        let mut arg_vals = vec![];
                        for arg in arguments {
                            let val = arg.evaluate(environment.clone(), distance)?;
                            arg_vals.push(val);
                        }
                        // Apply to arguments
                        Ok(fun(&arg_vals))
                    }
                    other => Err(format!("{} is not callable", other.to_type())),
                }
            }
            Expr::Literal { id: _, value } => Ok((*value).clone()),
            Expr::Logical {
                id: _,
                left,
                operator,
                right,
            } => match operator.token_type {
                TokenType::Or => {
                    let lhs_value = left.evaluate(environment.clone(), distance)?;
                    let lhs_true = lhs_value.is_truthy();
                    if lhs_true == True {
                        Ok(lhs_value)
                    } else {
                        right.evaluate(environment.clone(), distance)
                    }
                }
                TokenType::And => {
                    let lhs_value = left.evaluate(environment.clone(), distance)?;
                    let lhs_true = lhs_value.is_truthy();
                    if lhs_true == False {
                        Ok(lhs_true)
                    } else {
                        right.evaluate(environment.clone(), distance)
                    }
                }
                ttype => Err(format!("Invalid token in logical expression: {}", ttype)),
            },
            Expr::Grouping { id: _, expression } => expression.evaluate(environment, distance),
            Expr::Unary {
                id: _,
                operator,
                right,
            } => {
                let right = right.evaluate(environment, distance)?;

                match (&right, operator.token_type) {
                    (Number(x), TokenType::Minus) => Ok(Number(-x)),
                    (_, TokenType::Minus) => {
                        Err(format!("Minus not implemented for {}", right.to_type()))
                    }
                    (any, TokenType::Bang) => Ok(any.is_falsy()),
                    (_, ttype) => Err(format!("{} is not a valid unary operator", ttype)),
                }
            }
            Expr::Binary {
                id: _,
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment.clone(), distance)?;
                let right = right.evaluate(environment.clone(), distance)?;

                match (&left, operator.token_type, &right) {
                    (Number(x), TokenType::Plus, Number(y)) => Ok(Number(x + y)),
                    (Number(x), TokenType::Minus, Number(y)) => Ok(Number(x - y)),
                    (Number(x), TokenType::Star, Number(y)) => Ok(Number(x * y)),
                    (Number(x), TokenType::Slash, Number(y)) => Ok(Number(x / y)),
                    (Number(x), TokenType::Greater, Number(y)) => {
                        Ok(LiteralValue::from_bool(x > y))
                    }
                    (Number(x), TokenType::GreaterEqual, Number(y)) => {
                        Ok(LiteralValue::from_bool(x >= y))
                    }
                    (Number(x), TokenType::Less, Number(y)) => Ok(LiteralValue::from_bool(x < y)),
                    (Number(x), TokenType::LessEqual, Number(y)) => {
                        Ok(LiteralValue::from_bool(x <= y))
                    }

                    (StringValue(_), op, Number(_)) => {
                        Err(format!("{} is not defined for string and number", op))
                    }
                    (Number(_), op, StringValue(_)) => {
                        Err(format!("{} is not defined for string and number", op))
                    }

                    (StringValue(s1), TokenType::Plus, StringValue(s2)) => {
                        Ok(StringValue(format!("{}{}", s1, s2)))
                    }

                    (x, TokenType::BangEqual, y) => Ok(LiteralValue::from_bool(x != y)),
                    (x, TokenType::EqualEqual, y) => Ok(LiteralValue::from_bool(x == y)),
                    (StringValue(s1), TokenType::Greater, StringValue(s2)) => {
                        Ok(LiteralValue::from_bool(s1 > s2))
                    }
                    (StringValue(s1), TokenType::GreaterEqual, StringValue(s2)) => {
                        Ok(LiteralValue::from_bool(s1 >= s2))
                    }
                    (StringValue(s1), TokenType::Less, StringValue(s2)) => {
                        Ok(LiteralValue::from_bool(s1 < s2))
                    }
                    (StringValue(s1), TokenType::LessEqual, StringValue(s2)) => {
                        Ok(LiteralValue::from_bool(s1 <= s2))
                    }
                    (x, ttype, y) => Err(format!(
                        "{} is not implemented for operands {:?} and {:?}",
                        ttype, x, y
                    )),
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::Expr::*;
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn pretty_print_ast() {
        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            literal: None,
            line_number: 0,
        };
        let onetwothree = Literal {
            id: 0,
            value: Number(123.0),
        };
        let group = Grouping {
            id: 1,
            expression: Box::from(Literal {
                id: 2,
                value: Number(45.67),
            }),
        };
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*".to_string(),
            literal: None,
            line_number: 0,
        };
        let ast = Binary {
            id: 3,
            left: Box::from(Unary {
                id: 4,
                operator: minus_token,
                right: Box::from(onetwothree),
            }),
            operator: multi,
            right: Box::from(group),
        };

        let result = ast.to_string();
        assert_eq!(result, "(* (- 123) (group 45.67))");
    }

    #[test]
    fn expr_hashable() {
        let mut locals = HashMap::new();
        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            literal: None,
            line_number: 0,
        };
        let onetwothree = Literal {
            id: 0,
            value: Number(123.0),
        };
        let group = Grouping {
            id: 1,
            expression: Box::from(Literal {
                id: 2,
                value: Number(45.67),
            }),
        };
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*".to_string(),
            literal: None,
            line_number: 0,
        };
        let expr = Binary {
            id: 3,
            left: Box::from(Unary {
                id: 4,
                operator: minus_token,
                right: Box::from(onetwothree),
            }),
            operator: multi,
            right: Box::from(group),
        };

        let addr = std::ptr::addr_of!(expr) as usize;
        locals.insert(addr, 0);

        if let None = locals.get(&addr) {
            panic!("Failed");
        }
    }
}
