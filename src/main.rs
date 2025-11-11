pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod token;
pub mod vm;

use crate::compiler::Compiler;
use crate::evaluator::eval;
use crate::lexer::Lexer;
use crate::object::Environment;
use crate::parser::Parser;
use crate::vm::VM;
use std::cell::RefCell;
use std::env;
use std::io::{self, Write};
use std::rc::Rc;

fn main() {
    // Проверяем аргументы командной строки для выбора исполнителя
    let args: Vec<String> = env::args().collect();
    let use_vm = !args.contains(&"--ast".to_string());

    let env_ref = Rc::new(RefCell::new(Environment::new()));

    println!(
        "SOFIA Interpreter (Bytecode VM: {})",
        if use_vm { "ON" } else { "OFF" }
    );

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

        if use_vm {
            // Используем VM
            match run_with_vm(program) {
                Ok(result) => println!("{}", result),
                Err(e) => println!("ERROR: {}", e),
            }
        } else {
            // Используем AST-интерпретатор
            let evaluated = eval(ast::Node::Program(program), Rc::clone(&env_ref));
            println!("{}", evaluated);
        }
    }
}

/// Запустить программу на VM.
fn run_with_vm(program: ast::Program) -> Result<String, String> {
    let mut compiler = Compiler::new();
    let instructions = compiler.compile(&program)?;

    let mut vm = VM::new(instructions);
    let result = vm.run()?;

    Ok(result.to_string())
}
