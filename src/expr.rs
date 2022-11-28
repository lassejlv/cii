use crate::environment::Environment;
use crate::scanner;
use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue<'a> {
    Number(f32),
    StringValue(Vec<&'a str>),
    True,
    False,
    Nil,
}
use LiteralValue::*;

fn unwrap_as_f32(literal: Option<scanner::LiteralValue>) -> f32 {
    match literal {
        Some(scanner::LiteralValue::IntValue(x)) => x as f32,
        Some(scanner::LiteralValue::FValue(x)) => x as f32,
        _ => panic!("Could not unwrap as f32"),
    }
}

fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> &str {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s,
        Some(scanner::LiteralValue::IdentifierVal(s)) => s,
        _ => panic!("Could not unwrap as string"),
    }
}

impl<'a> LiteralValue<'a> {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(x) => {
                let mut s = String::new();
                for substring in x {
                    s.push_str(substring);
                }
                s
            }
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True => "Boolean",
            LiteralValue::False => "Boolean",
            LiteralValue::Nil => "nil",
        }
    }

    pub fn from_token(token: Token<'a>) -> Self {
        match token.token_type {
            TokenType::Number => Self::Number(unwrap_as_f32(token.literal)),
            TokenType::StringLit => Self::StringValue(vec![unwrap_as_string(token.literal)]),
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

    pub fn is_falsy(&self) -> LiteralValue<'a> {
        match self {
            Number(x) => {
                if *x == 0.0 as f32 {
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
        }
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expression: Box<Expr<'a>>,
    },
    Literal {
        value: LiteralValue<'a>,
    },
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Variable {
        name: Token<'a>,
    },
}

impl<'a> Expr<'a> {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Assign { name, value } => format!("({name:?} = {}", value.to_string()),
            Expr::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Grouping { expression } => format!("(group {})", (*expression).to_string()),
            Expr::Literal { value } => format!("{}", value.to_string()),
            Expr::Unary { operator, right } => {
                let operator_str = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {})", operator_str, right_str)
            }
            Expr::Variable { name } => format!("(var {})", name.lexeme),
        }
    }

    pub fn evaluate(&'a self, environment: &mut Environment<'a>) -> Result<LiteralValue<'a>, String> {
        match self {
            Expr::Assign { name, value } => {
                let get_value = environment.get(&name.lexeme);
                match get_value {
                    Some(_) => {
                        let new_value = (*value).evaluate(environment)?;
                        environment.define(name.lexeme.clone(), new_value.clone());
                        Ok(new_value)
                    }
                    None => Err(format!("Variable {name:?} has not been declared")),
                }
            }
            Expr::Variable { name } => match environment.get(&name.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Variable '{}' has not been declared", name.lexeme)),
            },
            Expr::Literal { value } => Ok((*value).clone()),
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = right.evaluate(environment)?;

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
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment)?;
                let right = right.evaluate(environment)?;

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
                        let mut strings = vec![];
                        strings.extend(s1);
                        strings.extend(s2);
                        Ok(StringValue(strings))
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

    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::Expr::*;
    use super::LiteralValue::*;
    use super::*;
    #[test]
    fn pretty_print_ast() {
        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-",
            literal: None,
            line_number: 0,
        };
        let onetwothree = Literal {
            value: Number(123.0),
        };
        let group = Grouping {
            expression: Box::from(Literal {
                value: Number(45.67),
            }),
        };
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*",
            literal: None,
            line_number: 0,
        };
        let ast = Binary {
            left: Box::from(Unary {
                operator: minus_token,
                right: Box::from(onetwothree),
            }),
            operator: multi,
            right: Box::from(group),
        };

        let result = ast.to_string();
        assert_eq!(result, "(* (- 123) (group 45.67))");
    }
}
