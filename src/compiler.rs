use crate::ast::{Expression, Program, Statement};
use crate::bytecode::instructions::Instructions;
use crate::bytecode::opcode::Opcode;
use crate::object::Object;
use std::collections::HashMap;

/// Представляет ошибку, возникшую в процессе компиляции.
#[derive(Debug, PartialEq, Clone)]
pub enum CompilerError {
    /// Указывает на неподдерживаемую конструкцию языка.
    Unsupported(String),
    /// Ошибка при компиляции выражения.
    ExpressionError(String),
    /// Неизвестный оператор.
    UnknownOperator(String),
}

impl From<CompilerError> for String {
    fn from(err: CompilerError) -> Self {
        format!("{:?}", err)
    }
}

// === SYMBOL TABLE ===

/// Область видимости символа.
#[derive(Debug, Clone, PartialEq)]
enum SymbolScope {
    Global,
    Local,
    Free,
    Builtin,
}

/// Символ в таблице символов.
#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    scope: SymbolScope,
    index: usize,
}

/// Таблица символов для отслеживания переменных и областей видимости.
#[derive(Debug, Clone)]
struct SymbolTable {
    outer: Option<Box<SymbolTable>>,
    store: HashMap<String, Symbol>,
    num_definitions: usize,
    free_symbols: Vec<Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: Vec::new(),
        }
    }

    fn new_enclosed(outer: Box<SymbolTable>) -> Self {
        SymbolTable {
            outer: Some(outer),
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: Vec::new(),
        }
    }

    fn define(&mut self, name: String) -> Symbol {
        let scope = if self.outer.is_some() {
            SymbolScope::Local
        } else {
            SymbolScope::Global
        };
        let symbol = Symbol {
            name: name.clone(),
            scope,
            index: self.num_definitions,
        };
        self.store.insert(name, symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    fn resolve(&mut self, name: &str) -> Option<Symbol> {
        if let Some(symbol) = self.store.get(name) {
            return Some(symbol.clone());
        }

        if let Some(ref mut outer) = self.outer {
            let obj = outer.resolve(name)?;
            if obj.scope == SymbolScope::Global || obj.scope == SymbolScope::Builtin {
                return Some(obj);
            }
            // Захватываем как free-переменную
            let free = self.define_free(obj);
            Some(free)
        } else {
            None
        }
    }

    fn define_free(&mut self, original: Symbol) -> Symbol {
        self.free_symbols.push(original.clone());
        let symbol = Symbol {
            name: original.name,
            scope: SymbolScope::Free,
            index: self.free_symbols.len() - 1,
        };
        self.store.insert(symbol.name.clone(), symbol.clone());
        symbol
    }

    fn define_builtin(&mut self, name: String, index: usize) -> Symbol {
        let symbol = Symbol {
            name: name.clone(),
            scope: SymbolScope::Builtin,
            index,
        };
        self.store.insert(name, symbol.clone());
        symbol
    }
}

// === COMPILER ===

/// Компилятор, преобразующий AST в байткод.
pub struct Compiler {
    /// Сгенерированные инструкции байткода.
    instructions: Instructions,

    /// Таблица символов для отслеживания переменных.
    symbol_table: SymbolTable,

    /// Стек слоев видимости (scopes).
    scopes: Vec<Scope>,

    /// Индекс текущего слоя видимости.
    scope_index: usize,
}

/// Информация о слое видимости (scope).
#[derive(Debug, Clone)]
struct Scope {
    /// Локальные переменные в этом слое видимости.
    locals: Vec<LocalVariable>,
    /// Количество локальных переменных.
    num_locals: usize,
}

/// Информация о локальной переменной.
#[derive(Debug, Clone)]
struct LocalVariable {
    /// Имя переменной.
    name: String,
    /// Индекс в стеке локальных переменных.
    index: usize,
}

impl Compiler {
    /// Создает новый экземпляр компилятора.
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable::new();

        // Регистрируем built-in функции
        let builtins = vec![
            "len".to_string(),
            "puts".to_string(),
            "first".to_string(),
            "last".to_string(),
            "rest".to_string(),
            "push".to_string(),
        ];
        for (i, name) in builtins.iter().enumerate() {
            symbol_table.define_builtin(name.clone(), i);
        }

        Compiler {
            instructions: Instructions::new(),
            symbol_table,
            scopes: vec![Scope {
                locals: Vec::new(),
                num_locals: 0,
            }],
            scope_index: 0,
        }
    }

    /// Получить текущий слой видимости.
    fn current_scope(&mut self) -> &mut Scope {
        &mut self.scopes[self.scope_index]
    }

    /// Добавить локальную переменную в текущий слой видимости.
    fn add_local(&mut self, name: String) -> usize {
        let scope = self.current_scope();
        let index = scope.num_locals;
        scope.locals.push(LocalVariable {
            name: name.clone(),
            index,
        });
        scope.num_locals += 1;
        self.symbol_table.define(name);
        index
    }

    /// Проверить является ли переменная локальной.
    fn is_local(&self, name: &str) -> bool {
        self.scopes[self.scope_index]
            .locals
            .iter()
            .any(|l| l.name == name)
    }

    /// Компилирует заданную программу (AST) в последовательность инструкций байткода.
    pub fn compile(&mut self, program: &Program) -> Result<Instructions, CompilerError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        Ok(self.instructions.clone())
    }

    /// Компилировать один оператор.
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.compile_expression(&expr_stmt.expression)?;
                self.instructions.emit(Opcode::Pop, &[]);
                Ok(())
            }
            Statement::Let(let_stmt) => {
                self.compile_expression(&let_stmt.value)?;
                let var_name = let_stmt.name.value.clone();

                // Определяем переменную в таблице символов
                let symbol = self.symbol_table.define(var_name.clone());

                if symbol.scope == SymbolScope::Local {
                    // Локальная переменная (внутри функции)
                    self.instructions.emit(Opcode::SetLocal, &[symbol.index as u16]);
                } else {
                    // Глобальная переменная
                    let name_idx = self
                        .instructions
                        .add_constant(Object::String(var_name));
                    self.instructions
                        .emit(Opcode::SetGlobal, &[name_idx as u16]);
                }
                Ok(())
            }
            Statement::Return(ret_stmt) => {
                self.compile_expression(&ret_stmt.return_value)?;
                self.instructions.emit(Opcode::Return, &[]);
                Ok(())
            }
            Statement::Block(block_stmt) => {
                for stmt in &block_stmt.statements {
                    self.compile_statement(stmt)?;
                }
                Ok(())
            }
            _ => Err(CompilerError::Unsupported(format!(
                "Неподдерживаемый тип оператора: {:?}",
                statement
            ))),
        }
    }

    /// Компилировать выражение.
    fn compile_expression(&mut self, expression: &Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::IntegerLiteral(il) => {
                let const_idx = self.instructions.add_constant(Object::Integer(il.value));
                self.instructions
                    .emit(Opcode::Constant, &[const_idx as u16]);
                Ok(())
            }
            Expression::Boolean(bl) => {
                if bl.value {
                    self.instructions.emit(Opcode::True, &[]);
                } else {
                    self.instructions.emit(Opcode::False, &[]);
                }
                Ok(())
            }
            Expression::StringLiteral(sl) => {
                let const_idx = self
                    .instructions
                    .add_constant(Object::String(sl.value.clone()));
                self.instructions
                    .emit(Opcode::Constant, &[const_idx as u16]);
                Ok(())
            }
            Expression::Identifier(ident) => {
                if let Some(symbol) = self.symbol_table.resolve(&ident.value) {
                    match symbol.scope {
                        SymbolScope::Global => {
                            let const_idx = self
                                .instructions
                                .add_constant(Object::String(ident.value.clone()));
                            self.instructions
                                .emit(Opcode::GetGlobal, &[const_idx as u16]);
                        }
                        SymbolScope::Local => {
                            self.instructions
                                .emit(Opcode::GetLocal, &[symbol.index as u16]);
                        }
                        SymbolScope::Free => {
                            self.instructions
                                .emit(Opcode::GetFree, &[symbol.index as u16]);
                        }
                        SymbolScope::Builtin => {
                            self.instructions
                                .emit(Opcode::GetBuiltin, &[symbol.index as u16]);
                        }
                    }
                } else {
                    // Это может быть ошибка, но давайте пока загружать null
                    self.instructions.emit(Opcode::Null, &[]);
                }
                Ok(())
            }
            Expression::Prefix(pe) => {
                self.compile_expression(&pe.right)?;
                match pe.operator.as_str() {
                    "!" => self.instructions.emit(Opcode::Not, &[]),
                    "-" => self.instructions.emit(Opcode::Neg, &[]),
                    _ => return Err(CompilerError::UnknownOperator(pe.operator.clone())),
                };
                Ok(())
            }
            Expression::Infix(ie) => {
                self.compile_expression(&ie.left)?;
                self.compile_expression(&ie.right)?;
                match ie.operator.as_str() {
                    "+" => self.instructions.emit(Opcode::Add, &[]),
                    "-" => self.instructions.emit(Opcode::Sub, &[]),
                    "*" => self.instructions.emit(Opcode::Mul, &[]),
                    "/" => self.instructions.emit(Opcode::Div, &[]),
                    "%" => self.instructions.emit(Opcode::Mod, &[]),
                    "**" => self.instructions.emit(Opcode::Pow, &[]),
                    "==" => self.instructions.emit(Opcode::Equal, &[]),
                    "!=" => self.instructions.emit(Opcode::NotEqual, &[]),
                    ">" => self.instructions.emit(Opcode::GreaterThan, &[]),
                    "<" => self.instructions.emit(Opcode::LessThan, &[]),
                    ">=" => self.instructions.emit(Opcode::GreaterThanOrEqual, &[]),
                    "<=" => self.instructions.emit(Opcode::LessThanOrEqual, &[]),
                    "&&" => self.instructions.emit(Opcode::And, &[]),
                    "||" => self.instructions.emit(Opcode::Or, &[]),
                    _ => return Err(CompilerError::UnknownOperator(ie.operator.clone())),
                };
                Ok(())
            }
            Expression::If(if_expr) => {
                self.compile_expression(&if_expr.condition)?;
                let jump_if_false_pos = self.instructions.bytes.len();
                self.instructions.emit(Opcode::JumpIfFalse, &[0]); // Placeholder

                // Компилируем тело if
                for stmt in &if_expr.consequence.statements {
                    self.compile_statement(stmt)?;
                }

                // Обновляем адрес прыжка
                let target = self.instructions.bytes.len();
                let high = ((target >> 8) & 0xFF) as u8;
                let low = (target & 0xFF) as u8;
                self.instructions.bytes[jump_if_false_pos + 1] = high;
                self.instructions.bytes[jump_if_false_pos + 2] = low;

                // Если есть else, компилируем его
                if let Some(alt) = &if_expr.alternative {
                    let jump_pos = self.instructions.bytes.len();
                    self.instructions.emit(Opcode::Jump, &[0]); // Placeholder для прыжка за else

                    // Компилируем else
                    for stmt in &alt.statements {
                        self.compile_statement(stmt)?;
                    }

                    // Обновляем адрес прыжка за else
                    let target = self.instructions.bytes.len();
                    let high = ((target >> 8) & 0xFF) as u8;
                    let low = (target & 0xFF) as u8;
                    self.instructions.bytes[jump_pos + 1] = high;
                    self.instructions.bytes[jump_pos + 2] = low;
                }

                Ok(())
            }
            Expression::ArrayLiteral(arr_expr) => {
                for element in &arr_expr.elements {
                    self.compile_expression(element)?;
                }
                self.instructions
                    .emit(Opcode::Array, &[arr_expr.elements.len() as u16]);
                Ok(())
            }
            Expression::FunctionLiteral(func) => {
                // Входим в новый scope
                self.symbol_table =
                    SymbolTable::new_enclosed(Box::new(self.symbol_table.clone()));

                // Определяем параметры как локальные переменные
                for param in &func.parameters {
                    self.symbol_table.define(param.value.clone());
                }

                // Компилируем тело функции
                for stmt in &func.body.statements {
                    self.compile_statement(stmt)?;
                }

                // Если в конце тела нет ReturnValue, добавляем Return (возврат Null)
                let last_byte = self.instructions.bytes.last().copied();
                if last_byte != Some(Opcode::ReturnValue as u8)
                    && last_byte != Some(Opcode::Return as u8)
                {
                    self.instructions.emit(Opcode::Return, &[]);
                }

                // Собираем данные о функции
                let num_locals = self.symbol_table.num_definitions;
                let free_symbols = self.symbol_table.free_symbols.clone();

                // Выходим из scope
                if let Some(outer) = self.symbol_table.outer.take() {
                    self.symbol_table = *outer;
                }

                // Создаём CompiledFunction и добавляем в пул констант
                let compiled_fn = Object::CompiledFunction(
                    crate::object::CompiledFunction {
                        instructions_offset: 0, // Будет исправлено позже
                        num_locals,
                        num_params: func.parameters.len(),
                    },
                );
                let const_idx = self.instructions.add_constant(compiled_fn);

                // Загружаем free-переменные на стек
                for free_sym in &free_symbols {
                    match free_sym.scope {
                        SymbolScope::Local => {
                            self.instructions
                                .emit(Opcode::GetLocal, &[free_sym.index as u16]);
                        }
                        SymbolScope::Free => {
                            self.instructions
                                .emit(Opcode::GetFree, &[free_sym.index as u16]);
                        }
                        _ => {
                            return Err(CompilerError::Unsupported(
                                "invalid free variable scope".to_string(),
                            ))
                        }
                    }
                }

                // Эмитируем опкод функции
                if free_symbols.is_empty() {
                    self.instructions.emit(Opcode::Constant, &[const_idx as u16]);
                } else {
                    self.instructions.emit(Opcode::Closure, &[const_idx as u16, free_symbols.len() as u16]);
                }

                Ok(())
            }
            Expression::Call(call) => {
                // Компилируем выражение-функцию (push на стек)
                self.compile_expression(&call.function)?;

                // Компилируем аргументы (push на стек)
                for arg in &call.arguments {
                    self.compile_expression(arg)?;
                }

                // Эмитируем Call опкод с количеством аргументов
                self.instructions
                    .emit(Opcode::Call, &[call.arguments.len() as u16]);

                Ok(())
            }
            _ => Err(CompilerError::Unsupported(format!(
                "Неподдерживаемое выражение: {:?}",
                expression
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BooleanLiteral, ExpressionStatement, Identifier, IntegerLiteral};
    use crate::token::{Token, TokenType};

    fn make_token() -> Token {
        Token::new(TokenType::Int, "".to_string())
    }

    fn make_program(stmts: Vec<Statement>) -> Program {
        Program { statements: stmts }
    }

    fn make_int_literal(value: i64) -> Expression {
        Expression::IntegerLiteral(IntegerLiteral {
            token: make_token(),
            value,
        })
    }

    fn make_bool_literal(value: bool) -> Expression {
        Expression::Boolean(BooleanLiteral {
            token: make_token(),
            value,
        })
    }

    fn make_identifier(value: String) -> Expression {
        Expression::Identifier(Identifier {
            token: make_token(),
            value,
        })
    }

    #[test]
    fn test_compiler_integer_literal() {
        let mut compiler = Compiler::new();
        let program = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: make_int_literal(42),
        })]);

        let result = compiler.compile(&program);
        assert!(result.is_ok());

        let instructions = result.unwrap();
        // Ожидаем: Constant (1 byte) + 2-byte operand (index 0) + Pop (1 byte)
        assert_eq!(instructions.bytes[0], Opcode::Constant as u8);
        assert_eq!(instructions.bytes[3], Opcode::Pop as u8);
        assert_eq!(instructions.constants.len(), 1);
        assert_eq!(instructions.constants[0], Object::Integer(42));
    }

    #[test]
    fn test_compiler_boolean_literal() {
        let mut compiler = Compiler::new();

        let program_true = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: make_bool_literal(true),
        })]);

        let result = compiler.compile(&program_true);
        assert!(result.is_ok());

        let instructions = result.unwrap();
        assert_eq!(instructions.bytes[0], Opcode::True as u8);
        assert_eq!(instructions.bytes[1], Opcode::Pop as u8);

        // Тест для false
        let mut compiler2 = Compiler::new();
        let program_false = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: make_bool_literal(false),
        })]);

        let result2 = compiler2.compile(&program_false);
        assert!(result2.is_ok());

        let instructions2 = result2.unwrap();
        assert_eq!(instructions2.bytes[0], Opcode::False as u8);
        assert_eq!(instructions2.bytes[1], Opcode::Pop as u8);
    }

    #[test]
    fn test_compiler_prefix_expression() {
        let mut compiler = Compiler::new();

        // Компилируем: -42
        let program = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: Expression::Prefix(crate::ast::PrefixExpression {
                token: make_token(),
                operator: "-".to_string(),
                right: Box::new(make_int_literal(42)),
            }),
        })]);

        let result = compiler.compile(&program);
        assert!(result.is_ok());

        let instructions = result.unwrap();
        // Ожидаем: Constant(42), Neg, Pop
        assert_eq!(instructions.bytes[0], Opcode::Constant as u8);
        assert_eq!(instructions.bytes[3], Opcode::Neg as u8);
    }

    #[test]
    fn test_compiler_infix_expression() {
        let mut compiler = Compiler::new();

        // Компилируем: 10 + 20
        let program = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: Expression::Infix(crate::ast::InfixExpression {
                token: make_token(),
                left: Box::new(make_int_literal(10)),
                operator: "+".to_string(),
                right: Box::new(make_int_literal(20)),
            }),
        })]);

        let result = compiler.compile(&program);
        assert!(result.is_ok());

        let instructions = result.unwrap();
        // Ожидаем: Constant(10), Constant(20), Add, Pop
        assert_eq!(instructions.bytes[0], Opcode::Constant as u8);
        assert_eq!(instructions.bytes[3], Opcode::Constant as u8);
        // Добавляем 3 (opcode) + 3 (operand) = 6 байт для второй константы
        assert_eq!(instructions.bytes[6], Opcode::Add as u8);
        assert_eq!(instructions.constants.len(), 2);
        assert_eq!(instructions.constants[0], Object::Integer(10));
        assert_eq!(instructions.constants[1], Object::Integer(20));
    }

    #[test]
    fn test_compiler_error_unknown_operator() {
        let mut compiler = Compiler::new();

        let program = make_program(vec![Statement::Expression(ExpressionStatement {
            token: make_token(),
            expression: Expression::Infix(crate::ast::InfixExpression {
                token: make_token(),
                left: Box::new(make_int_literal(10)),
                operator: "@@".to_string(), // Неподдерживаемый оператор
                right: Box::new(make_int_literal(20)),
            }),
        })]);

        let result = compiler.compile(&program);
        assert!(result.is_err());

        if let Err(CompilerError::UnknownOperator(op)) = result {
            assert_eq!(op, "@@");
        } else {
            panic!("Expected UnknownOperator error");
        }
    }

    #[test]
    fn test_compiler_multiple_statements() {
        let mut compiler = Compiler::new();

        let program = make_program(vec![
            Statement::Expression(ExpressionStatement {
                token: make_token(),
                expression: make_int_literal(1),
            }),
            Statement::Expression(ExpressionStatement {
                token: make_token(),
                expression: make_int_literal(2),
            }),
            Statement::Expression(ExpressionStatement {
                token: make_token(),
                expression: make_int_literal(3),
            }),
        ]);

        let result = compiler.compile(&program);
        assert!(result.is_ok());

        let instructions = result.unwrap();
        // Ожидаем 3 константы + инструкции для каждой
        assert_eq!(instructions.constants.len(), 3);
        assert_eq!(instructions.constants[0], Object::Integer(1));
        assert_eq!(instructions.constants[1], Object::Integer(2));
        assert_eq!(instructions.constants[2], Object::Integer(3));
    }
}
