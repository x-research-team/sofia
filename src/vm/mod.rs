use crate::bytecode::instructions::Instructions;
use crate::bytecode::opcode::Opcode;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Размер стека виртуальной машины (в элементах Object).
const STACK_SIZE: usize = 2048;

/// Количество регистров общего назначения.
const NUM_REGISTERS: usize = 16;

/// Виртуальная машина (VM) для выполнения байткода.
/// Использует стек для хранения значений и поддерживает глобальные переменные.
pub struct VM {
    /// Инструкции байткода, которые должна выполнить VM.
    instructions: Instructions,

    /// Стек значений для выполнения операций.
    stack: Vec<Object>,

    /// Указатель стека (индекс следующей свободной позиции).
    sp: usize,

    /// Регистры общего назначения.
    registers: Vec<Object>,

    /// Указатель инструкции, текущая позиция в байткоде.
    ip: usize,

    /// Стек вызовов для управления функциями.
    frames: Vec<CallFrame>,

    /// Индекс текущего фрейма вызова.
    current_frame_index: usize,

    /// Глобальные переменные.
    globals: Rc<RefCell<HashMap<String, Object>>>,

    /// Флаг режима отладки.
    debug_mode: bool,
}

/// Информация о фрейме вызова функции.
#[derive(Debug, PartialEq, Clone)]
pub struct CallFrame {
    /// Адрес возврата в байткоде.
    pub return_addr: usize,

    /// Базовый указатель стека для локальных переменных.
    pub base_pointer: usize,

    /// Количество локальных переменных.
    pub num_locals: usize,
}

impl VM {
    /// Создает новый экземпляр виртуальной машины с заданными инструкциями.
    pub fn new(instructions: Instructions) -> Self {
        VM {
            instructions,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
            registers: vec![Object::Null; NUM_REGISTERS],
            ip: 0,
            frames: Vec::new(),
            current_frame_index: 0,
            globals: Rc::new(RefCell::new(HashMap::new())),
            debug_mode: false,
        }
    }

    /// Включить режим отладки.
    pub fn enable_debug_mode(&mut self) {
        self.debug_mode = true;
    }

    /// Отключить режим отладки.
    pub fn disable_debug_mode(&mut self) {
        self.debug_mode = false;
    }

    /// Запускает выполнение байткода.
    /// Возвращает результат исполнения (верхний элемент стека) или ошибку.
    pub fn run(&mut self) -> Result<Object, String> {
        while self.ip < self.instructions.bytes.len() {
            if self.debug_mode {
                eprintln!("IP: {}, SP: {}", self.ip, self.sp);
            }

            let opcode = Opcode::from_byte(self.instructions.bytes[self.ip]).ok_or_else(|| {
                format!("Неизвестный опкод: {}", self.instructions.bytes[self.ip])
            })?;

            if self.debug_mode {
                eprintln!("Executing: {}", opcode.mnemonic());
            }

            self.ip += 1;

            match opcode {
                Opcode::Constant => {
                    let const_index = self.read_u16() as usize;
                    let constant = self
                        .instructions
                        .get_constant(const_index)
                        .ok_or_else(|| format!("Константа с индексом {} не найдена", const_index))?
                        .clone();
                    self.push(constant)?;
                }

                Opcode::Pop => {
                    self.pop()?;
                }

                Opcode::True => {
                    self.push(Object::Boolean(true))?;
                }

                Opcode::False => {
                    self.push(Object::Boolean(false))?;
                }

                Opcode::Null => {
                    self.push(Object::Null)?;
                }

                Opcode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.apply_operation(&a, &b, "+")?;
                    self.push(result)?;
                }

                Opcode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.apply_operation(&a, &b, "-")?;
                    self.push(result)?;
                }

                Opcode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.apply_operation(&a, &b, "*")?;
                    self.push(result)?;
                }

