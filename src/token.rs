// Типы токенов, которые распознает лексер
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum TokenType {
    // Нераспознанный токен
    Illegal,
    // Конец файла
    Eof,

    // Идентификаторы и литералы
    Ident,
    Int,
    String,

    // Операторы
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Eq,
    NotEq,
    Power,
    And,
    Or,
    Modulo,

    // Разделители
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    Colon,      // :
    Underscore, // _

    // Ключевые слова
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,

    // Ключевые слова для сопоставления с образцом
    Match,

    // Операторы для сопоставления с образцом
    Arrow, // =>
    Range, // ..

    // ООП ключевые слова
    Class,
    Interface,
    Struct,
    This,
    Super,
    New,
    Extends,
    Implements,
    Public,
    Private,
    Static,
}

// Структура, представляющая лексическую единицу (токен)
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

// Реализация методов структуры Token
impl Token {
    // Создает новый токен
    pub fn new(token_type: TokenType, literal: String) -> Self {
        Token {
            token_type,
            literal,
        }
    }
}
