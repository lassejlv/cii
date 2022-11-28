use crate::expr::{Expr, Expr::*, LiteralValue};
use crate::scanner::{Token, TokenType, TokenType::*};
use crate::stmt::Stmt;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        let mut errs = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(msg) => {
                    errs.push(msg);
                    self.synchronize();
                }
            }
        }

        if errs.len() == 0 {
            Ok(stmts)
        } else {
            Err(errs.join("\n"))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(Var) {
            match self.var_declaration() {
                Ok(stmt) => Ok(stmt),
                Err(msg) => {
                    // self.synchronize();
                    Err(msg)
                }
            }
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let token = self.consume(Identifier, "Expected variable name")?;

        let initializer;
        if self.match_token(Equal) {
            initializer = self.expression()?;
        } else {
            initializer = Literal {
                value: LiteralValue::Nil,
            };
        }

        self.consume(Semicolon, "Expected ';' after variable declaration")?;

        Ok(Stmt::Var {
            name: token,
            initializer: initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.match_token(Equal) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Variable { name } => Ok(Assign {
                    name,
                    value: Box::from(value),
                }),
                _ => Err("Invalid assignment target.".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.match_tokens(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let rhs = self.comparison()?;
            expr = Binary {
                left: Box::from(expr),
                operator: operator,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            let op = self.previous();
            let rhs = self.term()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[Minus, Plus]) {
            let op = self.previous();
            let rhs = self.factor()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[Slash, Star]) {
            let op = self.previous();
            let rhs = self.unary()?;
            expr = Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[Bang, Minus]) {
            let op = self.previous();
            let rhs = self.unary()?;
            Ok(Unary {
                operator: op,
                right: Box::from(rhs),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();
        let result;
        match token.token_type {
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')'")?;
                result = Grouping {
                    expression: Box::from(expr),
                };
            }
            False | True | Nil | Number | StringLit => {
                self.advance();
                result = Literal {
                    value: LiteralValue::from_token(token),
                }
            }
            Identifier => {
                self.advance();
                result = Variable {
                    name: self.previous(),
                };
            }
            _ => return Err("Expected expression".to_string()),
        }

        Ok(result)
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            Err(msg.to_string())
        }
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            if self.peek().token_type == typ {
                self.advance();
                true
            } else {
                false
            }
        }
    }

    fn match_tokens(&mut self, typs: &[TokenType]) -> bool {
        for typ in typs {
            if self.match_token(*typ) {
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&mut self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == Eof
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }

            match self.peek().token_type {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{LiteralValue::*, Scanner};

    #[test]
    fn test_addition() {
        let one = Token {
            token_type: Number,
            lexeme: "1",
            literal: Some(IntValue(1)),
            line_number: 0,
        };
        let plus = Token {
            token_type: Plus,
            lexeme: "+",
            literal: None,
            line_number: 0,
        };
        let two = Token {
            token_type: Number,
            lexeme: "2",
            literal: Some(IntValue(2)),
            line_number: 0,
        };
        let semicol = Token {
            token_type: Semicolon,
            lexeme: ";",
            literal: None,
            line_number: 0,
        };

        let tokens = vec![one, plus, two, semicol];
        let mut parser = Parser::new(tokens);

        let parsed_expr = parser.parse().unwrap();
        assert_eq!(parsed_expr.len(), 1);
        let string_expr = parsed_expr[0].tostring();

        assert_eq!(string_expr, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 2 == 5 + 7";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let tokens = scanner.tokens;
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        assert_eq!(parsed_expr.len(), 1);
        let string_expr = parsed_expr[0].tostring();

        assert_eq!(string_expr, "(== (+ 1 2) (+ 5 7))");
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1 == (2 + 2)";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        let tokens = scanner.tokens;
        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse().unwrap();
        assert_eq!(parsed_expr.len(), 1);
        let string_expr = parsed_expr[0].tostring();

        assert_eq!(string_expr, "(== 1 (group (+ 2 2)))");
    }
}
