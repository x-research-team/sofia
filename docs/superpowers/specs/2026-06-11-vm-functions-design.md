# Дизайн: Функции в VM (Call, Return, Closures, Built-ins)

## 1. Обзор

Реализовать полную поддержку функций в байткод-компиляторе и VM для языка SOFIA. Это включает: простые функции, аргументы, локальные переменные, замыкания (closures), рекурсивные замыкания, built-in функции, first-class functions.

**Подход:** Monkey-паттерн (из "Writing a Compiler in Go" — Thorsten Ball). Единый стек с basePointer-разделением фреймов. Замыкания через копирование значений в массив `free[]`.

**Паритетность:** VM и AST-интерпретатор должны давать идентичные результаты для всех тестов.

**Лимит глубины вызовов:** Настраиваемый через `VM::max_call_depth: Option<usize>`. Если `None` — без ограничения (до OOM/stack overflow).

## 2. Структуры данных

### 2.1 Object::CompiledFunction

Новый вариант в `enum Object` (`src/object.rs`):

```rust
CompiledFunction {
    instructions_offset: usize,  // Смещение начала байткода функции в Instructions.bytes
    num_locals: usize,           // Локальные переменные (включая параметры)
    num_params: usize,           // Количество параметров
}
```

### 2.2 Object::Closure

Обёртка для функций с захваченными переменными:

```rust
Closure {
    function: Box<CompiledFunction>,  // Скомпилированная функция (inlined, не Object)
    free: Vec<Object>,                 // Захваченные значения (копии при создании)
}
```

Где `CompiledFunction` — отдельная структура (не variant Object):
```rust
pub struct CompiledFunction {
    pub instructions_offset: usize,
    pub num_locals: usize,
    pub num_params: usize,
}
```

`Object::Closure` оборачивает `CompiledFunction` и добавляет `free[]`.
Простая функция без замыканий хранится как `Object::CompiledFunction(CompiledFunction)` напрямую.

### 2.3 Object::BuiltinFunction

```rust
BuiltinFunction {
    name: String,
    num_params: i32,  // -1 = variadic
    handler: fn(Vec<Object>) -> Object,
}
```

### 2.4 CallFrame (обновлённый)

```rust
pub struct CallFrame {
    pub return_addr: usize,     // IP вызывающей функции
    pub base_pointer: usize,    // Начало стекового окна для локальных
    pub num_locals: usize,      // Количество локальных переменных
}
```

### 2.5 VM (новые поля)

```rust
pub struct VM {
    // ... существующие поля ...
    pub max_call_depth: Option<usize>,  // Лимит глубины вызовов
}
```

### 2.6 Compiler: Symbol Table

```rust
#[derive(Debug, Clone)]
enum SymbolScope {
    Global,
    Local(usize),   // Индекс в стековом окне
    Free(usize),    // Индекс в массиве free-переменных closure
    Builtin(usize), // Индекс в таблице built-in функций
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    scope: SymbolScope,
    index: usize,
}

struct SymbolTable {
    outer: Option<Box<SymbolTable>>,
    store: HashMap<String, Symbol>,
    num_definitions: usize,
    free_symbols: Vec<Symbol>,  // Символы, захваченные из внешнего scope
}
```

## 3. Новые опкоды

| Опкод | Операнд | Описание |
|---|---|---|
| `Call` | `u8` (num_args) | Вызов функции. Функция и аргументы уже на стеке |
| `ReturnValue` | — | Возврат значения (top of stack) из функции |
| `Return` | — | Возврат без значения (push Null) |
| `GetLocal` | `u8` (idx) | Загрузить локальную переменную |
| `SetLocal` | `u8` (idx) | Сохранить локальную переменную |
| `GetFree` | `u8` (idx) | Загрузить free-переменную из closure |
| `SetFree` | `u8` (idx) | Сохранить free-переменную |
| `GetCurrentClosure` | — | Загрузить текущий closure (для рекурсии) |
| `MakeClosure` | `u8` (num_free) | Создать Closure из CompiledFunction + free-значений |
| `GetBuiltin` | `u8` (idx) | Загрузить built-in функцию |
| `Closure` | `u16` (const_idx) + `u8` (num_free) | Загрузить функцию как closure |

## 4. Stack layout

### До Call:
```
│ ...        │
│ func_obj   │  ← Object::Closure или CompiledFunction
│ arg1       │
│ arg2       │
  sp = N
```

### После Call (внутри функции):
```
│ ...        │
│ func_obj   │  ← base_pointer - 1 (func_obj для GetFree)
│ arg1       │  ← base_pointer + 0 (параметр 0)
│ arg2       │  ← base_pointer + 1 (параметр 1)
│ local1     │  ← base_pointer + num_params
│ local2     │
│ temp       │
  sp
```

### После Return:
```
│ ...        │
│ result     │  ← результат функции
  sp
```

## 5. Компилятор

### 5.1 FunctionLiteral

1. Push новый scope в symbol table
2. Определить параметры как `Local(0..n)` в новом scope
3. Компилировать тело функции
4. Записать offset начала тела и num_locals
5. Pop scope, collect free_symbols
6. Если есть free_symbols → эмитировать `Closure` опкод
7. Иначе → эмитировать `Constant(compiled_fn)`
8. Эмитировать `Jump` для пропуска тела функции в основном потоке

### 5.2 CallExpression

1. Компилировать выражение-функцию (push на стек)
2. Компилировать каждый аргумент (push на стек)
3. Эмитировать `Call(num_args)`

### 5.3 Let внутри функции

