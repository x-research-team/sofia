# VM Functions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement full function support in the bytecode VM, including simple functions, arguments, local variables, closures, recursive closures, built-in functions, and first-class functions, with parity verification against the AST interpreter.

**Architecture:** Monkey-pattern approach (Thorsten Ball's "Writing a Compiler in Go"). Single stack with basePointer frame separation. Closures via value copying into `free[]` array. Symbol table for scope resolution (Global/Local/Free/Builtin).

**Tech Stack:** Rust, existing bytecode VM infrastructure, TDD approach

---

## File Structure

- **Modify:** `src/object.rs:8-23` - Add CompiledFunction, Closure, BuiltinFunction variants to Object enum
- **Modify:** `src/bytecode/opcode.rs:5-103` - Add new opcodes (Call, ReturnValue, GetLocal, SetLocal, GetFree, SetFree, GetCurrentClosure, MakeClosure, GetBuiltin, Closure)
- **Modify:** `src/bytecode/instructions.rs:6-11` - Update operand widths for new opcodes
- **Modify:** `src/compiler.rs:1-487` - Add SymbolTable, function compilation, call compilation, local variables
- **Modify:** `src/vm/mod.rs:1-508` - Implement Call/Return execution, GetLocal/SetLocal, closures, built-ins
- **Test:** Tests in respective files (compiler.rs, vm/mod.rs)

---

### Task 1: Add Data Structures to Object Enum

**Files:**
- Modify: `src/object.rs:8-23`

**Context:** The Object enum needs new variants to represent compiled functions, closures, and built-in functions. These will be used throughout the compiler and VM.

- [ ] **Step 1: Define CompiledFunction struct**

Add before the Object enum:

```rust
/// Represents a compiled function's bytecode metadata.
#[derive(Debug, PartialEq, Clone)]
pub struct CompiledFunction {
    pub instructions_offset: usize,
    pub num_locals: usize,
    pub num_params: usize,
}
```

- [ ] **Step 2: Add new Object variants**

Add to the Object enum (after existing variants):

```rust
pub enum Object {
    // ... existing variants ...
    CompiledFunction(CompiledFunction),
    Closure(Box<CompiledFunction>, Vec<Object>),  // function + free variables
    BuiltinFunction {
        name: String,
        num_params: i32,  // -1 = variadic
        handler: fn(Vec<Object>) -> Object,
    },
}
```

- [ ] **Step 3: Update type_str() method**

Add to the match in `type_str()`:

```rust
Object::CompiledFunction(_) => "COMPILED_FUNCTION",
Object::Closure(_, _) => "CLOSURE",
Object::BuiltinFunction { .. } => "BUILTIN_FUNCTION",
```

- [ ] **Step 4: Update Display trait**

Add to the match in Display impl:

```rust
Object::CompiledFunction(cf) => write!(
    f,
    "compiled fn(offset={}, locals={}, params={})",
    cf.instructions_offset, cf.num_locals, cf.num_params
),
Object::Closure(cf, free) => write!(
    f,
    "closure(offset={}, free={})",
    cf.instructions_offset,
    free.len()
),
Object::BuiltinFunction { name, .. } => write!(f, "builtin fn {}", name),
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build`
Expected: BUILD SUCCESS

- [ ] **Step 6: Commit**

```bash
git add src/object.rs
git commit -m "feat: add CompiledFunction, Closure, BuiltinFunction to Object"
```

---

### Task 2: Add New Opcodes

**Files:**
- Modify: `src/bytecode/opcode.rs:5-103`
- Modify: `src/bytecode/opcode.rs:156-201` (operand_widths)

**Context:** New opcodes needed for function calls, returns, local variable access, closures, and built-ins.

- [ ] **Step 1: Add new opcodes to enum**

Add to the Opcode enum (in appropriate sections):

```rust
pub enum Opcode {
    // ... existing opcodes ...
    
    // Function calls and returns
    ReturnValue = 42,
    GetLocal = 43,
    SetLocal = 44,
    GetFree = 45,
    SetFree = 46,
    GetCurrentClosure = 47,
    
    // Closures and built-ins
    Closure = 48,
    GetBuiltin = 49,
}
```

- [ ] **Step 2: Update operand_widths()**

Add to the match in `operand_widths()`:

```rust
// Function opcodes
Opcode::GetLocal
| Opcode::SetLocal
| Opcode::GetFree
| Opcode::SetFree
| Opcode::GetBuiltin => &[1],  // 1-byte operand

Opcode::Closure => &[2, 1],  // 2-byte const index + 1-byte num_free

// No operands
Opcode::ReturnValue
| Opcode::GetCurrentClosure => &[],
```

- [ ] **Step 3: Update mnemonic()**

Add to the match in `mnemonic()`:

```rust
Opcode::ReturnValue => "RETURN_VALUE",
Opcode::GetLocal => "GET_LOCAL",
Opcode::SetLocal => "SET_LOCAL",
Opcode::GetFree => "GET_FREE",
Opcode::SetFree => "SET_FREE",
Opcode::GetCurrentClosure => "GET_CURRENT_CLOSURE",
Opcode::Closure => "CLOSURE",
Opcode::GetBuiltin => "GET_BUILTIN",
```

- [ ] **Step 4: Update from_byte()**

Add to the match in `from_byte()`:

```rust
42 => Some(Opcode::ReturnValue),
43 => Some(Opcode::GetLocal),
44 => Some(Opcode::SetLocal),
45 => Some(Opcode::GetFree),
46 => Some(Opcode::SetFree),
47 => Some(Opcode::GetCurrentClosure),
48 => Some(Opcode::Closure),
49 => Some(Opcode::GetBuiltin),
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build`
Expected: BUILD SUCCESS

- [ ] **Step 6: Commit**

```bash
git add src/bytecode/opcode.rs
git commit -m "feat: add function-related opcodes (Call, Return, Locals, Closures)"
```

---

### Task 3: Update CallFrame Structure

**Files:**
- Modify: `src/vm/mod.rs:47-56`

**Context:** CallFrame needs to track base_pointer for stack frame separation and num_locals for local variable allocation.

- [ ] **Step 1: Update CallFrame fields**

Replace the CallFrame struct:

```rust
#[derive(Debug, PartialEq, Clone)]
pub struct CallFrame {
    pub return_addr: usize,
    pub base_pointer: usize,
    pub num_locals: usize,
}
```

- [ ] **Step 2: Update VM::new() initialization**

The frames field is already `Vec::new()`, no changes needed.

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: BUILD SUCCESS (warnings about unused fields are OK)

- [ ] **Step 4: Commit**

```bash
git add src/vm/mod.rs
git commit -m "refactor: update CallFrame with base_pointer and num_locals"
```

---

### Task 4: Implement Symbol Table for Compiler

**Files:**
- Modify: `src/compiler.rs:1-80` (add new structures before Compiler struct)

**Context:** The compiler needs a symbol table to track variable scopes (Global, Local, Free, Builtin) and resolve identifiers correctly.

- [ ] **Step 1: Define SymbolScope enum**

Add at the top of compiler.rs:

```rust
#[derive(Debug, Clone, PartialEq)]
enum SymbolScope {
    Global,
    Local,
    Free,
    Builtin,
}
```

- [ ] **Step 2: Define Symbol struct**

```rust
#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    scope: SymbolScope,
    index: usize,
}
```

- [ ] **Step 3: Define SymbolTable struct**

```rust
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
            
            // Capture as free variable
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
```

- [ ] **Step 4: Update Compiler struct**

Add symbol_table field:

```rust
pub struct Compiler {
    instructions: Instructions,
    symbol_table: SymbolTable,
    scopes: Vec<Scope>,
    scope_index: usize,
}
```

- [ ] **Step 5: Update Compiler::new()**

Initialize symbol_table and register built-ins:

```rust
pub fn new() -> Self {
    let mut symbol_table = SymbolTable::new();
    
    // Register built-in functions
    let builtins = vec!["len", "puts", "first", "last", "rest", "push"];
    for (i, name) in builtins.iter().enumerate() {
        symbol_table.define_builtin(name.to_string(), i);
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
```

- [ ] **Step 6: Verify compilation**

Run: `cargo build`
Expected: BUILD SUCCESS (warnings about unused methods are OK)

- [ ] **Step 7: Commit**

```bash
git add src/compiler.rs
git commit -m "feat: add SymbolTable with Global/Local/Free/Builtin scopes"
```

---

### Task 5: Compile Simple Function Literals

**Files:**
- Modify: `src/compiler.rs:168-294` (compile_expression method)

**Context:** Compile FunctionLiteral expressions into bytecode. The function body is compiled inline, and a CompiledFunction object is created and pushed onto the stack.

- [ ] **Step 1: Write failing test**

Add to compiler.rs tests:

```rust
#[test]
fn test_compiler_function_literal() {
    let mut compiler = Compiler::new();
    let program = make_program(vec![Statement::Expression(ExpressionStatement {
        token: make_token(),
        expression: Expression::FunctionLiteral(FunctionLiteral {
            token: Token::new(TokenType::Function, "fn".to_string()),
            parameters: vec![
                Identifier {
                    token: Token::new(TokenType::Ident, "x".to_string()),
                    value: "x".to_string(),
                },
                Identifier {
                    token: Token::new(TokenType::Ident, "y".to_string()),
                    value: "y".to_string(),
                },
            ],
            body: BlockStatement {
                token: Token::new(TokenType::LBrace, "{".to_string()),
                statements: vec![Statement::Return(ReturnStatement {
                    token: Token::new(TokenType::Return, "return".to_string()),
                    return_value: Expression::Infix(InfixExpression {
                        token: Token::new(TokenType::Plus, "+".to_string()),
                        left: Box::new(Expression::Identifier(Identifier {
                            token: Token::new(TokenType::Ident, "x".to_string()),
                            value: "x".to_string(),
                        })),
                        operator: "+".to_string(),
                        right: Box::new(Expression::Identifier(Identifier {
                            token: Token::new(TokenType::Ident, "y".to_string()),
                            value: "y".to_string(),
                        })),
                    }),
                })],
            },
        }),
    })]);

    let result = compiler.compile(&program);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    
    // Should have a Closure or Constant opcode for the function
    assert_eq!(instructions.bytes[0], Opcode::Closure as u8);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_compiler_function_literal -- --nocapture`
Expected: FAIL (unimplemented match arm)

- [ ] **Step 3: Implement FunctionLiteral compilation**

Add to compile_expression match:

```rust
Expression::FunctionLiteral(func) => {
    // Enter new scope
    self.symbol_table = SymbolTable::new_enclosed(Box::new(self.symbol_table.clone()));
    
    // Define parameters as locals
    for param in &func.parameters {
        self.symbol_table.define(param.value.clone());
    }
    
    // Compile body
    for stmt in &func.body.statements {
        self.compile_statement(stmt)?;
    }
    
    // Ensure function returns Null if no explicit return
    let last_opcode = if self.instructions.bytes.is_empty() {
        Opcode::Null
    } else {
        Opcode::from_byte(self.instructions.bytes[self.instructions.bytes.len() - 1]).unwrap_or(Opcode::Null)
    };
    
    if last_opcode != Opcode::ReturnValue && last_opcode != Opcode::Return {
        self.instructions.emit(Opcode::Return, &[]);
    }
    
    // Exit scope and collect free variables
    let num_locals = self.symbol_table.num_definitions;
    let free_symbols = self.symbol_table.free_symbols.clone();
    
    if let Some(outer) = self.symbol_table.outer.take() {
        self.symbol_table = *outer;
    }
    
    // Create CompiledFunction
    let func_offset = self.instructions.bytes.len();
    let compiled_fn = Object::CompiledFunction(CompiledFunction {
        instructions_offset: 0,  // Will be patched
        num_locals,
        num_params: func.parameters.len(),
    });
    
    // Add to constants pool
    let const_idx = self.instructions.add_constant(compiled_fn);
    
    // Load free variables onto stack
    for free_sym in &free_symbols {
        match free_sym.scope {
            SymbolScope::Local => self.instructions.emit(Opcode::GetLocal, &[free_sym.index as u16]),
            SymbolScope::Free => self.instructions.emit(Opcode::GetFree, &[free_sym.index as u16]),
            _ => return Err(CompilerError::Unsupported("invalid free variable scope".to_string())),
        }
    }
    
    // Emit Closure opcode
    if free_symbols.is_empty() {
        self.instructions.emit(Opcode::Constant, &[const_idx as u16]);
    } else {
        self.instructions.emit(Opcode::Closure, &[const_idx as u16, free_symbols.len() as u16]);
    }
    
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_compiler_function_literal -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/compiler.rs
git commit -m "feat: compile FunctionLiteral with scope and free variables"
```

---

### Task 6: Compile Call Expressions

**Files:**
- Modify: `src/compiler.rs:168-294` (compile_expression method)

**Context:** Compile CallExpression to push function and arguments onto stack, then emit Call opcode.

- [ ] **Step 1: Write failing test**

Add to compiler.rs tests:

```rust
#[test]
fn test_compiler_call_expression() {
    let mut compiler = Compiler::new();
    
    // Compile: let add = fn(x, y) { return x + y; }; add(2, 3);
    let program = make_program(vec![
        // Let statement for add
        Statement::Let(LetStatement {
            token: Token::new(TokenType::Let, "let".to_string()),
            name: Identifier {
                token: Token::new(TokenType::Ident, "add".to_string()),
                value: "add".to_string(),
            },
            value: Expression::FunctionLiteral(FunctionLiteral {
                token: Token::new(TokenType::Function, "fn".to_string()),
                parameters: vec![
                    Identifier { token: Token::new(TokenType::Ident, "x".to_string()), value: "x".to_string() },
                    Identifier { token: Token::new(TokenType::Ident, "y".to_string()), value: "y".to_string() },
                ],
                body: BlockStatement {
                    token: Token::new(TokenType::LBrace, "{".to_string()),
                    statements: vec![Statement::Return(ReturnStatement {
                        token: Token::new(TokenType::Return, "return".to_string()),
                        return_value: Expression::Infix(InfixExpression {
                            token: Token::new(TokenType::Plus, "+".to_string()),
                            left: Box::new(Expression::Identifier(Identifier { token: Token::new(TokenType::Ident, "x".to_string()), value: "x".to_string() })),
                            operator: "+".to_string(),
                            right: Box::new(Expression::Identifier(Identifier { token: Token::new(TokenType::Ident, "y".to_string()), value: "y".to_string() })),
                        }),
                    })],
                },
            }),
        }),
        // Call: add(2, 3)
        Statement::Expression(ExpressionStatement {
            token: Token::new(TokenType::Ident, "add".to_string()),
            expression: Expression::Call(CallExpression {
                token: Token::new(TokenType::LParen, "(".to_string()),
                function: Box::new(Expression::Identifier(Identifier {
                    token: Token::new(TokenType::Ident, "add".to_string()),
                    value: "add".to_string(),
                })),
                arguments: vec![
                    Expression::IntegerLiteral(IntegerLiteral { token: make_token(), value: 2 }),
                    Expression::IntegerLiteral(IntegerLiteral { token: make_token(), value: 3 }),
                ],
            }),
        }),
    ]);

    let result = compiler.compile(&program);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    
    // Should contain Call opcode with num_args=2
    let call_pos = instructions.bytes.iter().position(|&b| b == Opcode::Call as u8);
    assert!(call_pos.is_some());
    let call_pos = call_pos.unwrap();
    assert_eq!(instructions.bytes[call_pos + 1], 2);  // num_args
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_compiler_call_expression -- --nocapture`
Expected: FAIL (unimplemented match arm)

- [ ] **Step 3: Implement CallExpression compilation**

Add to compile_expression match:

```rust
Expression::Call(call) => {
    // Compile function expression (push onto stack)
    self.compile_expression(&call.function)?;
    
    // Compile arguments (push onto stack)
    for arg in &call.arguments {
        self.compile_expression(arg)?;
    }
    
    // Emit Call opcode with number of arguments
    self.instructions.emit(Opcode::Call, &[call.arguments.len() as u16]);
    
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_compiler_call_expression -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/compiler.rs
git commit -m "feat: compile CallExpression with arguments"
```

---

### Task 7: Compile Local Variables (Let inside functions)

**Files:**
- Modify: `src/compiler.rs:124-165` (compile_statement method)

**Context:** Let statements inside functions should create local variables, not globals.

- [ ] **Step 1: Write failing test**

Add to compiler.rs tests:

```rust
#[test]
fn test_compiler_local_variables() {
    let mut compiler = Compiler::new();
    
    // Compile: fn() { let x = 10; return x; }
    let program = make_program(vec![Statement::Expression(ExpressionStatement {
        token: make_token(),
        expression: Expression::FunctionLiteral(FunctionLiteral {
            token: Token::new(TokenType::Function, "fn".to_string()),
            parameters: vec![],
            body: BlockStatement {
                token: Token::new(TokenType::LBrace, "{".to_string()),
                statements: vec![
                    Statement::Let(LetStatement {
                        token: Token::new(TokenType::Let, "let".to_string()),
                        name: Identifier { token: Token::new(TokenType::Ident, "x".to_string()), value: "x".to_string() },
                        value: Expression::IntegerLiteral(IntegerLiteral { token: make_token(), value: 10 }),
                    }),
                    Statement::Return(ReturnStatement {
                        token: Token::new(TokenType::Return, "return".to_string()),
                        return_value: Expression::Identifier(Identifier { token: Token::new(TokenType::Ident, "x".to_string()), value: "x".to_string() }),
                    }),
                ],
            },
        }),
    })]);

    let result = compiler.compile(&program);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    
    // Should contain SetLocal and GetLocal opcodes
    let set_local_pos = instructions.bytes.iter().position(|&b| b == Opcode::SetLocal as u8);
    let get_local_pos = instructions.bytes.iter().position(|&b| b == Opcode::GetLocal as u8);
    assert!(set_local_pos.is_some());
    assert!(get_local_pos.is_some());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_compiler_local_variables -- --nocapture`
Expected: FAIL (currently compiles as SetGlobal/GetGlobal)

- [ ] **Step 3: Update Let statement compilation**

Replace the Let arm in compile_statement:

```rust
Statement::Let(let_stmt) => {
    self.compile_expression(&let_stmt.value)?;
    
    // Check if we're in a function scope
    if self.symbol_table.outer.is_some() {
        // Local variable
        let symbol = self.symbol_table.define(let_stmt.name.value.clone());
        self.instructions.emit(Opcode::SetLocal, &[symbol.index as u16]);
    } else {
        // Global variable
        let var_name = let_stmt.name.value.clone();
        let name_idx = self.instructions.add_constant(Object::String(var_name.clone()));
        self.instructions.emit(Opcode::SetGlobal, &[name_idx as u16]);
        self.symbol_table.define(var_name);
    }
    
    Ok(())
}
```

- [ ] **Step 4: Update Identifier compilation**

Replace the Identifier arm in compile_expression:

```rust
Expression::Identifier(ident) => {
    if let Some(symbol) = self.symbol_table.resolve(&ident.value) {
        match symbol.scope {
            SymbolScope::Global => {
                let const_idx = self.instructions.add_constant(Object::String(ident.value.clone()));
                self.instructions.emit(Opcode::GetGlobal, &[const_idx as u16]);
            }
            SymbolScope::Local => {
                self.instructions.emit(Opcode::GetLocal, &[symbol.index as u16]);
            }
            SymbolScope::Free => {
                self.instructions.emit(Opcode::GetFree, &[symbol.index as u16]);
            }
            SymbolScope::Builtin => {
                self.instructions.emit(Opcode::GetBuiltin, &[symbol.index as u16]);
            }
        }
    } else {
        self.instructions.emit(Opcode::Null, &[]);
    }
    Ok(())
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_compiler_local_variables -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/compiler.rs
git commit -m "feat: compile local variables with SetLocal/GetLocal"
```

---

### Task 8: Implement VM Call/Return Execution

**Files:**
- Modify: `src/vm/mod.rs:86-396` (run method)

**Context:** The VM needs to execute Call and Return opcodes, managing the call stack and frame transitions.

- [ ] **Step 1: Write failing test**

Add to vm/mod.rs tests:

```rust
#[test]
fn test_vm_simple_function_call() {
    // Manually construct bytecode for:
    // let add = fn(x, y) { return x + y; };
    // add(2, 3);
    
    let mut instr = Instructions::new();
    
    // Function body: x + y
    let func_offset = instr.bytes.len();
    instr.bytes.push(Opcode::GetLocal as u8);
    instr.bytes.push(0);  // x
    instr.bytes.push(Opcode::GetLocal as u8);
    instr.bytes.push(1);  // y
    instr.bytes.push(Opcode::Add as u8);
    instr.bytes.push(Opcode::ReturnValue as u8);
    
    // Create CompiledFunction
    let compiled_fn = Object::CompiledFunction(CompiledFunction {
        instructions_offset: func_offset,
        num_locals: 2,
        num_params: 2,
    });
    let const_idx = instr.constants.len();
    instr.constants.push(compiled_fn);
    
    // Main code
    let main_offset = instr.bytes.len();
    instr.bytes.push(Opcode::Constant as u8);
    instr.bytes.extend_from_slice(&(const_idx as u16).to_be_bytes());
    
    // Store in global "add"
    let name_idx = instr.constants.len();
    instr.constants.push(Object::String("add".to_string()));
    instr.bytes.push(Opcode::SetGlobal as u8);
    instr.bytes.extend_from_slice(&(name_idx as u16).to_be_bytes());
    
    // Call add(2, 3)
    instr.bytes.push(Opcode::GetGlobal as u8);
    instr.bytes.extend_from_slice(&(name_idx as u16).to_be_bytes());
    
    instr.bytes.push(Opcode::Constant as u8);
    let two_idx = instr.constants.len();
    instr.constants.push(Object::Integer(2));
    instr.bytes.extend_from_slice(&(two_idx as u16).to_be_bytes());
    
    instr.bytes.push(Opcode::Constant as u8);
    let three_idx = instr.constants.len();
    instr.constants.push(Object::Integer(3));
    instr.bytes.extend_from_slice(&(three_idx as u16).to_be_bytes());
    
    instr.bytes.push(Opcode::Call as u8);
    instr.bytes.push(2);  // num_args
    
    let mut vm = VM::new(instr);
    let result = vm.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Object::Integer(5));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_vm_simple_function_call -- --nocapture`
Expected: FAIL (Call opcode not implemented)

- [ ] **Step 3: Implement Call opcode**

Add to the run() match (replace the existing Call arm):

```rust
Opcode::Call => {
    let num_args = self.read_u8() as usize;
    
    // Function is on stack below arguments
    let fn_idx = self.sp - 1 - num_args;
    let func_obj = self.stack[fn_idx].clone();
    
    match func_obj {
        Object::CompiledFunction(cf) => {
            if num_args != cf.num_params {
                return Err(format!(
                    "wrong number of arguments: expected {}, got {}",
                    cf.num_params, num_args
                ));
            }
            
            // Check call depth
            if let Some(max) = self.max_call_depth {
                if self.frames.len() >= max {
                    return Err("call stack overflow".to_string());
                }
            }
            
            // Save current frame
            self.frames.push(CallFrame {
                return_addr: self.ip,
                base_pointer: fn_idx + 1,
                num_locals: cf.num_locals,
            });
            
            // Initialize local variables (params already on stack)
            for _ in num_args..cf.num_locals {
                self.push(Object::Null)?;
            }
            
            // Jump to function body
            self.ip = cf.instructions_offset;
        }
        _ => return Err(format!("not a function: {}", func_obj.type_str())),
    }
}
```

- [ ] **Step 4: Implement ReturnValue opcode**

Add to the run() match:

```rust
Opcode::ReturnValue => {
    let result = self.pop()?;
    
    if let Some(frame) = self.frames.pop() {
        // Restore stack pointer (remove func_obj + args + locals)
        self.sp = frame.base_pointer - 1;
        
        // Restore instruction pointer
        self.ip = frame.return_addr;
        
        // Push result onto caller's stack
        self.push(result)?;
    } else {
        // Return from top-level
        return Ok(result);
    }
}
```

- [ ] **Step 5: Implement Return opcode**

Add to the run() match:

```rust
Opcode::Return => {
    if let Some(frame) = self.frames.pop() {
        self.sp = frame.base_pointer - 1;
        self.ip = frame.return_addr;
        self.push(Object::Null)?;
    } else {
        return Ok(Object::Null);
    }
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test test_vm_simple_function_call -- --nocapture`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/vm/mod.rs
git commit -m "feat: implement Call/Return execution in VM"
```

---

### Task 9: Implement GetLocal/SetLocal in VM

**Files:**
- Modify: `src/vm/mod.rs:317-328` (GetLocal/SetLocal opcodes)

**Context:** VM needs to access local variables using base_pointer from current frame.

- [ ] **Step 1: Write failing test**

Add to vm/mod.rs tests:

```rust
#[test]
fn test_vm_get_set_local() {
    // Test: let x = 10; return x;
    let mut instr = Instructions::new();
    
    // Constant 10
    instr.constants.push(Object::Integer(10));
    instr.bytes.push(Opcode::Constant as u8);
    instr.bytes.extend_from_slice(&0u16.to_be_bytes());
    
    // SetLocal 0
    instr.bytes.push(Opcode::SetLocal as u8);
    instr.bytes.push(0);
    
    // GetLocal 0
    instr.bytes.push(Opcode::GetLocal as u8);
    instr.bytes.push(0);
    
    let mut vm = VM::new(instr);
    let result = vm.run();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Object::Integer(10));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_vm_get_set_local -- --nocapture`
Expected: FAIL (GetLocal/SetLocal not using base_pointer)

- [ ] **Step 3: Implement GetLocal**

Replace the GetLocal arm:

```rust
Opcode::GetLocal => {
    let idx = self.read_u8() as usize;
    let bp = self.frames.last()
        .map(|f| f.base_pointer)
        .unwrap_or(0);
    let value = self.stack[bp + idx].clone();
    self.push(value)?;
}
```

- [ ] **Step 4: Implement SetLocal**

Replace the SetLocal arm:

```rust
Opcode::SetLocal => {
    let idx = self.read_u8() as usize;
    let bp = self.frames.last()
        .map(|f| f.base_pointer)
        .unwrap_or(0);
    let value = self.pop()?;
    self.stack[bp + idx] = value;
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_vm_get_set_local -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/vm/mod.rs
git commit -m "feat: implement GetLocal/SetLocal with base_pointer"
```

---

### Task 10: End-to-End Function Test (Compiler + VM)

**Files:**
- Create: `tests/vm_functions.rs` (new integration test file)

**Context:** Verify that compiled functions execute correctly in the VM.

- [ ] **Step 1: Create integration test file**

Create `tests/vm_functions.rs`:

```rust
use project_sofia_lib::compiler::Compiler;
use project_sofia_lib::lexer::Lexer;
use project_sofia_lib::parser::Parser;
use project_sofia_lib::vm::VM;
use project_sofia_lib::object::Object;

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
    let input = r#"
        let add = fn(x, y) { return x + y; };
        add(2, 3);
    "#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_function_with_locals() {
    let input = r#"
        let compute = fn(a, b) {
            let sum = a + b;
            let product = a * b;
            return sum + product;
        };
        compute(2, 3);
    "#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(11));  // (2+3) + (2*3) = 5 + 6 = 11
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test vm_functions -- --nocapture`
Expected: FAIL (compilation or execution errors)

- [ ] **Step 3: Fix any compilation/runtime issues**

Debug and fix issues found in the previous step.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test vm_functions -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/vm_functions.rs
git commit -m "test: add integration tests for VM functions"
```

---

### Task 11: Implement Closures (Free Variables)

**Files:**
- Modify: `src/vm/mod.rs:373-382` (Closure opcode)
- Modify: `src/vm/mod.rs:86-396` (Call opcode to handle Closure)

**Context:** Closures capture free variables from outer scopes. The Closure opcode creates a Closure object from a CompiledFunction and free variable values.

- [ ] **Step 1: Write failing test**

Add to tests/vm_functions.rs:

```rust
#[test]
fn test_closure() {
    let input = r#"
        let newAdder = fn(x) {
            return fn(y) { x + y; };
        };
        let addTwo = newAdder(2);
        addTwo(3);
    "#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(5));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_closure -- --nocapture`
Expected: FAIL (Closure opcode not implemented)

- [ ] **Step 3: Implement Closure opcode in VM**

Add to the run() match:

```rust
Opcode::Closure => {
    let const_idx = self.read_u16() as usize;
    let num_free = self.read_u8() as usize;
    
    let func_obj = self.instructions.get_constant(const_idx)
        .ok_or("constant not found")?
        .clone();
    
    if let Object::CompiledFunction(cf) = func_obj {
        // Pop free variables (in reverse order)
        let mut free = Vec::new();
        for _ in 0..num_free {
            free.push(self.pop()?);
        }
        free.reverse();
        
        let closure = Object::Closure(Box::new(cf), free);
        self.push(closure)?;
    } else {
        return Err("Closure opcode requires CompiledFunction".to_string());
    }
}
```

- [ ] **Step 4: Update Call opcode to handle Closure**

Add Closure arm to the Call match:

```rust
Object::Closure(cf, free) => {
    if num_args != cf.num_params {
        return Err(format!(
            "wrong number of arguments: expected {}, got {}",
            cf.num_params, num_args
        ));
    }
    
    if let Some(max) = self.max_call_depth {
        if self.frames.len() >= max {
            return Err("call stack overflow".to_string());
        }
    }
    
    self.frames.push(CallFrame {
        return_addr: self.ip,
        base_pointer: fn_idx + 1,
        num_locals: cf.num_locals,
    });
    
    // Store free variables for this frame
    self.current_closure_free = free;
    
    for _ in num_args..cf.num_locals {
        self.push(Object::Null)?;
    }
    
    self.ip = cf.instructions_offset;
}
```

Add field to VM struct:

```rust
pub struct VM {
    // ... existing fields ...
    current_closure_free: Vec<Object>,
}
```

Initialize in VM::new():

```rust
current_closure_free: Vec::new(),
```

- [ ] **Step 5: Implement GetFree/SetFree opcodes**

Add to the run() match:

```rust
Opcode::GetFree => {
    let idx = self.read_u8() as usize;
    let value = self.current_closure_free.get(idx)
        .ok_or("free variable index out of bounds")?
        .clone();
    self.push(value)?;
}

Opcode::SetFree => {
    let idx = self.read_u8() as usize;
    let value = self.pop()?;
    if idx >= self.current_closure_free.len() {
        return Err("free variable index out of bounds".to_string());
    }
    self.current_closure_free[idx] = value;
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test test_closure -- --nocapture`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/vm/mod.rs tests/vm_functions.rs
git commit -m "feat: implement closures with free variables"
```

---

### Task 12: Implement Built-in Functions

**Files:**
- Modify: `src/vm/mod.rs:86-396` (Call opcode to handle BuiltinFunction)
- Modify: `src/compiler.rs:80-100` (register built-ins in symbol table)

**Context:** Built-in functions (len, puts, first, last, rest, push) are pre-defined and callable without compilation.

- [ ] **Step 1: Define built-in function handlers**

Add to object.rs:

```rust
pub fn builtin_len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!("len() takes 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Object::String(s) => Object::Integer(s.len() as i64),
        Object::Array(arr) => Object::Integer(arr.len() as i64),
        _ => Object::Error(format!("len() not supported for {}", args[0].type_str())),
    }
}

pub fn builtin_puts(args: Vec<Object>) -> Object {
    for arg in args {
        println!("{}", arg);
    }
    Object::Null
}

pub fn builtin_first(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!("first() takes 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Object::Array(arr) => {
            if arr.is_empty() {
                Object::Null
            } else {
                arr[0].clone()
            }
        }
        _ => Object::Error(format!("first() requires array, got {}", args[0].type_str())),
    }
}

