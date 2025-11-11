/// Определяет коды операций (опкоды) для виртуальной машины.
/// Каждый опкод представляет собой инструкцию, которую VM может выполнить.
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    // === КОНСТАНТЫ И ЛИТЕРАЛЫ ===
    /// Загружает константу на стек. Операнд: индекс константы (2 байта).
    Constant = 1,

    // === АРИФМЕТИЧЕСКИЕ И ЛОГИЧЕСКИЕ ОПЕРАЦИИ ===
    /// Добавляет два верхних элемента стека.
    Add = 2,
    /// Вычитает два верхних элемента стека.
    Sub = 3,
    /// Умножает два верхних элемента стека.
    Mul = 4,
    /// Делит два верхних элемента стека.
    Div = 5,
    /// Модуль (остаток от деления) двух верхних элементов стека.
    Mod = 6,
    /// Возведение в степень.
    Pow = 7,
    /// Унарный минус.
    Neg = 8,
    /// Логическое НЕ.
    Not = 9,
    /// Логическое И.
    And = 10,
    /// Логическое ИЛИ.
    Or = 11,

    // === ОПЕРАЦИИ СРАВНЕНИЯ ===
    /// Сравнивает два верхних элемента стека на равенство.
    Equal = 12,
    /// Сравнивает два верхних элемента стека на неравенство.
    NotEqual = 13,
    /// Сравнивает два верхних элемента стека: больше ли первый второго.
    GreaterThan = 14,
    /// Сравнивает два верхних элемента стека: меньше ли первый второго.
    LessThan = 15,
    /// Сравнивает два верхних элемента стека: больше или равно.
    GreaterThanOrEqual = 16,
    /// Сравнивает два верхних элемента стека: меньше или равно.
    LessThanOrEqual = 17,

    // === УПРАВЛЕНИЕ ПОТОКОМ ===
    /// Безусловный переход. Операнд: смещение (2 байта).
    Jump = 18,
    /// Условный переход, если верхний элемент стека ложен. Операнд: смещение (2 байта).
    JumpIfFalse = 19,
    /// Условный переход, если верхний элемент стека истинен. Операнд: смещение (2 байта).
    JumpIfTrue = 20,
    /// Вызов функции. Операнд: количество аргументов (1 байт).
    Call = 21,
    /// Возврат из функции.
    Return = 22,

    // === РАБОТА С ПЕРЕМЕННЫМИ ===
    /// Получить глобальную переменную. Операнд: индекс имени в пуле констант (2 байта).
    GetGlobal = 23,
    /// Установить глобальную переменную. Операнд: индекс имени в пуле констант (2 байта).
    SetGlobal = 24,
    /// Получить локальную переменную. Операнд: индекс локальной переменной (1 байт).
    GetLocal = 25,
    /// Установить локальную переменную. Операнд: индекс локальной переменной (1 байт).
    SetLocal = 26,

    // === РАБОТА СО СТРУКТУРАМИ ДАННЫХ ===
    /// Создать массив. Операнд: количество элементов (2 байта).
    Array = 27,
    /// Создать хэш-таблицу (объект). Операнд: количество пар ключ-значение (2 байта).
    Hash = 28,
    /// Доступ по индексу.
    Index = 29,

    // === РАБОТА С КЛАССАМИ И ОБЪЕКТАМИ ===
    /// Объявить класс. Операнд: индекс имени класса в пуле констант (2 байта).
    Class = 30,
    /// Получить свойство объекта. Операнд: индекс имени свойства в пуле констант (2 байта).
    GetProperty = 31,
    /// Установить свойство объекта. Операнд: индекс имени свойства в пуле констант (2 байта).
    SetProperty = 32,
    /// Создать новый экземпляр класса/структуры. Операнд: количество аргументов конструктора (1 байт).
    New = 33,
    /// Загрузить текущий экземпляр (this).
    This = 34,
    /// Загрузить родительский класс (super).
    Super = 35,

    // === СПЕЦИАЛЬНЫЕ ОПЕРАЦИИ ===
    /// Выталкивает верхний элемент стека.
    Pop = 36,
    /// Загружает булево значение "истина" на стек.
    True = 37,
    /// Загружает булево значение "ложь" на стек.
    False = 38,
    /// Загружает значение "null" на стек.
    Null = 39,
    /// Операция без действия.
    NoOp = 40,
    /// Связать текущий опкод с узлом AST (для отладки). Операнд: ID узла AST (2 байта).
    MapToAst = 41,
}

