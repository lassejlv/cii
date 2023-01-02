mod environment;
mod expr;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod tests;
use crate::interpreter::*;
use crate::parser::*;
use crate::resolver::*;
use crate::scanner::*;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::exit;
use std::rc::Rc;

pub fn run_file(path: &str) -> Result<(), String> {
    // let mut interpreter = Interpreter::new();
    match fs::read_to_string(path) {
        Err(msg) => return Err(msg.to_string()),
        Ok(contents) => return run_string(&contents),
    }
}

pub fn run_string(contents: &str) -> Result<(), String> {
    let interpreter = Rc::new(RefCell::new(Interpreter::new()));

    run(interpreter, contents)
}

fn run(interpreter: Rc<RefCell<Interpreter>>, contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;

    // for stmt in &stmts {
    //     println!("{stmt:?}");
    // }
    
    let mut resolver = Resolver::new(interpreter.clone());
    resolver.resolve_many(&stmts.iter().collect())?;
    // println!("#####################RESOLVE DONE###############");
    // for stmt in &stmts {
    //     println!("{stmt:?}");
    // }

    // Ok(())
    interpreter.borrow_mut().interpret(stmts.iter().collect())?;
    return Ok(());
}

fn run_prompt() -> Result<(), String> {
    let interpreter = Rc::new(RefCell::new(Interpreter::new()));
    loop {
        print!("> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(_) => return Err("Could not flush stdout".to_string()),
        }

        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        match handle.read_line(&mut buffer) {
            Ok(n) => {
                if n <= 1 {
                    return Ok(());
                }
            }
            Err(_) => return Err("Couldnt read line".to_string()),
        }

        println!("ECHO: {}", buffer);
        match run(interpreter.clone(), &buffer) {
            Ok(_) => (),
            Err(msg) => println!("{}", msg),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        match run_file(&args[1]) {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR:\n{}", msg);
                exit(1);
            }
        }
    } else if args.len() == 3 && args[1] == "e" {
        match run_string(&args[2]) {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR:\n{msg}");
                exit(1);
            }
        }
    } else if args.len() == 1 {
        match run_prompt() {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR\n{}", msg);
                exit(1);
            }
        }
    } else {
        println!("Usage: jlox [script]");
        exit(64);
    }
}
