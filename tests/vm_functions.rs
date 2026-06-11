use project_sofia_lib::compiler::Compiler;
use project_sofia_lib::lexer::Lexer;
use project_sofia_lib::object::Object;
use project_sofia_lib::parser::Parser;
use project_sofia_lib::vm::VM;

fn eval_with_vm(input: &str) -> Object {
    let lexer = Lexer::new(input.to_string());
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();

    let mut compiler = Compiler::new();
    let instructions = compiler.compile(&program).unwrap();

    let mut vm = VM::new(instructions);
    vm.run().unwrap()
}

#[test]
fn test_simple_function() {
    let result = eval_with_vm("let identity = fn(x) { x; }; identity(42);");
    assert_eq!(result, Object::Integer(42));
}

#[test]
fn test_function_with_return() {
    let result = eval_with_vm("let add = fn(x, y) { return x + y; }; add(2, 3);");
    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_function_with_locals() {
    let result = eval_with_vm("let compute = fn(a, b) { let sum = a + b; let product = a * b; return sum + product; }; compute(2, 3);");
    assert_eq!(result, Object::Integer(11));
}
