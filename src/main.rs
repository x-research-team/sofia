pub mod ast;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod token;

use crate::evaluator::eval;
use crate::lexer::Lexer;
use crate::object::Environment;
use crate::parser::Parser;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

fn main() {
    let env = Rc::new(RefCell::new(Environment::new()));

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = match parser.parse_program() {
            Ok(program) => program,
            Err(errors) => {
                for error in errors {
                    println!("\t{:?}", error);
                }
                continue;
            }
        };

        let evaluated = eval(ast::Node::Program(program), Rc::clone(&env));
        println!("{}", evaluated);
    }
}