impl Opcode {
    /// Возвращает мнемоническое представление опкода.
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Opcode::Constant => "CONSTANT",
            Opcode::Add => "ADD",
            Opcode::Sub => "SUB",
            Opcode::Mul => "MUL",
            Opcode::Div => "DIV",
            Opcode::Mod => "MOD",
            Opcode::Pow => "POW",
            Opcode::Neg => "NEG",
            Opcode::Not => "NOT",
            Opcode::And => "AND",
            Opcode::Or => "OR",
            Opcode::Equal => "EQUAL",
            Opcode::NotEqual => "NOT_EQUAL",
            Opcode::GreaterThan => "GREATER_THAN",
            Opcode::LessThan => "LESS_THAN",
            Opcode::GreaterThanOrEqual => "GREATER_THAN_OR_EQUAL",
            Opcode::LessThanOrEqual => "LESS_THAN_OR_EQUAL",
            Opcode::Jump => "JUMP",
            Opcode::JumpIfFalse => "JUMP_IF_FALSE",
            Opcode::JumpIfTrue => "JUMP_IF_TRUE",
            Opcode::Call => "CALL",
            Opcode::Return => "RETURN",
            Opcode::GetGlobal => "GET_GLOBAL",
            Opcode::SetGlobal => "SET_GLOBAL",
            Opcode::GetLocal => "GET_LOCAL",
            Opcode::SetLocal => "SET_LOCAL",
            Opcode::Array => "ARRAY",
            Opcode::Hash => "HASH",
            Opcode::Index => "INDEX",
            Opcode::Class => "CLASS",
            Opcode::GetProperty => "GET_PROPERTY",
            Opcode::SetProperty => "SET_PROPERTY",
            Opcode::New => "NEW",
            Opcode::This => "THIS",
            Opcode::Super => "SUPER",
            Opcode::Pop => "POP",
            Opcode::True => "TRUE",
            Opcode::False => "FALSE",
            Opcode::Null => "NULL",
            Opcode::NoOp => "NO_OP",
            Opcode::MapToAst => "MAP_TO_AST",
        }
    }

    /// Возвращает размеры операндов для данного опкода.
    /// Например, [2] означает один 2-байтовый операнд.
    pub fn operand_widths(&self) -> &'static [u8] {
        match self {
            // Опкоды с двухбайтовым операндом
            Opcode::Constant
            | Opcode::Jump
            | Opcode::JumpIfFalse
            | Opcode::JumpIfTrue
            | Opcode::GetGlobal
            | Opcode::SetGlobal
            | Opcode::Array
            | Opcode::Hash
            | Opcode::Class
            | Opcode::GetProperty
            | Opcode::SetProperty
            | Opcode::MapToAst => &[2],

            // Опкоды с однобайтовым операндом
            Opcode::GetLocal | Opcode::SetLocal | Opcode::Call | Opcode::New => &[1],

            // Опкоды без операндов
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Mod
            | Opcode::Pow
            | Opcode::Neg
            | Opcode::Not
            | Opcode::And
            | Opcode::Or
            | Opcode::Equal
            | Opcode::NotEqual
            | Opcode::GreaterThan
            | Opcode::LessThan
            | Opcode::GreaterThanOrEqual
            | Opcode::LessThanOrEqual
            | Opcode::Index
            | Opcode::Return
            | Opcode::Pop
            | Opcode::True
            | Opcode::False
            | Opcode::Null
            | Opcode::This
            | Opcode::Super
            | Opcode::NoOp => &[],
        }
    }

    /// Преобразовать байт в опкод.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Opcode::Constant),
            2 => Some(Opcode::Add),
            3 => Some(Opcode::Sub),
            4 => Some(Opcode::Mul),
            5 => Some(Opcode::Div),
            6 => Some(Opcode::Mod),
            7 => Some(Opcode::Pow),
            8 => Some(Opcode::Neg),
            9 => Some(Opcode::Not),
            10 => Some(Opcode::And),
            11 => Some(Opcode::Or),
            12 => Some(Opcode::Equal),
            13 => Some(Opcode::NotEqual),
            14 => Some(Opcode::GreaterThan),
            15 => Some(Opcode::LessThan),
            16 => Some(Opcode::GreaterThanOrEqual),
            17 => Some(Opcode::LessThanOrEqual),
            18 => Some(Opcode::Jump),
            19 => Some(Opcode::JumpIfFalse),
            20 => Some(Opcode::JumpIfTrue),
            21 => Some(Opcode::Call),
            22 => Some(Opcode::Return),
            23 => Some(Opcode::GetGlobal),
            24 => Some(Opcode::SetGlobal),
            25 => Some(Opcode::GetLocal),
            26 => Some(Opcode::SetLocal),
            27 => Some(Opcode::Array),
            28 => Some(Opcode::Hash),
            29 => Some(Opcode::Index),
            30 => Some(Opcode::Class),
            31 => Some(Opcode::GetProperty),
            32 => Some(Opcode::SetProperty),
            33 => Some(Opcode::New),
            34 => Some(Opcode::This),
            35 => Some(Opcode::Super),
            36 => Some(Opcode::Pop),
            37 => Some(Opcode::True),
            38 => Some(Opcode::False),
            39 => Some(Opcode::Null),
            40 => Some(Opcode::NoOp),
            41 => Some(Opcode::MapToAst),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_byte() {
        // Тест преобразования байта в opcode
        assert_eq!(Opcode::from_byte(1), Some(Opcode::Constant));
        assert_eq!(Opcode::from_byte(2), Some(Opcode::Add));
        assert_eq!(Opcode::from_byte(9), Some(Opcode::Not));
        assert_eq!(Opcode::from_byte(12), Some(Opcode::Equal));
        assert_eq!(Opcode::from_byte(18), Some(Opcode::Jump));
        assert_eq!(Opcode::from_byte(37), Some(Opcode::True));
        assert_eq!(Opcode::from_byte(41), Some(Opcode::MapToAst));
        assert_eq!(Opcode::from_byte(42), None); // Несуществующий опкод
        assert_eq!(Opcode::from_byte(0), None); // Несуществующий опкод
    }

    #[test]
    fn test_opcode_mnemonic() {
        // Тест получения мнемоники для опкодов (используют UPPER_CASE мнемоники)
        assert_eq!(Opcode::Constant.mnemonic(), "CONSTANT");
        assert_eq!(Opcode::Add.mnemonic(), "ADD");
        assert_eq!(Opcode::Sub.mnemonic(), "SUB");
        assert_eq!(Opcode::Mul.mnemonic(), "MUL");
        assert_eq!(Opcode::Div.mnemonic(), "DIV");
        assert_eq!(Opcode::Equal.mnemonic(), "EQUAL");
        assert_eq!(Opcode::Jump.mnemonic(), "JUMP");
        assert_eq!(Opcode::Return.mnemonic(), "RETURN");
        assert_eq!(Opcode::Null.mnemonic(), "NULL");
    }

    #[test]
    fn test_opcode_operand_widths() {
        // Тест получения ширины операндов
        // Опкоды с 2-байтовыми операндами
        assert_eq!(Opcode::Constant.operand_widths(), vec![2]);
        assert_eq!(Opcode::Jump.operand_widths(), vec![2]);
        assert_eq!(Opcode::JumpIfFalse.operand_widths(), vec![2]);
        assert_eq!(Opcode::GetGlobal.operand_widths(), vec![2]);
        assert_eq!(Opcode::SetGlobal.operand_widths(), vec![2]);
        assert_eq!(Opcode::Array.operand_widths(), vec![2]);
        assert_eq!(Opcode::Hash.operand_widths(), vec![2]);

        // Опкоды с 1-байтовыми операндами
        assert_eq!(Opcode::GetLocal.operand_widths(), vec![1]);
        assert_eq!(Opcode::SetLocal.operand_widths(), vec![1]);
        assert_eq!(Opcode::Call.operand_widths(), vec![1]);

        // Опкоды без операндов
        assert_eq!(Opcode::Add.operand_widths(), vec![]);
        assert_eq!(Opcode::Sub.operand_widths(), vec![]);
        assert_eq!(Opcode::Pop.operand_widths(), vec![]);
        assert_eq!(Opcode::True.operand_widths(), vec![]);
        assert_eq!(Opcode::False.operand_widths(), vec![]);
        assert_eq!(Opcode::Null.operand_widths(), vec![]);
    }

    #[test]
    fn test_all_opcodes_roundtrip() {
        // Проверяем что все опкоды от 1 до 41 правильно преобразуются туда-сюда
        for byte in 1..=41 {
            let opcode = Opcode::from_byte(byte);
            assert!(opcode.is_some(), "Opcode with byte {} should exist", byte);
            let opcode = opcode.unwrap();
            assert_eq!(opcode as u8, byte, "Opcode byte should be {}", byte);
        }
    }

    #[test]
    fn test_opcode_equality() {
        // Тест на равенство опкодов
        assert_eq!(Opcode::Add, Opcode::Add);
        assert_ne!(Opcode::Add, Opcode::Sub);
        assert_ne!(Opcode::Equal, Opcode::NotEqual);
    }
}
