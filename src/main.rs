mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod environment;
use crate::interpreter::*;
use crate::parser::*;
use crate::scanner::*;

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::exit;

fn run_file(path: &str) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    match fs::read_to_string(path) {
        Err(msg) => return Err(msg.to_string()),
        Ok(contents) => return run(&mut interpreter, &contents),
    }
}

fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), String> {
    let mut scanner = Scanner::new(contents);
    scanner.scan_tokens()?;
    let tokens = scanner.tokens;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;
    interpreter.interpret(stmts);
    return Ok(());
}

fn run_prompt() -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();
    loop {
        print!("> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(_) => return Err("Could not flush stdout".to_string()),
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let current_length = buffer.len();
        match handle.read_line(&mut buffer) {
            Ok(n) => {
                if n < 1 {
                    println!("");
                    return Ok(());
                }
            }
            Err(_) => return Err("Couldnt read line".to_string()),
        }
        
        println!("ECHO: {}", &buffer[current_length..]);
        match run(&mut interpreter, &buffer[current_length..]) {
            Ok(_) => (),
            Err(msg) => println!("{}", msg),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: jlox [script]");
        exit(64);
    } else if args.len() == 2 {
        match run_file(&args[1]) {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR:\n{}", msg);
                exit(1);
            }
        }
    } else {
        match run_prompt() {
            Ok(_) => exit(0),
            Err(msg) => {
                println!("ERROR\n{}", msg);
                exit(1);
            }
        }
    }
}
