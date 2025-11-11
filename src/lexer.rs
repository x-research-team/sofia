use crate::token::{Token, TokenType};

// Лексер для языка SOFIA
pub struct Lexer {
    input: Vec<char>,
    position: usize,      // текущая позиция (указывает на текущий символ)
    read_position: usize, // следующая позиция для чтения (после текущей)
    ch: char,             // текущий символ
}

impl Lexer {
    // Создает новый лексер
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    // Считывает следующий символ и сдвигает позиции
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0'; // Нулевой символ как признак конца ввода
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    // "Подглядывает" следующий символ, не сдвигая позиций
    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    // Основной метод, возвращающий следующий токен
    pub fn next_token(&mut self) -> Token {
        // Циклически пропускаем пробелы и комментарии
        loop {
            self.skip_whitespace();
            if !self.is_comment_start() {
                break;
            }
            self.skip_comments();
        }

        let token = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::new(TokenType::Eq, "==".to_string())
                } else if self.peek_char() == '>' {
                    self.read_char();
                    Token::new(TokenType::Arrow, "=>".to_string())
                } else {
                    Token::new(TokenType::Assign, "=".to_string())
                }
            }
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::new(TokenType::NotEq, "!=".to_string())
                } else {
                    Token::new(TokenType::Bang, "!".to_string())
                }
            }
            '+' => Token::new(TokenType::Plus, "+".to_string()),
            '-' => Token::new(TokenType::Minus, "-".to_string()),
            '/' => Token::new(TokenType::Slash, "/".to_string()),
            '*' => {
                if self.peek_char() == '*' {
                    self.read_char();
                    Token::new(TokenType::Power, "**".to_string())
                } else {
                    Token::new(TokenType::Asterisk, "*".to_string())
                }
            }
            '<' => Token::new(TokenType::Lt, "<".to_string()),
            '>' => Token::new(TokenType::Gt, ">".to_string()),
            '&' => {
                if self.peek_char() == '&' {
                    self.read_char();
                    Token::new(TokenType::And, "&&".to_string())
                } else {
                    Token::new(TokenType::Illegal, "&".to_string())
                }
            }
            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    Token::new(TokenType::Or, "||".to_string())
                } else {
                    Token::new(TokenType::Illegal, "|".to_string())
                }
            }
            '%' => Token::new(TokenType::Modulo, "%".to_string()),
            ';' => Token::new(TokenType::Semicolon, ";".to_string()),
            ',' => Token::new(TokenType::Comma, ",".to_string()),
            '.' => {
                if self.peek_char() == '.' {
                    self.read_char();
                    Token::new(TokenType::Range, "..".to_string())
                } else {
                    Token::new(TokenType::Dot, ".".to_string())
                }
            }
            '(' => Token::new(TokenType::LParen, "(".to_string()),
            ')' => Token::new(TokenType::RParen, ")".to_string()),
            '{' => Token::new(TokenType::LBrace, "{".to_string()),
            '}' => Token::new(TokenType::RBrace, "}".to_string()),
            '[' => Token::new(TokenType::LBracket, "[".to_string()),
            ']' => Token::new(TokenType::RBracket, "]".to_string()),
            '"' => self.read_string(),
            '\0' => Token::new(TokenType::Eof, "".to_string()),
            _ => {
                if self.is_letter() {
                    let literal = self.read_identifier();
                    let token_type = Self::lookup_ident(&literal);
                    return Token::new(token_type, literal);
                }
                if self.is_digit() {
                    let literal = self.read_number();
                    return Token::new(TokenType::Int, literal);
                }
                Token::new(TokenType::Illegal, self.ch.to_string())
            }
        };

        self.read_char();
        token
    }

    // Пропускает все пробельные символы
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    // Пропускает однострочные комментарии (//)
    fn skip_comments(&mut self) {
        if self.ch == '/' && self.peek_char() == '/' {
            while self.ch != '\n' && self.ch != '\0' {
                self.read_char();
            }
        }
    }

    // Проверяет, начинается ли комментарий в текущей позиции
    fn is_comment_start(&self) -> bool {
        self.ch == '/' && self.peek_char() == '/'
    }

    // Считывает идентификатор (или ключевое слово)
    fn read_identifier(&mut self) -> String {
        let start_pos = self.position;
        // Первый символ уже проверен как буква в is_letter()
        // Теперь читаем буквы, цифры и подчеркивания
        while self.is_letter() || self.is_digit() {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    // Считывает число
    fn read_number(&mut self) -> String {
        let start_pos = self.position;
        while self.is_digit() {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    // Считывает строку в кавычках
    fn read_string(&mut self) -> Token {
        let start_pos = self.position + 1;
        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\'' || self.ch == '`' || self.ch == '\0' {
                break;
            }
        }
        let literal: String = self.input[start_pos..self.position].iter().collect();
        Token::new(TokenType::String, literal)
    }

    // Проверяет, является ли символ буквой (или '_')
    fn is_letter(&self) -> bool {
        self.ch.is_alphabetic() || self.ch == '_'
    }

    // Проверяет, является ли символ цифрой
    fn is_digit(&self) -> bool {
        self.ch.is_ascii_digit()
    }

    // Определяет, является ли идентификатор ключевым словом
    fn lookup_ident(ident: &str) -> TokenType {
        match ident {
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "extends" => TokenType::Extends,
            "false" => TokenType::False,
            "fn" => TokenType::Function,
            "if" => TokenType::If,
            "implements" => TokenType::Implements,
            "interface" => TokenType::Interface,
            "let" => TokenType::Let,
            "new" => TokenType::New,
            "private" => TokenType::Private,
            "public" => TokenType::Public,
            "return" => TokenType::Return,
            "static" => TokenType::Static,
            "struct" => TokenType::Struct,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "match" => TokenType::Match,
            _ => TokenType::Ident,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_next_token() {
        let input = r#"
             let five = 5;
             let ten = 10;
 
             let add = fn(x, y) {
                 x + y;
             };
 
             let result = add(five, ten);
             !-/*5;
             5 < 10 > 5;
 
             if (5 < 10) {
                 return true;
             } else {
                 return false;
             }
 
             10 == 10;
             10 != 9;
             "foobar"
             "foo bar"
             2 ** 3;
             true && false;
             true || false;
             10 % 3;
 
             class.new();
             iface.implements();
             obj.public;
             obj.private;
             MyClass.static;
             this.super;
             match (1..5) {
                1 => true,
                _ => false,
             };
             "#;

        let tests = vec![
            (TokenType::Let, "let"),
            (TokenType::Ident, "five"),
            (TokenType::Assign, "="),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "ten"),
            (TokenType::Assign, "="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "add"),
            (TokenType::Assign, "="),
            (TokenType::Function, "fn"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "x"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "y"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Ident, "x"),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "y"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Semicolon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "result"),
            (TokenType::Assign, "="),
            (TokenType::Ident, "add"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "five"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "ten"),
            (TokenType::RParen, ")"),
            (TokenType::Semicolon, ";"),
            (TokenType::Bang, "!"),
            (TokenType::Minus, "-"),
            (TokenType::Slash, "/"),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Gt, ">"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),
            (TokenType::If, "if"),
            (TokenType::LParen, "("),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::True, "true"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Else, "else"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Int, "10"),
            (TokenType::Eq, "=="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "10"),
            (TokenType::NotEq, "!="),
            (TokenType::Int, "9"),
            (TokenType::Semicolon, ";"),
            (TokenType::String, "foobar"),
            (TokenType::String, "foo bar"),
            (TokenType::Int, "2"),
            (TokenType::Power, "**"),
            (TokenType::Int, "3"),
            (TokenType::Semicolon, ";"),
            (TokenType::True, "true"),
            (TokenType::And, "&&"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::True, "true"),
            (TokenType::Or, "||"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::Int, "10"),
            (TokenType::Modulo, "%"),
            (TokenType::Int, "3"),
            (TokenType::Semicolon, ";"),
            (TokenType::Class, "class"),
            (TokenType::Dot, "."),
            (TokenType::New, "new"),
            (TokenType::LParen, "("),
            (TokenType::RParen, ")"),
            (TokenType::Semicolon, ";"),
            (TokenType::Ident, "iface"),
            (TokenType::Dot, "."),
            (TokenType::Implements, "implements"),
            (TokenType::LParen, "("),
            (TokenType::RParen, ")"),
            (TokenType::Semicolon, ";"),
            (TokenType::Ident, "obj"),
            (TokenType::Dot, "."),
            (TokenType::Public, "public"),
            (TokenType::Semicolon, ";"),
            (TokenType::Ident, "obj"),
            (TokenType::Dot, "."),
            (TokenType::Private, "private"),
            (TokenType::Semicolon, ";"),
            (TokenType::Ident, "MyClass"),
            (TokenType::Dot, "."),
            (TokenType::Static, "static"),
            (TokenType::Semicolon, ";"),
            (TokenType::This, "this"),
            (TokenType::Dot, "."),
            (TokenType::Super, "super"),
            (TokenType::Semicolon, ";"),
            (TokenType::Match, "match"),
            (TokenType::LParen, "("),
            (TokenType::Int, "1"),
            (TokenType::Range, ".."),
            (TokenType::Int, "5"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Int, "1"),
            (TokenType::Arrow, "=>"),
            (TokenType::True, "true"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "_"),
            (TokenType::Arrow, "=>"),
            (TokenType::False, "false"),
            (TokenType::Comma, ","),
            (TokenType::RBrace, "}"),
            (TokenType::Semicolon, ";"),
            (TokenType::Eof, ""),
        ];

        let mut lexer = Lexer::new(input.to_string());

        for (i, (expected_type, expected_literal)) in tests.iter().enumerate() {
            let tok = lexer.next_token();
            assert_eq!(
                tok.token_type, *expected_type,
                "tests[{}] - тип токена неверный. ожидался={:?}, получен={:?}",
                i, expected_type, tok.token_type
            );
            assert_eq!(
                tok.literal, *expected_literal,
                "tests[{}] - литерал неверный. ожидался={}, получен={}",
                i, expected_literal, tok.literal
            );
        }
    }
}