pub fn builtin_last(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!("last() takes 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Object::Array(arr) => {
            if arr.is_empty() {
                Object::Null
            } else {
                arr[arr.len() - 1].clone()
            }
        }
        _ => Object::Error(format!("last() requires array, got {}", args[0].type_str())),
    }
}

pub fn builtin_rest(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!("rest() takes 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Object::Array(arr) => {
            if arr.is_empty() {
                Object::Array(vec![])
            } else {
                Object::Array(arr[1..].to_vec())
            }
        }
        _ => Object::Error(format!("rest() requires array, got {}", args[0].type_str())),
    }
}

pub fn builtin_push(args: Vec<Object>) -> Object {
    if args.len() != 2 {
        return Object::Error(format!("push() takes 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Object::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[1].clone());
            Object::Array(new_arr)
        }
        _ => Object::Error(format!("push() requires array, got {}", args[0].type_str())),
    }
}
```

- [ ] **Step 2: Create built-in function table**

Add to compiler.rs or a new file:

```rust
pub fn get_builtins() -> Vec<(&'static str, i32, fn(Vec<Object>) -> Object)> {
    vec![
        ("len", 1, builtin_len),
        ("puts", -1, builtin_puts),
        ("first", 1, builtin_first),
        ("last", 1, builtin_last),
        ("rest", 1, builtin_rest),
        ("push", 2, builtin_push),
    ]
}
```

- [ ] **Step 3: Register built-ins in VM**

Add to VM::new():

```rust
let builtins = get_builtins();
for (i, (name, num_params, handler)) in builtins.iter().enumerate() {
    let builtin = Object::BuiltinFunction {
        name: name.to_string(),
        num_params: *num_params,
        handler: *handler,
    };
    let name_idx = instructions.constants.len();
    instructions.constants.push(Object::String(name.to_string()));
    let builtin_idx = instructions.constants.len();
    instructions.constants.push(builtin);
    // Store mapping for GetBuiltin
}
```

Actually, simpler approach: store builtins in a separate Vec in VM:

```rust
pub struct VM {
    // ... existing fields ...
    builtins: Vec<Object>,
}
```

Initialize:

```rust
let builtins = vec![
    Object::BuiltinFunction { name: "len".to_string(), num_params: 1, handler: builtin_len },
    Object::BuiltinFunction { name: "puts".to_string(), num_params: -1, handler: builtin_puts },
    // ... etc
];
```

- [ ] **Step 4: Implement GetBuiltin opcode**

Add to run() match:

```rust
Opcode::GetBuiltin => {
    let idx = self.read_u8() as usize;
    let builtin = self.builtins.get(idx)
        .ok_or("builtin function not found")?
        .clone();
    self.push(builtin)?;
}
```

- [ ] **Step 5: Update Call opcode to handle BuiltinFunction**

Add BuiltinFunction arm to Call match:

```rust
Object::BuiltinFunction { num_params, handler, .. } => {
    if num_params >= 0 && num_args as i32 != num_params {
        return Err(format!(
            "builtin function expected {} arguments, got {}",
            num_params, num_args
        ));
    }
    
    // Pop arguments
    let mut args = Vec::new();
    for _ in 0..num_args {
        args.push(self.pop()?);
    }
    args.reverse();
    
    // Pop function object
    self.sp -= 1;
    
    // Call handler
    let result = handler(args);
    self.push(result)?;
}
```

- [ ] **Step 6: Write test**

Add to tests/vm_functions.rs:

```rust
#[test]
fn test_builtin_len() {
    let input = r#"len("hello");"#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_builtin_first() {
    let input = r#"first([1, 2, 3]);"#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(1));
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test test_builtin -- --nocapture`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src/object.rs src/vm/mod.rs tests/vm_functions.rs
git commit -m "feat: implement built-in functions (len, puts, first, last, rest, push)"
```

---

### Task 13: Implement Recursive Closures

**Files:**
- Modify: `src/compiler.rs:168-294` (FunctionLiteral compilation)
- Modify: `src/vm/mod.rs` (GetCurrentClosure opcode)

**Context:** Recursive functions need to reference themselves. GetCurrentClosure loads the current closure onto the stack for self-calls.

- [ ] **Step 1: Write failing test**

Add to tests/vm_functions.rs:

```rust
#[test]
fn test_recursive_function() {
    let input = r#"
        let factorial = fn(n) {
            if (n < 2) {
                return 1;
            }
            return n * factorial(n - 1);
        };
        factorial(5);
    "#;
    let result = eval_with_vm(input);
    assert_eq!(result, Object::Integer(120));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_recursive_function -- --nocapture`
Expected: FAIL (factorial not found in scope)

- [ ] **Step 3: Define function name in its own scope**

Update FunctionLiteral compilation in compiler.rs:

After defining parameters, add:

```rust
// For named functions, define the function itself in its scope for recursion
if let Some(name) = func_name {
    let symbol = self.symbol_table.define(name);
    // Mark as current closure
}
```

Actually, simpler: when compiling a Let statement with a function, the function name is already in the outer scope. Inside the function, we need to capture it as a free variable.

The issue is that when we compile `factorial(n - 1)` inside the function, `factorial` is not in the function's local scope. We need to resolve it from the outer scope as a free variable.

Let me revise: the symbol table's resolve method should already handle this if `factorial` is defined in the outer scope before the function body is compiled.

The problem is timing: when we compile the function literal, we haven't yet added it to the symbol table (that happens after compilation in the Let statement).

Solution: compile the function body after defining the name in the outer scope.

Update Let statement compilation:

```rust
Statement::Let(let_stmt) => {
    // Define the name first (for recursion support)
    let symbol = if self.symbol_table.outer.is_some() {
        self.symbol_table.define(let_stmt.name.value.clone())
    } else {
        self.symbol_table.define(let_stmt.name.value.clone())
    };
    
    self.compile_expression(&let_stmt.value)?;
    
    if self.symbol_table.outer.is_some() {
        self.instructions.emit(Opcode::SetLocal, &[symbol.index as u16]);
    } else {
        let name_idx = self.instructions.add_constant(Object::String(let_stmt.name.value.clone()));
        self.instructions.emit(Opcode::SetGlobal, &[name_idx as u16]);
    }
    
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_recursive_function -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/compiler.rs tests/vm_functions.rs
git commit -m "feat: support recursive function calls"
```

---

### Task 14: Parity Tests (VM vs AST Interpreter)

**Files:**
- Create: `tests/vm_ast_parity.rs` (new test file)

**Context:** Verify that VM and AST interpreter produce identical results for the same code.

- [ ] **Step 1: Create parity test file**

Create `tests/vm_ast_parity.rs`:

```rust
use project_sofia_lib::compiler::Compiler;
use project_sofia_lib::evaluator::eval;
use project_sofia_lib::lexer::Lexer;
use project_sofia_lib::object::{Environment, Object};
use project_sofia_lib::parser::Parser;
use project_sofia_lib::vm::VM;
use std::cell::RefCell;
use std::rc::Rc;

fn eval_with_ast(input: &str) -> Object {
    let lexer = Lexer::new(input.to_string());
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    let env = Rc::new(RefCell::new(Environment::new()));
    eval(project_sofia_lib::ast::Node::Program(program), env)
}

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
fn test_parity_simple_function() {
    let input = "let add = fn(x, y) { return x + y; }; add(2, 3);";
    assert_eq!(eval_with_ast(input), eval_with_vm(input));
}

#[test]
fn test_parity_closure() {
    let input = "let newAdder = fn(x) { fn(y) { x + y } }; let addTwo = newAdder(2); addTwo(3);";
    assert_eq!(eval_with_ast(input), eval_with_vm(input));
}

#[test]
fn test_parity_recursion() {
    let input = "let fib = fn(n) { if (n < 2) { n } else { fib(n-1) + fib(n-2) } }; fib(10);";
    assert_eq!(eval_with_ast(input), eval_with_vm(input));
}

#[test]
fn test_parity_local_variables() {
    let input = "let compute = fn(a, b) { let sum = a + b; let product = a * b; sum + product }; compute(2, 3);";
    assert_eq!(eval_with_ast(input), eval_with_vm(input));
}

// Add more parity tests (aim for 30+)
```

- [ ] **Step 2: Run parity tests**

Run: `cargo test --test vm_ast_parity -- --nocapture`
Expected: PASS for all tests

- [ ] **Step 3: Fix any discrepancies**

Debug and fix cases where AST and VM produce different results.

- [ ] **Step 4: Commit**

```bash
git add tests/vm_ast_parity.rs
git commit -m "test: add parity tests between VM and AST interpreter"
```

---

### Task 15: Add max_call_depth Configuration

**Files:**
- Modify: `src/vm/mod.rs` (VM struct and new() method)

**Context:** Make call depth limit configurable via VM constructor or setter method.

- [ ] **Step 1: Add setter method**

Add to VM impl:

```rust
pub fn set_max_call_depth(&mut self, depth: Option<usize>) {
    self.max_call_depth = depth;
}
```

- [ ] **Step 2: Write test**

Add to vm/mod.rs tests:

```rust
#[test]
fn test_call_depth_limit() {
    // Infinite recursion
    let input = r#"
        let recurse = fn() { recurse(); };
        recurse();
    "#;
    let lexer = Lexer::new(input.to_string());
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    let mut compiler = Compiler::new();
    let instructions = compiler.compile(&program).unwrap();
    
    let mut vm = VM::new(instructions);
    vm.set_max_call_depth(Some(100));
    
    let result = vm.run();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("call stack overflow"));
}
```

- [ ] **Step 3: Run test**

Run: `cargo test test_call_depth_limit -- --nocapture`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/mod.rs
git commit -m "feat: add configurable max_call_depth limit"
```

---

## Self-Review Checklist

After completing all tasks:

- [ ] All compiler tests pass: `cargo test --lib compiler`
- [ ] All VM tests pass: `cargo test --lib vm`
- [ ] All integration tests pass: `cargo test --tests`
- [ ] Parity tests pass: `cargo test --test vm_ast_parity`
- [ ] No compiler warnings: `cargo build`
- [ ] README.md updated with new features (if needed)

---

## Notes

- **TDD approach:** Each task starts with a failing test, then minimal implementation to pass
- **Incremental:** Each task produces working, testable code
- **Commit frequently:** One commit per task for easy rollback
- **Parity verification:** Task 14 ensures VM and AST interpreter behave identically
- **Scope boundaries:** OOP, Hash, super, implements are NOT in this plan (separate tasks)