1. Компилировать значение
2. Определить переменную как `Local(next_idx)` в текущем scope
3. Эмитировать `SetLocal(idx)` (значение остаётся на стеке в позиции локальной)

### 5.4 Identifier resolution

```
lookup(name) →
  Local(idx)   → emit GetLocal(idx)
  Free(idx)    → emit GetFree(idx)
  Global(idx)  → emit GetGlobal(idx)
  Builtin(idx) → emit GetBuiltin(idx)
  NotFound     → error
```

### 5.5 Замыкания

При выходе из scope функции:
1. Собрать все `Free` символы
2. Для каждой free-переменной эмитировать загрузку значения (GetLocal/GetFree из внешнего scope)
3. Эмитировать `MakeClosure(num_free)` или `Closure(const_idx, num_free)`

## 6. VM: выполнение

### 6.1 Opcode::Call

```
1. fn_idx = sp - 1 - num_args
2. func_obj = stack[fn_idx]
3. match func_obj:
   - CompiledFunction: проверить num_args == num_params
     → push CallFrame { return_addr: ip, base_pointer: fn_idx+1, num_locals }
     → инициализировать локальные Null (num_params..num_locals)
     → ip = instructions_offset
   - Closure: то же + сохранить free[] для GetFree
   - BuiltinFunction: pop args, вызвать handler, push result
   - _ → Error("not a function")
4. Проверить max_call_depth
```

### 6.2 Opcode::Return / ReturnValue

```
ReturnValue:
  1. result = pop()  (top of stack — возвращаемое значение)
  2. frame = frames.pop()
  3. sp = frame.base_pointer - 1  (убрать func_obj + args + locals)
  4. ip = frame.return_addr
  5. push(result)

Return (без значения):
  1. push(Null)
  2. то же что ReturnValue
```

### 6.3 Opcode::GetLocal / SetLocal

```
GetLocal(idx):
  bp = current_frame().base_pointer
  push(stack[bp + idx])

SetLocal(idx):
  bp = current_frame().base_pointer
  stack[bp + idx] = pop()
```

### 6.4 Opcode::GetFree / SetFree

```
GetFree(idx):
  closure = current_closure()  // closure текущего фрейма
  push(closure.free[idx])

SetFree(idx):
  closure = current_closure()
  closure.free[idx] = pop()
```

### 6.5 Opcode::MakeClosure

```
1. num_free = read_u8()
2. free = pop num_free values (reverse order)
3. func_obj = pop()  // CompiledFunction
4. closure = Object::Closure { function, free }
5. push(closure)
```

### 6.6 Opcode::GetBuiltin

```
1. idx = read_u8()
2. builtin = builtins[idx]
3. push(builtin)
```

## 7. Built-in функции

| Имя | Параметры | Описание |
|---|---|---|
| `len` | 1 | Длина строки или массива |
| `puts` | variadic | Вывести значения в stdout |
| `first` | 1 | Первый элемент массива |
| `last` | 1 | Последний элемент массива |
| `rest` | 1 | Массив без первого элемента |
| `push` | 2 | Добавить элемент в массив (возвращает новый) |

## 8. Рекурсия

Рекурсия работает через именованное замыкание. При компиляции `let fib = fn(n) { ... fib(n-1) ... }`:

1. Компилятор создаёт `let`-биндинг `fib` в текущем scope
2. При входе в тело функции `fib` определяется как `Local` в своём собственном scope
3. `GetCurrentClosure` загружает closure текущего фрейма на стек
4. При вызове `fib(n-1)` — это `GetCurrentClosure` + `Call`

Это позволяет функции ссылаться на саму себя через closure, даже если она была создана внутри другой функции.

## 9. Тестирование

### 9.1 Unit-тесты компилятора

Для каждой новой фичи:
- Проверить генерируемые опкоды
- Проверить symbol table resolution

### 9.2 Unit-тесты VM

Для каждой новой фичи:
- Построить байткод вручную
- Проверить результат выполнения

### 9.3 Интеграционные тесты

Проверка через `compile → VM.run`:
- Простые функции: `fn(x) { x }`
- Функции с аргументами: `fn(a, b) { a + b }`
- Локальные переменные: `fn() { let x = 1; x + 2 }`
- Замыкания: `fn(x) { fn(y) { x + y } }`
- Рекурсия: `fn fib(n) { if (n < 2) { n } else { fib(n-1) + fib(n-2) } }`
- Higher-order functions: `let map = fn(arr, f) { ... }`
- Built-in функции: `len("hello")`, `puts("test")`

### 9.4 Паритет-тесты

```rust
#[test]
fn test_parity_functions() {
    let cases = vec![
        "let add = fn(x, y) { x + y }; add(2, 3);",
        "let double = fn(x) { x * 2 }; double(5);",
        "let newAdder = fn(x) { fn(y) { x + y } }; let addTwo = newAdder(2); addTwo(3);",
        "let fib = fn(n) { if (n < 2) { n } else { fib(n-1) + fib(n-2) } }; fib(10);",
        "len([1, 2, 3]);",
    ];
    for input in cases {
        let ast_result = eval_with_ast(input);
        let vm_result = eval_with_vm(input);
        assert_eq!(ast_result, vm_result, "Parity failed for: {}", input);
    }
}
```

Цель: ≥30 паритет-тестов.

## 10. Границы scope (что НЕ включено)

- OOP в VM (Class, New, GetProperty, SetProperty, This, Super) — отдельная задача
- Hash в VM (полноценная реализация) — отдельная задача
- `super` в evaluator — отдельная задача
- `implements` в parser — отдельная задача