                Opcode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.apply_operation(&a, &b, "/")?;
                    self.push(result)?;
                }

                Opcode::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.apply_operation(&a, &b, "%")?;
                    self.push(result)?;
                }

                Opcode::Pow => {
                    let exp = self.pop()?;
                    let base = self.pop()?;
                    let result = self.apply_operation(&base, &exp, "**")?;
                    self.push(result)?;
                }

                Opcode::Neg => {
                    let a = self.pop()?;
                    match a {
                        Object::Integer(n) => self.push(Object::Integer(-n))?,
                        _ => {
                            return Err(format!(
                                "Невозможно применить унарный минус к {}",
                                a.type_str()
                            ))
                        }
                    }
                }

                Opcode::Not => {
                    let a = self.pop()?;
                    let result = match a {
                        Object::Boolean(b) => Object::Boolean(!b),
                        Object::Null => Object::Boolean(true),
                        _ => Object::Boolean(false),
                    };
                    self.push(result)?;
                }

                Opcode::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.is_truthy(&a) && self.is_truthy(&b);
                    self.push(Object::Boolean(result))?;
                }

                Opcode::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.is_truthy(&a) || self.is_truthy(&b);
                    self.push(Object::Boolean(result))?;
                }

                Opcode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Boolean(a == b))?;
                }

                Opcode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Boolean(a != b))?;
                }

                Opcode::GreaterThan => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.compare_objects(&a, &b)?;
                    self.push(Object::Boolean(result > 0))?;
                }

                Opcode::LessThan => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.compare_objects(&a, &b)?;
                    self.push(Object::Boolean(result < 0))?;
                }

                Opcode::GreaterThanOrEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.compare_objects(&a, &b)?;
                    self.push(Object::Boolean(result >= 0))?;
                }

                Opcode::LessThanOrEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.compare_objects(&a, &b)?;
                    self.push(Object::Boolean(result <= 0))?;
                }

                Opcode::Jump => {
                    let pos = self.read_u16() as usize;
                    self.ip = pos;
                }

                Opcode::JumpIfFalse => {
                    let pos = self.read_u16() as usize;
                    let condition = self.pop()?;
                    if !self.is_truthy(&condition) {
                        self.ip = pos;
                    }
                }

                Opcode::JumpIfTrue => {
                    let pos = self.read_u16() as usize;
                    let condition = self.pop()?;
                    if self.is_truthy(&condition) {
                        self.ip = pos;
                    }
                }

                Opcode::Return => {
                    if self.frames.is_empty() {
                        let result = if self.sp > 0 {
                            self.stack[self.sp - 1].clone()
                        } else {
                            Object::Null
                        };
                        return Ok(result);
                    }
                    // TODO: Реализовать возврат из функции с фреймами вызова
                    return Err("Return из функций пока не реализован".to_string());
                }

                Opcode::GetGlobal => {
                    let name_idx = self.read_u16() as usize;
                    let name = self
                        .instructions
                        .get_constant(name_idx)
                        .ok_or_else(|| format!("Константа {} не найдена", name_idx))?;
                    if let Object::String(var_name) = name {
                        // Используем блок scope для освобождения borrow перед push()
                        let value = {
                            let globals = self.globals.borrow();
                            globals.get(var_name).cloned().unwrap_or(Object::Null)
                        };
                        self.push(value)?;
                    } else {
                        return Err(format!("Ожидалось имя переменной, получено {}", name));
                    }
                }

                Opcode::SetGlobal => {
                    let name_idx = self.read_u16() as usize;
                    // Клонируем name перед вызовом pop() чтобы избежать borrow конфликта
                    let name = {
                        self.instructions
                            .get_constant(name_idx)
                            .ok_or_else(|| format!("Константа {} не найдена", name_idx))?
                            .clone()
                    };
                    let value = self.pop()?;
                    if let Object::String(var_name) = name {
                        self.globals.borrow_mut().insert(var_name.clone(), value);
                    } else {
                        return Err(format!("Ожидалось имя переменной, получено {}", name));
                    }
                }

                Opcode::GetLocal => {
                    let idx = self.read_u8() as usize;
                    let value = self.stack[idx].clone();
                    self.push(value)?;
                }

                Opcode::SetLocal => {
                    let idx = self.read_u8() as usize;
                    let value = self.pop()?;
                    self.stack[idx] = value;
                }

                Opcode::Array => {
                    let len = self.read_u16() as usize;
                    let mut elements = Vec::new();
                    for _ in 0..len {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    self.push(Object::Array(elements))?;
                }

                Opcode::Hash => {
                    let num_pairs = self.read_u16() as usize;
                    let mut hash = HashMap::new();
                    for _ in 0..num_pairs {
                        let value = self.pop()?;
                        let key = self.pop()?;
                        if let Object::String(k) = key {
                            hash.insert(k, value);
                        } else {
                            return Err(format!(
                                "Ключ хэша должен быть строкой, получено {}",
                                key.type_str()
                            ));
                        }
                    }
                    // TODO: Реализовать правильный объект Hash с Object::Hash
                    self.push(Object::Null)?; // Временное решение
                }

                Opcode::Index => {
                    let index = self.pop()?;
                    let array = self.pop()?;
                    match (array, index) {
                        (Object::Array(arr), Object::Integer(idx)) => {
                            if idx < 0 || idx as usize >= arr.len() {
                                self.push(Object::Null)?;
                            } else {
                                self.push(arr[idx as usize].clone())?;
                            }
                        }
                        _ => return Err("Неподдерживаемая операция индексирования".to_string()),
                    }
                }

                Opcode::Call
                | Opcode::New
                | Opcode::Class
                | Opcode::GetProperty
                | Opcode::SetProperty
                | Opcode::This
                | Opcode::Super
                | Opcode::MapToAst => {
                    return Err(format!("Опкод {} пока не реализован", opcode.mnemonic()));
                }

                Opcode::NoOp => {
                    // Ничего не делаем
                }
            }
        }

        // Возвращаем верхний элемент стека как результат
        if self.sp > 0 {
            Ok(self.stack[self.sp - 1].clone())
        } else {
            Ok(Object::Null)
        }
    }

    /// Поместить значение на стек.
    fn push(&mut self, obj: Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err("Переполнение стека".to_string());
        }
        self.stack[self.sp] = obj;
        self.sp += 1;
        Ok(())
    }

    /// Взять значение со стека.
    fn pop(&mut self) -> Result<Object, String> {
        if self.sp == 0 {
            return Err("Underflow стека".to_string());
        }
        self.sp -= 1;
        Ok(self.stack[self.sp].clone())
    }

    /// Прочитать двухбайтовый операнд и увеличить IP.
    fn read_u16(&mut self) -> u16 {
        let high = self.instructions.bytes[self.ip] as u16;
        let low = self.instructions.bytes[self.ip + 1] as u16;
        self.ip += 2;
        (high << 8) | low
    }

    /// Прочитать однобайтовый операнд и увеличить IP.
    fn read_u8(&mut self) -> u8 {
        let byte = self.instructions.bytes[self.ip];
        self.ip += 1;
        byte
    }

    /// Проверить является ли значение "истинным" (truthy).
    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    /// Сравнить два объекта. Возвращает: < 0 если a < b, 0 если a == b, > 0 если a > b.
    fn compare_objects(&self, a: &Object, b: &Object) -> Result<i32, String> {
        match (a, b) {
            (Object::Integer(x), Object::Integer(y)) => Ok(if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }),
            (Object::String(x), Object::String(y)) => Ok(if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }),
            _ => Err(format!(
                "Невозможно сравнить {} и {}",
                a.type_str(),
                b.type_str()
            )),
        }
    }

    /// Применить бинарную операцию к двум объектам.
    fn apply_operation(&self, a: &Object, b: &Object, op: &str) -> Result<Object, String> {
        match (a, b) {
            (Object::Integer(x), Object::Integer(y)) => match op {
                "+" => Ok(Object::Integer(x + y)),
                "-" => Ok(Object::Integer(x - y)),
                "*" => Ok(Object::Integer(x * y)),
                "/" => {
                    if *y == 0 {
                        Err("Деление на ноль".to_string())
                    } else {
                        Ok(Object::Integer(x / y))
                    }
                }
                "%" => {
                    if *y == 0 {
                        Err("Деление на ноль в операции модуля".to_string())
                    } else {
                        Ok(Object::Integer(x % y))
                    }
                }
                "**" => {
                    if *y < 0 {
                        Err("Отрицательные степени не поддерживаются для целых чисел".to_string())
                    } else {
                        Ok(Object::Integer(x.pow(*y as u32)))
                    }
                }
                _ => Err(format!("Неизвестная операция: {}", op)),
            },
            (Object::String(x), Object::String(y)) => match op {
                "+" => Ok(Object::String(format!("{}{}", x, y))),
                _ => Err(format!("Неподдерживаемая операция для строк: {}", op)),
            },
            _ => Err(format!(
                "Операция {} не поддерживается для {} и {}",
                op,
                a.type_str(),
                b.type_str()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::instructions::Instructions;
    use crate::bytecode::opcode::Opcode;

    #[test]
    fn test_vm_constant() {
        // Тестируем: Constant(10)
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![Opcode::Constant as u8, 0, 0]; // Opcode + 2-byte operand 0

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(10));
    }

    #[test]
    fn test_vm_add() {
        // Тестируем: Constant(5), Constant(10), Add
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(10)
            Opcode::Add as u8, // Add
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(15));
    }

    #[test]
    fn test_vm_sub() {
        // Тестируем: Constant(20), Constant(7), Sub → 20 - 7 = 13
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(20));
        instr.constants.push(Object::Integer(7));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(20)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(7)
            Opcode::Sub as u8, // Sub
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(13));
    }

    #[test]
    fn test_vm_mul() {
        // Тестируем: Constant(4), Constant(5), Mul → 4 * 5 = 20
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(4));
        instr.constants.push(Object::Integer(5));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(4)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(5)
            Opcode::Mul as u8, // Mul
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(20));
    }

    #[test]
    fn test_vm_div() {
        // Тестируем: Constant(20), Constant(4), Div → 20 / 4 = 5
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(20));
        instr.constants.push(Object::Integer(4));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(20)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(4)
            Opcode::Div as u8, // Div
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(5));
    }

    #[test]
    fn test_vm_div_by_zero() {
        // Тестируем ошибку: деление на ноль
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::Integer(0));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(10)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(0)
            Opcode::Div as u8, // Div
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ноль"));
    }

    #[test]
    fn test_vm_mod() {
        // Тестируем: Constant(17), Constant(5), Mod → 17 % 5 = 2
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(17));
        instr.constants.push(Object::Integer(5));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(17)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(5)
            Opcode::Mod as u8, // Mod
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(2));
    }

    #[test]
    fn test_vm_pow() {
        // Тестируем: Constant(2), Constant(8), Pow → 2 ^ 8 = 256
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(2));
        instr.constants.push(Object::Integer(8));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(2)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(8)
            Opcode::Pow as u8, // Pow
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(256));
    }

    #[test]
    fn test_vm_neg() {
        // Тестируем: Constant(42), Neg → -42
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(42));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0,                 // Constant(42)
            Opcode::Neg as u8, // Neg
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(-42));
    }

    #[test]
    fn test_vm_not_true() {
        // Тестируем: True, Not → False
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::True as u8, // True
            Opcode::Not as u8,  // Not
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(false));
    }

    #[test]
    fn test_vm_not_false() {
        // Тестируем: False, Not → True
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::False as u8, // False
            Opcode::Not as u8,   // Not
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_and_true_true() {
        // Тестируем: True, True, And → True
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::True as u8, // True
            Opcode::True as u8, // True
            Opcode::And as u8,  // And
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_and_true_false() {
        // Тестируем: True, False, And → False
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::True as u8,  // True
            Opcode::False as u8, // False
            Opcode::And as u8,   // And
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(false));
    }

    #[test]
    fn test_vm_or_false_false() {
        // Тестируем: False, False, Or → False
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::False as u8, // False
            Opcode::False as u8, // False
            Opcode::Or as u8,    // Or
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(false));
    }

    #[test]
    fn test_vm_or_true_false() {
        // Тестируем: True, False, Or → True
        let mut instr = Instructions::new();
        instr.bytes = vec![
            Opcode::True as u8,  // True
            Opcode::False as u8, // False
            Opcode::Or as u8,    // Or
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_equal_integers() {
        // Тестируем: Constant(5), Constant(5), Equal → True
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(5));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                   // Constant(5)
            Opcode::Equal as u8, // Equal
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_not_equal_integers() {
        // Тестируем: Constant(5), Constant(7), NotEqual → True
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(7));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                      // Constant(7)
            Opcode::NotEqual as u8, // NotEqual
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_greater_than() {
        // Тестируем: Constant(10), Constant(5), GreaterThan → True (10 > 5)
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::Integer(5));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(10)
            Opcode::Constant as u8,
            0,
            1,                         // Constant(5)
            Opcode::GreaterThan as u8, // GreaterThan
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_less_than() {
        // Тестируем: Constant(5), Constant(10), LessThan → True (5 < 10)
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                      // Constant(10)
            Opcode::LessThan as u8, // LessThan
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_greater_than_or_equal() {
        // Тестируем: Constant(10), Constant(10), GreaterThanOrEqual → True
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(10)
            Opcode::Constant as u8,
            0,
            1,                                // Constant(10)
            Opcode::GreaterThanOrEqual as u8, // GreaterThanOrEqual
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_less_than_or_equal() {
        // Тестируем: Constant(5), Constant(10), LessThanOrEqual → True
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                             // Constant(10)
            Opcode::LessThanOrEqual as u8, // LessThanOrEqual
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_pop() {
        // Тестируем: Constant(10), Pop → Null (стек пуст)
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0,                 // Constant(10)
            Opcode::Pop as u8, // Pop
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Null);
    }

    #[test]
    fn test_vm_multiple_operations() {
        // Тестируем: Constant(5), Constant(10), Add, Constant(3), Mul → (5 + 10) * 3 = 45
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::Integer(3));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Constant as u8,
            0,
            1,                 // Constant(10)
            Opcode::Add as u8, // Add (результат 15)
            Opcode::Constant as u8,
            0,
            2,                 // Constant(3)
            Opcode::Mul as u8, // Mul (15 * 3 = 45)
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(45));
    }

    #[test]
    fn test_vm_null() {
        // Тестируем: Null
        let mut instr = Instructions::new();
        instr.bytes = vec![Opcode::Null as u8];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Null);
    }

    #[test]
    fn test_vm_true() {
        // Тестируем: True
        let mut instr = Instructions::new();
        instr.bytes = vec![Opcode::True as u8];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_vm_false() {
        // Тестируем: False
        let mut instr = Instructions::new();
        instr.bytes = vec![Opcode::False as u8];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Boolean(false));
    }

    #[test]
    fn test_vm_stack_overflow() {
        // Пытаемся переполнить стек
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(1));
        let mut bytes = vec![];
        // Добавляем больше операций, чем вмещает стек
        for _ in 0..(STACK_SIZE + 10) {
            bytes.push(Opcode::Constant as u8);
            bytes.push(0);
            bytes.push(0);
        }
        instr.bytes = bytes;

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Переполнение"));
    }

    #[test]
    fn test_vm_stack_underflow() {
        // Попытка взять значение со пустого стека
        let mut instr = Instructions::new();
        instr.bytes = vec![Opcode::Pop as u8];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Underflow"));
    }

    #[test]
    fn test_vm_string_concatenation() {
        // Тестируем: Constant("Hello"), Constant(" World"), Add → "Hello World"
        let mut instr = Instructions::new();
        instr.constants.push(Object::String("Hello".to_string()));
        instr.constants.push(Object::String(" World".to_string()));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant("Hello")
            Opcode::Constant as u8,
            0,
            1,                 // Constant(" World")
            Opcode::Add as u8, // Add (конкатенация строк)
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::String("Hello World".to_string()));
    }

    #[test]
    fn test_vm_jump() {
        // Тестируем: Jump(3) - пропускаем одну инструкцию
        // Constant(5), Jump(4), Constant(10), Constant(20)
        // Результат должен быть 20 (пропускаем Constant(10))
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(5));
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::Integer(20));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(5)
            Opcode::Jump as u8,
            0,
            6, // Jump to 6 (пропускаем следующие 3 байта)
            Opcode::Constant as u8,
            0,
            1, // Constant(10) - пропускается
            Opcode::Constant as u8,
            0,
            2, // Constant(20)
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(20));
    }

    #[test]
    fn test_vm_get_global() {
        // Тестируем: SetGlobal("x", 10), GetGlobal("x") → 10
        let mut instr = Instructions::new();
        instr.constants.push(Object::Integer(10));
        instr.constants.push(Object::String("x".to_string()));
        instr.bytes = vec![
            Opcode::Constant as u8,
            0,
            0, // Constant(10)
            Opcode::SetGlobal as u8,
            0,
            1, // SetGlobal("x", 10)
            Opcode::GetGlobal as u8,
            0,
            1, // GetGlobal("x")
        ];

        let mut vm = VM::new(instr);
        let result = vm.run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Object::Integer(10));
    }
}
