use crate::ast::{self, AccessModifier, Program};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

// Определение приоритетов операторов
#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Arrow,       // => (самый низкий приоритет для разделения паттерна и выражения)
    Or,          // ||
    And,         // &&
    Equals,      // ==, !=
    LessGreater, // >, <
    Sum,         // +, -
    Product,     // *, /, %
    Power,       // **
    Range,       // .. , ..= (выше Sum/Product, ниже Prefix)
    Prefix,      // -X или !X
    Call,        // myFunction(X)
    Dot,         // object.member
}

// Ошибки, которые могут возникнуть во время парсинга
#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(String),
}

// Парсер
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    next_token: Token,
    errors: Vec<ParserError>,
}

impl Parser {
    // Создает новый парсер
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::new(TokenType::Illegal, "".to_string()),
            next_token: Token::new(TokenType::Illegal, "".to_string()),
            errors: Vec::new(),
        };

        // Инициализация current_token и next_token
        parser.next_token();
        parser.next_token();

        parser
    }

    // Сдвигает токены
    fn next_token(&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = self.lexer.next_token();
    }

    // Основной метод парсинга программы
    pub fn parse_program(&mut self) -> Result<Program, Vec<ParserError>> {
        let mut program = Program::new();

        while self.current_token.token_type != TokenType::Eof {
            match self.parse_statement() {
                Ok(statement) => program.statements.push(statement),
                Err(e) => self.errors.push(e),
            }
            self.next_token();
        }

        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // Парсинг оператора
    fn parse_statement(&mut self) -> Result<ast::Statement, ParserError> {
        match self.current_token.token_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Class => self.parse_class_declaration(),
            TokenType::Struct => self.parse_struct_declaration(),
            TokenType::Interface => self.parse_interface_declaration(),
            TokenType::Match => self.parse_expression_statement(), // Match - это выражение, поэтому парсим как expression statement
            _ => self.parse_expression_statement(),
        }
    }

    // Парсинг оператора let
    fn parse_let_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let let_token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected next token to be IDENT, got {:?} instead",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected next token to be =, got {:?} instead",
                self.next_token.token_type
            )));
        }

        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::Let(ast::LetStatement {
            token: let_token,
            name,
            value,
        }))
    }

    // Парсинг оператора return
    fn parse_return_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let return_token = self.current_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::Return(ast::ReturnStatement {
            token: return_token,
            return_value,
        }))
    }

    // Парсинг оператора-выражения
    fn parse_expression_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let expression = self.parse_expression(Precedence::Lowest)?;

        let stmt = ast::ExpressionStatement {
            token: self.current_token.clone(),
            expression,
        };

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::Expression(stmt))
    }

    // Парсинг выражения (Pratt parser)
    fn parse_expression(&mut self, precedence: Precedence) -> Result<ast::Expression, ParserError> {
        eprintln!(
            "DEBUG: parse_expression: current_token={:?}, next_token={:?}",
            self.current_token, self.next_token
        );
        let mut left_exp = self.parse_prefix()?;

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            match self.next_token.token_type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Asterisk
                | TokenType::Eq
                | TokenType::NotEq
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::Power
                | TokenType::And
                | TokenType::Or
                | TokenType::Modulo
                | TokenType::LParen
                | TokenType::Dot => {
                    self.next_token();
                    if self.current_token.token_type == TokenType::LParen {
                        left_exp = self.parse_call_expression(left_exp)?;
                    } else {
                        left_exp = self.parse_infix(left_exp)?;
                    }
                }
                _ => return Ok(left_exp),
            }
        }

        Ok(left_exp)
    }

    // Парсинг префиксного выражения
    fn parse_prefix(&mut self) -> Result<ast::Expression, ParserError> {
        match self.current_token.token_type {
            TokenType::Ident => Ok(ast::Expression::Identifier(ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            })),
            TokenType::Int => self.parse_integer_literal(),
            TokenType::String => self.parse_string_literal(),
            TokenType::Bang | TokenType::Minus => self.parse_prefix_expression(),
            TokenType::True | TokenType::False => self.parse_boolean(),
            TokenType::LParen => self.parse_grouped_expression(),
            TokenType::LBracket => self.parse_array_literal(),
            TokenType::If => self.parse_if_expression(),
            TokenType::Function => self.parse_function_literal(),
            TokenType::New => self.parse_new_expression(),
            TokenType::This => self.parse_this_expression(),
            TokenType::Super => self.parse_super_expression(),
            TokenType::Match => self.parse_match_expression(),
            _ => Err(ParserError::UnexpectedToken(format!(
                "no prefix parse function for {:?} found",
                self.current_token.token_type
            ))),
        }
    }

    // Парсинг инфиксного выражения
    fn parse_infix(&mut self, left: ast::Expression) -> Result<ast::Expression, ParserError> {
        match self.current_token.token_type {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Slash
            | TokenType::Asterisk
            | TokenType::Eq
            | TokenType::NotEq
            | TokenType::Lt
            | TokenType::Gt
            | TokenType::Power
            | TokenType::And
            | TokenType::Or
            | TokenType::Modulo
            | TokenType::Assign => self.parse_infix_expression(left),
            TokenType::LParen => self.parse_call_expression(left),
            TokenType::Dot => self.parse_property_access_expression(left),
            _ => Err(ParserError::UnexpectedToken(format!(
                "no infix parse function for {:?} found",
                self.current_token.token_type
            ))),
        }
    }

    // Парсинг целочисленного литерала
    fn parse_integer_literal(&mut self) -> Result<ast::Expression, ParserError> {
        let value = self.current_token.literal.parse::<i64>().map_err(|_| {
            ParserError::UnexpectedToken(format!(
                "could not parse {} as integer",
                self.current_token.literal
            ))
        })?;

        Ok(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
            token: self.current_token.clone(),
            value,
        }))
    }

    fn parse_string_literal(&mut self) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::StringLiteral(ast::StringLiteral {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        }))
    }

    // Парсинг префиксного выражения
    fn parse_prefix_expression(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();
        let operator = self.current_token.literal.clone();

        self.next_token();

        let right = self.parse_expression(Precedence::Prefix)?;

        Ok(ast::Expression::Prefix(ast::PrefixExpression {
            token,
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_infix_expression(
        &mut self,
        left: ast::Expression,
    ) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();
        let operator = self.current_token.literal.clone();
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Ok(ast::Expression::Infix(ast::InfixExpression {
            token,
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }))
    }

    fn parse_boolean(&mut self) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::Boolean(ast::BooleanLiteral {
            token: self.current_token.clone(),
            value: self.current_token_is(TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Result<ast::Expression, ParserError> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenType::RParen) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected next token to be ), got {:?} instead",
                self.next_token.token_type
            )));
        }
        Ok(exp)
    }

    fn parse_array_literal(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();
        let elements = self.parse_expression_list(TokenType::RBracket)?;
        // После parse_expression_list(), current_token указывает на RBracket
        // Нам нужно оставить current_token на последнем элементе или на скобке?
        // Посмотрим на то, как это работает в parse_expression()

        Ok(ast::Expression::ArrayLiteral(ast::ArrayLiteral {
            token,
            elements,
        }))
    }

    fn parse_block_statement(&mut self) -> Result<ast::BlockStatement, ParserError> {
        let token = self.current_token.clone();
        let mut statements = Vec::new();
        self.next_token();

        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.next_token();
        }

        Ok(ast::BlockStatement { token, statements })
    }

    fn parse_if_expression(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(
                "expected '(' after 'if'".to_string(),
            ));
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenType::RParen) {
            return Err(ParserError::UnexpectedToken(
                "expected ')' after condition".to_string(),
            ));
        }

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(
                "expected '{{' after ')'".to_string(),
            ));
        }

        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token_is(TokenType::Else) {
            self.next_token();
            if !self.expect_peek(TokenType::LBrace) {
                return Err(ParserError::UnexpectedToken(
                    "expected '{{' after 'else'".to_string(),
                ));
            }
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(ast::Expression::If(ast::IfExpression {
            token,
            condition: Box::new(condition),
            consequence,
            alternative,
        }))
    }

    fn parse_function_literal(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(
                "expected '(' after 'fn'".to_string(),
            ));
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(
                "expected '{{' after parameters".to_string(),
            ));
        }

        let body = self.parse_block_statement()?;

        Ok(ast::Expression::FunctionLiteral(ast::FunctionLiteral {
            token,
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<ast::Identifier>, ParserError> {
        let mut identifiers = Vec::new();

        if self.peek_token_is(TokenType::RParen) {
            self.next_token();
            return Ok(identifiers);
        }

        self.next_token();

        let ident = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };
        identifiers.push(ident);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            let ident = ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            };
            identifiers.push(ident);
        }

        if !self.expect_peek(TokenType::RParen) {
            return Err(ParserError::UnexpectedToken(
                "expected ')' after parameters".to_string(),
            ));
        }

        Ok(identifiers)
    }

    fn parse_call_expression(
        &mut self,
        function: ast::Expression,
    ) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();
        let arguments = self.parse_expression_list(TokenType::RParen)?;
        Ok(ast::Expression::Call(ast::CallExpression {
            token,
            function: Box::new(function),
            arguments,
        }))
    }

    fn parse_expression_list(
        &mut self,
        end: TokenType,
    ) -> Result<Vec<ast::Expression>, ParserError> {
        let mut list = Vec::new();

        if self.peek_token_is(end) {
            self.next_token();
            return Ok(list);
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(end) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected next token to be {:?}, got {:?} instead",
                end, self.next_token.token_type
            )));
        }

        Ok(list)
    }

    fn parse_property_access_expression(
        &mut self,
        left: ast::Expression,
    ) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after '.', got {:?}",
                self.next_token.token_type
            )));
        }

        let property = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        Ok(ast::Expression::PropertyAccess(
            ast::PropertyAccessExpression {
                token,
                left: Box::new(left),
                property,
            },
        ))
    }

    fn parse_new_expression(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after 'new', got {:?}",
                self.next_token.token_type
            )));
        }

        let class_name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '(' after class name in new expression, got {:?}",
                self.next_token.token_type
            )));
        }

        let arguments = self.parse_expression_list(TokenType::RParen)?;

        Ok(ast::Expression::New(ast::NewExpression {
            token,
            class_name,
            arguments,
        }))
    }

    fn parse_this_expression(&mut self) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::This(ast::ThisExpression {
            token: self.current_token.clone(),
        }))
    }

    fn parse_super_expression(&mut self) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::Super(ast::SuperExpression {
            token: self.current_token.clone(),
        }))
    }

    // Парсит match выражение.
    fn parse_match_expression(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.current_token.clone(); // Токен 'match'

        // Ожидаем выражение, которое будет сопоставляться
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;

        // Ожидаем открывающую фигурную скобку '{'
        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after match value, got {:?}",
                self.next_token.token_type
            )));
        }

        let mut arms = Vec::new();
        self.next_token(); // Пропускаем '{'

        // Парсим ветви match
        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            arms.push(self.parse_match_arm()?);

            // После parse_match_arm(), current_token указывает на запятую или '}'
            if self.current_token_is(TokenType::Comma) {
                // Пропускаем запятую, переходим к следующей ветви
                self.next_token();
            } else if !self.current_token_is(TokenType::RBrace) {
                // Если это не запятая и не '}', ошибка
                return Err(ParserError::UnexpectedToken(format!(
                    "expected ',' or '}}' after match arm, got {:?}",
                    self.current_token.token_type
                )));
            }
        }

        Ok(ast::Expression::Match(ast::MatchExpression {
            token,
            value: Box::new(value),
            arms,
        }))
    }

    fn parse_class_declaration(&mut self) -> Result<ast::Statement, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after class, got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        let super_class = if self.peek_token_is(TokenType::Extends) {
            self.next_token(); // consume 'extends'
            if !self.expect_peek(TokenType::Ident) {
                return Err(ParserError::UnexpectedToken(format!(
                    "expected superclass name after 'extends', got {:?}",
                    self.next_token.token_type
                )));
            }
            Some(ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            })
        } else {
            None
        };

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after class name, got {:?}",
                self.next_token.token_type
            )));
        }

        let mut properties = Vec::new();
        let mut methods = Vec::new();

        self.next_token(); // Пропускаем LBrace

        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            let (access_modifier, is_static) = self.parse_access_modifier_and_static();

            if self.current_token_is(TokenType::Let) {
                let prop = self.parse_property_declaration(access_modifier, is_static)?;
                properties.push(prop);
            } else if self.current_token_is(TokenType::Function) {
                let method = self.parse_method_declaration(access_modifier, is_static)?;
                methods.push(method);
            } else if self.current_token_is(TokenType::Ident) {
                // Синтаксис: public x = 10; или public getName() { }
                // Проверяем, это свойство или метод, смотря на следующий токен
                if self.peek_token_is(TokenType::LParen) {
                    // Это метод без fn: public getName() { }
                    let method =
                        self.parse_method_declaration_without_fn(access_modifier, is_static)?;
                    methods.push(method);
                } else if self.peek_token_is(TokenType::Assign)
                    || self.peek_token_is(TokenType::Semicolon)
                {
                    // Это свойство без let: public x = 10; или public x;
                    let prop =
                        self.parse_property_declaration_without_let(access_modifier, is_static)?;
                    properties.push(prop);
                } else {
                    return Err(ParserError::UnexpectedToken(format!(
                        "expected '(' or '=' or ';' after identifier in class body, got {:?}",
                        self.next_token.token_type
                    )));
                }
            } else {
                return Err(ParserError::UnexpectedToken(format!(
                    "expected 'let', 'fn', or identifier in class body, got {:?}",
                    self.current_token.token_type
                )));
            }
        }

        if !self.current_token_is(TokenType::RBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '}}' to close class, got {:?}",
                self.current_token.token_type
            )));
        }

        // Потребляем точку с запятой, если она присутствует
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::ClassDeclaration(ast::ClassDeclaration {
            token,
            name,
            super_class,
            interfaces: Vec::new(), // TODO: Реализовать парсинг `implements`
            properties,
            methods,
        }))
    }

    fn parse_struct_declaration(&mut self) -> Result<ast::Statement, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after struct, got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after struct name, got {:?}",
                self.next_token.token_type
            )));
        }

        let mut properties = Vec::new();
        self.next_token(); // Пропускаем LBrace

        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            let (access_modifier, is_static) = self.parse_access_modifier_and_static();
            if self.current_token_is(TokenType::Let) {
                let prop = self.parse_property_declaration(access_modifier, is_static)?;
                properties.push(prop);
            } else {
                return Err(ParserError::UnexpectedToken(format!(
                    "expected 'let' in struct body, got {:?}",
                    self.current_token.token_type
                )));
            }
        }

        if !self.current_token_is(TokenType::RBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '}}' to close struct, got {:?}",
                self.current_token.token_type
            )));
        }

        // Потребляем точку с запятой, если она присутствует
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::StructDeclaration(ast::StructDeclaration {
            token,
            name,
            properties,
        }))
    }

    fn parse_interface_declaration(&mut self) -> Result<ast::Statement, ParserError> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after interface, got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after interface name, got {:?}",
                self.next_token.token_type
            )));
        }

        let mut method_signatures = Vec::new();
        self.next_token(); // Пропускаем LBrace

        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            let signature = self.parse_method_signature_declaration()?;
            method_signatures.push(signature);
        }

        if !self.current_token_is(TokenType::RBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '}}' to close interface, got {:?}",
                self.current_token.token_type
            )));
        }

        // Потребляем точку с запятой, если она присутствует
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Ok(ast::Statement::InterfaceDeclaration(
            ast::InterfaceDeclaration {
                token,
                name,
                method_signatures,
            },
        ))
    }

    // Парсит модификаторы доступа и static.
    fn parse_access_modifier_and_static(&mut self) -> (AccessModifier, bool) {
        let mut access_modifier = AccessModifier::Private; // По умолчанию private
        let mut is_static = false;

        // Модификаторы могут идти в любом порядке, но только один раз
        loop {
            if self.current_token_is(TokenType::Public) {
                access_modifier = AccessModifier::Public;
                self.next_token();
            } else if self.current_token_is(TokenType::Private) {
                access_modifier = AccessModifier::Private;
                self.next_token();
            } else if self.current_token_is(TokenType::Static) {
                is_static = true;
                self.next_token();
            } else {
                break;
            }
        }
        (access_modifier, is_static)
    }

    // Парсит объявление свойства.
    fn parse_property_declaration(
        &mut self,
        access_modifier: AccessModifier,
        is_static: bool,
    ) -> Result<ast::PropertyDeclaration, ParserError> {
        let token = self.current_token.clone(); // `let` token

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after 'let', got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        let value = if self.peek_token_is(TokenType::Assign) {
            self.next_token();
            self.next_token();
            Some(self.parse_expression(Precedence::Lowest)?)
        } else {
            None
        };

        if !self.expect_peek(TokenType::Semicolon) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected ';' after property declaration, got {:?}",
                self.next_token.token_type
            )));
        }
        self.next_token(); // Пропускаем точку с запятой для следующей итерации

        Ok(ast::PropertyDeclaration {
            token,
            name,
            value,
            access_modifier,
            is_static,
        })
    }

    // Парсит объявление метода.
    fn parse_method_declaration(
        &mut self,
        access_modifier: AccessModifier,
        is_static: bool,
    ) -> Result<ast::MethodDeclaration, ParserError> {
        let token = self.current_token.clone(); // `fn` token

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after 'fn', got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '(' after method name, got {:?}",
                self.next_token.token_type
            )));
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after method parameters, got {:?}",
                self.next_token.token_type
            )));
        }

        let body = self.parse_block_statement()?;
        self.next_token(); // Пропускаем '}'

        Ok(ast::MethodDeclaration {
            token,
            name,
            parameters,
            body,
            access_modifier,
            is_static,
        })
    }

    // Парсит объявление метода без ключевого слова fn: public getName() { }
    fn parse_method_declaration_without_fn(
        &mut self,
        access_modifier: AccessModifier,
        is_static: bool,
    ) -> Result<ast::MethodDeclaration, ParserError> {
        let token = self.current_token.clone(); // Первый токен идентификатора

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '(' after method name, got {:?}",
                self.next_token.token_type
            )));
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after method parameters, got {:?}",
                self.next_token.token_type
            )));
        }

        let body = self.parse_block_statement()?;
        self.next_token(); // Пропускаем '}'

        Ok(ast::MethodDeclaration {
            token,
            name,
            parameters,
            body,
            access_modifier,
            is_static,
        })
    }

    // Парсит объявление свойства без ключевого слова let: public x = 10; или public x;
    fn parse_property_declaration_without_let(
        &mut self,
        access_modifier: AccessModifier,
        is_static: bool,
    ) -> Result<ast::PropertyDeclaration, ParserError> {
        let token = self.current_token.clone(); // Идентификатор

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        let value = if self.peek_token_is(TokenType::Assign) {
            self.next_token(); // Переходим на '='
            self.next_token(); // Переходим на первый токен выражения
            Some(self.parse_expression(Precedence::Lowest)?)
        } else {
            None
        };

        if !self.expect_peek(TokenType::Semicolon) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected ';' after property declaration, got {:?}",
                self.next_token.token_type
            )));
        }
        self.next_token(); // Пропускаем точку с запятой для следующей итерации

        Ok(ast::PropertyDeclaration {
            token,
            name,
            value,
            access_modifier,
            is_static,
        })
    }

    // Парсит сигнатуру метода в интерфейсе.
    fn parse_method_signature_declaration(
        &mut self,
    ) -> Result<ast::MethodSignatureDeclaration, ParserError> {
        if !self.current_token_is(TokenType::Function) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected 'fn' for method signature, got {:?}",
                self.current_token.token_type
            )));
        }
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::Ident) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected identifier after 'fn', got {:?}",
                self.next_token.token_type
            )));
        }

        let name = ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::LParen) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '(' after method name, got {:?}",
                self.next_token.token_type
            )));
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenType::Semicolon) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected ';' after method signature, got {:?}",
                self.next_token.token_type
            )));
        }
        self.next_token(); // Пропускаем ';'

        Ok(ast::MethodSignatureDeclaration {
            token,
            name,
            parameters,
        })
    }

    // Вспомогательные функции
    fn peek_token_is(&self, t: TokenType) -> bool {
        self.next_token.token_type == t
    }

    fn current_token_is(&self, t: TokenType) -> bool {
        self.current_token.token_type == t
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.next_token.token_type
        );
        self.errors.push(ParserError::UnexpectedToken(msg));
    }

    fn get_precedence(token_type: &TokenType) -> Precedence {
        match token_type {
            TokenType::Eq | TokenType::NotEq => Precedence::Equals,
            TokenType::Lt | TokenType::Gt => Precedence::LessGreater,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Slash | TokenType::Asterisk | TokenType::Modulo => Precedence::Product,
            TokenType::Power => Precedence::Power,
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            TokenType::Assign => Precedence::Lowest,
            TokenType::LParen => Precedence::Call,
            TokenType::Dot => Precedence::Dot,
            TokenType::Range => Precedence::Range,
            TokenType::Arrow => Precedence::Arrow,
            _ => Precedence::Lowest,
        }
    }

    fn peek_precedence(&self) -> Precedence {
        Self::get_precedence(&self.next_token.token_type)
    }

    fn current_precedence(&self) -> Precedence {
        Self::get_precedence(&self.current_token.token_type)
    }

    // Парсит паттерн для match выражения.
    fn parse_pattern(&mut self) -> Result<ast::Pattern, ParserError> {
        match self.current_token.token_type {
            TokenType::Int | TokenType::String | TokenType::True | TokenType::False => {
                // Литеральные паттерны
                let expr = self.parse_prefix()?;
                // После parse_prefix(), current_token указывает на последний токен выражения
                // Проверяем, является ли это диапазонным паттерном
                if self.peek_token_is(TokenType::Range) {
                    self.next_token(); // Переместиться на '..'
                    return self.parse_range_pattern(expr);
                }
                Ok(ast::Pattern::Literal(expr))
            }
            TokenType::Ident => {
                // Проверяем, является ли это wildcard паттерном или идентификатором
                if self.current_token.literal == "_" {
                    Ok(ast::Pattern::Wildcard)
                } else {
                    // Это идентификаторный паттерн (переменная)
                    let ident_value = self.current_token.literal.clone();
                    let ident = ast::Identifier {
                        token: self.current_token.clone(),
                        value: ident_value,
                    };
                    Ok(ast::Pattern::Identifier(ident))
                }
            }
            TokenType::LBrace | TokenType::LBracket => {
                // Кортежный паттерн {a, b, c} или [a, b, c]
                let closing_bracket = if self.current_token_is(TokenType::LBrace) {
                    TokenType::RBrace
                } else {
                    TokenType::RBracket
                };

                let mut patterns = vec![];
                self.next_token(); // Переместиться внутрь скобок

                while !self.current_token_is(closing_bracket)
                    && !self.current_token_is(TokenType::Eof)
                {
                    patterns.push(self.parse_pattern()?);
                    // После parse_pattern(), current_token указывает на последний токен паттерна

                    self.next_token(); // Переместиться на запятую или закрывающую скобку

                    if self.current_token_is(TokenType::Comma) {
                        self.next_token(); // Переместиться на следующий паттерн
                    } else if !self.current_token_is(closing_bracket) {
                        return Err(ParserError::UnexpectedToken(format!(
                            "expected ',' or '{}' in tuple pattern, got {:?}",
                            if closing_bracket == TokenType::RBrace {
                                "}"
                            } else {
                                "]"
                            },
                            self.current_token.token_type
                        )));
                    }
                }

                // Теперь current_token должен быть закрывающей скобкой
                if !self.current_token_is(closing_bracket) {
                    return Err(ParserError::UnexpectedToken(format!(
                        "expected '{}' to close tuple pattern, got {:?}",
                        if closing_bracket == TokenType::RBrace {
                            "}"
                        } else {
                            "]"
                        },
                        self.current_token.token_type
                    )));
                }

                Ok(ast::Pattern::Tuple(patterns))
            }
            _ => Err(ParserError::UnexpectedToken(format!(
                "unexpected token in pattern: {:?}",
                self.current_token.token_type
            ))),
        }
    }

    fn parse_range_pattern(
        &mut self,
        start_expr: ast::Expression,
    ) -> Result<ast::Pattern, ParserError> {
        // current_token указывает на '..'
        let is_inclusive = if self.current_token_is(TokenType::Range) {
            false
        } else {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '..' in range pattern, got {:?}",
                self.current_token.token_type
            )));
        };

        self.next_token(); // Пропускаем '..'
        let end_expr = self.parse_prefix()?;
        // После parse_prefix(), current_token указывает на последний токен выражения
        // Не вызываем next_token здесь - это сделает вызывающая функция

        Ok(ast::Pattern::Range(ast::RangePattern {
            start: Box::new(start_expr),
            end: Box::new(end_expr),
            inclusive: is_inclusive,
        }))
    }

    #[allow(dead_code)]
    fn parse_struct_pattern(&mut self, name: ast::Identifier) -> Result<ast::Pattern, ParserError> {
        // current_token уже должен быть LBrace
        if !self.current_token_is(TokenType::LBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '{{' after struct name in pattern, got {:?}",
                self.current_token.token_type
            )));
        }
        self.next_token(); // Пропускаем '{'

        let mut fields = Vec::new();

        while !self.current_token_is(TokenType::RBrace) && !self.current_token_is(TokenType::Eof) {
            if !self.current_token_is(TokenType::Ident) {
                return Err(ParserError::UnexpectedToken(format!(
                    "expected identifier for struct field, got {:?}",
                    self.current_token.token_type
                )));
            }
            let field_name = ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            };
            self.next_token();

            let field_pattern = if self.current_token_is(TokenType::Colon) {
                self.next_token(); // Пропускаем ':'
                self.next_token(); // Переходим к паттерну значения
                Some(self.parse_pattern()?)
            } else {
                None
            };
            fields.push((field_name, field_pattern));

            if self.peek_token_is(TokenType::Comma) {
                self.next_token(); // Пропускаем ','
                self.next_token(); // Переходим к следующему полю
            } else if !self.peek_token_is(TokenType::RBrace) {
                return Err(ParserError::UnexpectedToken(format!(
                    "expected ',' or '}}' after struct field, got {:?}",
                    self.next_token.token_type
                )));
            }
        }

        if !self.expect_peek(TokenType::RBrace) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '}}' to close struct pattern, got {:?}",
                self.next_token.token_type
            )));
        }

        Ok(ast::Pattern::Struct(ast::StructPattern { name, fields }))
    }

    // Парсит одну ветвь match выражения.
    fn parse_match_arm(&mut self) -> Result<ast::MatchArm, ParserError> {
        let pattern = self.parse_pattern()?;
        // После parse_pattern(), current_token указывает на последний токен паттерна

        self.next_token(); // Переместиться на следующий токен (guard или '=>')

        let guard = if self.current_token_is(TokenType::If) {
            self.next_token(); // Пропускаем 'if'
            let guard_expr = self.parse_expression(Precedence::Lowest)?;
            // После parse_expression, current_token указывает на последний токен выражения
            self.next_token(); // Переместиться на следующий токен (=>)
            Some(guard_expr)
        } else {
            None
        };

        // Теперь current_token должен быть '=>'
        if !self.current_token_is(TokenType::Arrow) {
            return Err(ParserError::UnexpectedToken(format!(
                "expected '=>' after match pattern{}, got {:?}",
                if guard.is_some() { " and guard" } else { "" },
                self.current_token.token_type
            )));
        }
        self.next_token(); // Пропускаем '=>'

        // Тело ветви: парсим выражение до запятой или '}'
        let body_expr = self.parse_expression(Precedence::Lowest)?;

        // После parse_expression(), current_token указывает на последний токен выражения
        self.next_token(); // Перемещаемся на следующий токен (запятую или '}')

        // Создаем BlockStatement с одним ExpressionStatement
        let consequence = ast::BlockStatement {
            token: Token::new(TokenType::LBrace, "{".to_string()),
            statements: vec![ast::Statement::Expression(ast::ExpressionStatement {
                token: Token::new(TokenType::Ident, "expr".to_string()), // Временный токен
                expression: body_expr,
            })],
        };

        Ok(ast::MatchArm {
            pattern,
            guard,
            consequence,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{AccessModifier, Expression, Statement};
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_let_statements() {
        let input = "
            let x = 5;
            let y = 10;
            let foobar = 838383;
        ";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 3);

        let tests = vec!["x", "y", "foobar"];

        for (i, tt) in tests.iter().enumerate() {
            let stmt = &program.statements[i];
            if let Statement::Let(let_stmt) = stmt {
                assert_eq!(let_stmt.name.value, *tt);
                assert_eq!(let_stmt.name.token.literal, *tt);
            } else {
                panic!("stmt not a let statement");
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = "
            return 5;
            return 10;
            return 993322;
        ";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 3);

        for stmt in program.statements {
            if let Statement::Return(_) = stmt {
                // ok
            } else {
                panic!("stmt not a return statement");
            }
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::Identifier(ident) = &exp_stmt.expression {
                assert_eq!(ident.value, "foobar");
                assert_eq!(ident.token.literal, "foobar");
            } else {
                panic!("not an identifier");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::IntegerLiteral(literal) = &exp_stmt.expression {
                assert_eq!(literal.value, 5);
                assert_eq!(literal.token.literal, "5");
            } else {
                panic!("not an integer literal");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_boolean_literal_expression() {
        let input = "true; false;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 2);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::Boolean(boolean) = &exp_stmt.expression {
                assert_eq!(boolean.value, true);
                assert_eq!(boolean.token.literal, "true");
            } else {
                panic!("not a boolean");
            }
        } else {
            panic!("not an expression statement");
        }

        if let Statement::Expression(exp_stmt) = &program.statements[1] {
            if let Expression::Boolean(boolean) = &exp_stmt.expression {
                assert_eq!(boolean.value, false);
                assert_eq!(boolean.token.literal, "false");
            } else {
                panic!("not a boolean");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_string_literal_expression() {
        let input = "\"hello world\";";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let stmt = &program.statements[0];
        if let Statement::Expression(exp) = stmt {
            if let Expression::StringLiteral(string_lit) = &exp.expression {
                assert_eq!(string_lit.value, "hello world");
            } else {
                panic!("expression not a string literal");
            }
        } else {
            panic!("statement not an expression statement");
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let prefix_tests = vec![("!5;", "!", 5), ("-15;", "-", 15)];

        for tt in prefix_tests {
            let lexer = Lexer::new(tt.0.to_string());
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression(exp_stmt) = &program.statements[0] {
                if let Expression::Prefix(prefix_exp) = &exp_stmt.expression {
                    assert_eq!(prefix_exp.operator, tt.1);
                    if let Expression::IntegerLiteral(int_lit) = &*prefix_exp.right {
                        assert_eq!(int_lit.value, tt.2);
                    } else {
                        panic!("not an integer literal");
                    }
                } else {
                    panic!("not a prefix expression");
                }
            } else {
                panic!("not an expression statement");
            }
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let infix_tests = vec![
            ("5 + 5;", 5, "+", 5),
            ("5 - 5;", 5, "-", 5),
            ("5 * 5;", 5, "*", 5),
            ("5 / 5;", 5, "/", 5),
            ("5 > 5;", 5, ">", 5),
            ("5 < 5;", 5, "<", 5),
            ("5 == 5;", 5, "==", 5),
            ("5 != 5;", 5, "!=", 5),
        ];

        for tt in infix_tests {
            let lexer = Lexer::new(tt.0.to_string());
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression(exp_stmt) = &program.statements[0] {
                if let Expression::Infix(infix_exp) = &exp_stmt.expression {
                    if let Expression::IntegerLiteral(left) = &*infix_exp.left {
                        assert_eq!(left.value, tt.1);
                    } else {
                        panic!("left not integer literal");
                    }
                    assert_eq!(infix_exp.operator, tt.2);
                    if let Expression::IntegerLiteral(right) = &*infix_exp.right {
                        assert_eq!(right.value, tt.3);
                    } else {
                        panic!("right not integer literal");
                    }
                } else {
                    panic!("not an infix expression");
                }
            } else {
                panic!("not an expression statement");
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
            ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];

        for tt in tests {
            let lexer = Lexer::new(tt.0.to_string());
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            assert_eq!(program.to_string(), tt.1);
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::If(if_exp) = &exp_stmt.expression {
                if let Expression::Infix(infix) = &*if_exp.condition {
                    assert_eq!(infix.left.to_string(), "x");
                    assert_eq!(infix.operator, "<");
                    assert_eq!(infix.right.to_string(), "y");
                } else {
                    panic!("condition not infix expression");
                }
                assert_eq!(if_exp.consequence.statements.len(), 1);
                if let Statement::Expression(cons_stmt) = &if_exp.consequence.statements[0] {
                    assert_eq!(cons_stmt.expression.to_string(), "x");
                } else {
                    panic!("consequence not expression statement");
                }
                assert!(if_exp.alternative.is_none());
            } else {
                panic!("not an if expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::If(if_exp) = &exp_stmt.expression {
                if let Expression::Infix(infix) = &*if_exp.condition {
                    assert_eq!(infix.left.to_string(), "x");
                    assert_eq!(infix.operator, "<");
                    assert_eq!(infix.right.to_string(), "y");
                } else {
                    panic!("condition not infix expression");
                }
                assert_eq!(if_exp.consequence.statements.len(), 1);
                if let Statement::Expression(cons_stmt) = &if_exp.consequence.statements[0] {
                    assert_eq!(cons_stmt.expression.to_string(), "x");
                } else {
                    panic!("consequence not expression statement");
                }
                assert!(if_exp.alternative.is_some());
                if let Some(alt) = &if_exp.alternative {
                    assert_eq!(alt.statements.len(), 1);
                    if let Statement::Expression(alt_stmt) = &alt.statements[0] {
                        assert_eq!(alt_stmt.expression.to_string(), "y");
                    } else {
                        panic!("alternative not expression statement");
                    }
                }
            } else {
                panic!("not an if expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::FunctionLiteral(func) = &exp_stmt.expression {
                assert_eq!(func.parameters.len(), 2);
                assert_eq!(func.parameters[0].value, "x");
                assert_eq!(func.parameters[1].value, "y");
                assert_eq!(func.body.statements.len(), 1);
                if let Statement::Expression(body_stmt) = &func.body.statements[0] {
                    if let Expression::Infix(infix) = &body_stmt.expression {
                        assert_eq!(infix.left.to_string(), "x");
                        assert_eq!(infix.operator, "+");
                        assert_eq!(infix.right.to_string(), "y");
                    } else {
                        panic!("body not infix expression");
                    }
                } else {
                    panic!("body not expression statement");
                }
            } else {
                panic!("not a function literal");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(exp_stmt) = &program.statements[0] {
            if let Expression::Call(call) = &exp_stmt.expression {
                if let Expression::Identifier(ident) = &*call.function {
                    assert_eq!(ident.value, "add");
                } else {
                    panic!("function not identifier");
                }
                assert_eq!(call.arguments.len(), 3);
                assert_eq!(call.arguments[0].to_string(), "1");
                assert_eq!(call.arguments[1].to_string(), "(2 * 3)");
                assert_eq!(call.arguments[2].to_string(), "(4 + 5)");
            } else {
                panic!("not a call expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_this_expression() {
        let input = "this;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];
        if let Statement::Expression(exp_stmt) = stmt {
            if let Expression::This(_) = &exp_stmt.expression {
                // ok
            } else {
                panic!("not a this expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_super_expression() {
        let input = "super;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];
        if let Statement::Expression(exp_stmt) = stmt {
            if let Expression::Super(_) = &exp_stmt.expression {
                // ok
            } else {
                panic!("not a super expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_new_expression() {
        let input = "new MyClass(1, 2);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];
        if let Statement::Expression(exp_stmt) = stmt {
            if let Expression::New(new_exp) = &exp_stmt.expression {
                assert_eq!(new_exp.class_name.value, "MyClass");
                assert_eq!(new_exp.arguments.len(), 2);
                assert_eq!(new_exp.arguments[0].to_string(), "1");
                assert_eq!(new_exp.arguments[1].to_string(), "2");
            } else {
                panic!("not a new expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_property_access_expression() {
        let input = "myObject.myProperty;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];
        if let Statement::Expression(exp_stmt) = stmt {
            if let Expression::PropertyAccess(prop_access) = &exp_stmt.expression {
                assert_eq!(prop_access.left.to_string(), "myObject");
                assert_eq!(prop_access.property.value, "myProperty");
            } else {
                panic!("not a property access expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_method_call_expression() {
        let input = "myObject.myMethod(1);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];
        if let Statement::Expression(exp_stmt) = stmt {
            if let Expression::Call(call_exp) = &exp_stmt.expression {
                if let Expression::PropertyAccess(prop_access) = &*call_exp.function {
                    assert_eq!(prop_access.left.to_string(), "myObject");
                    assert_eq!(prop_access.property.value, "myMethod");
                } else {
                    panic!("not a property access expression");
                }
                assert_eq!(call_exp.arguments.len(), 1);
                assert_eq!(call_exp.arguments[0].to_string(), "1");
            } else {
                panic!("not a call expression");
            }
        } else {
            panic!("not an expression statement");
        }
    }

    #[test]
    fn test_class_declaration() {
        let input = "class MyClass {}";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::ClassDeclaration(class_decl) = &program.statements[0] {
            assert_eq!(class_decl.name.value, "MyClass");
            assert!(class_decl.super_class.is_none());
            assert!(class_decl.interfaces.is_empty());
            assert!(class_decl.properties.is_empty());
            assert!(class_decl.methods.is_empty());
        } else {
            panic!("statement not a ClassDeclaration");
        }
    }

    #[test]
    fn test_class_with_members_declaration() {
        let input = r#"
        class MyClass {
            public static let a = 1;
            private let b;
            
            public fn methodA(x, y) {
                return x + y;
            }

            private static fn methodB() {
                // do nothing
            }
        }
        "#;

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        let stmt = &program.statements[0];
        if let Statement::ClassDeclaration(class_decl) = stmt {
            assert_eq!(class_decl.name.value, "MyClass");

            // Проверка свойств
            assert_eq!(class_decl.properties.len(), 2);
            let prop1 = &class_decl.properties[0];
            assert_eq!(prop1.name.value, "a");
            assert_eq!(prop1.access_modifier, AccessModifier::Public);
            assert!(prop1.is_static);
            assert!(prop1.value.is_some());

            let prop2 = &class_decl.properties[1];
            assert_eq!(prop2.name.value, "b");
            assert_eq!(prop2.access_modifier, AccessModifier::Private);
            assert!(!prop2.is_static);
            assert!(prop2.value.is_none());

            // Проверка методов
            assert_eq!(class_decl.methods.len(), 2);
            let method1 = &class_decl.methods[0];
            assert_eq!(method1.name.value, "methodA");
            assert_eq!(method1.access_modifier, AccessModifier::Public);
            assert!(!method1.is_static);
            assert_eq!(method1.parameters.len(), 2);

            let method2 = &class_decl.methods[1];
            assert_eq!(method2.name.value, "methodB");
            assert_eq!(method2.access_modifier, AccessModifier::Private);
            assert!(method2.is_static);
            assert_eq!(method2.parameters.len(), 0);
        } else {
            panic!("Statement is not a ClassDeclaration");
        }
    }

    #[test]
    fn test_class_declaration_with_inheritance() {
        let input = "class B extends A {}";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        let stmt = &program.statements[0];
        if let Statement::ClassDeclaration(class_decl) = stmt {
            assert_eq!(class_decl.name.value, "B");
            assert!(class_decl.super_class.is_some());
            let super_class = class_decl.super_class.as_ref().unwrap();
            assert_eq!(super_class.value, "A");
        } else {
            panic!("Statement is not a ClassDeclaration");
        }
    }

    #[test]
    fn test_struct_declaration() {
        let input = r#"
        struct MyStruct {
            public let x;
            let y = 10;
        }
        "#;
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::StructDeclaration(struct_decl) = &program.statements[0] {
            assert_eq!(struct_decl.name.value, "MyStruct");
            assert_eq!(struct_decl.properties.len(), 2);

            let prop1 = &struct_decl.properties[0];
            assert_eq!(prop1.name.value, "x");
            assert_eq!(prop1.access_modifier, AccessModifier::Public);
            assert!(!prop1.is_static);
            assert!(prop1.value.is_none());

            let prop2 = &struct_decl.properties[1];
            assert_eq!(prop2.name.value, "y");
            assert_eq!(prop2.access_modifier, AccessModifier::Private); // По умолчанию
            assert!(!prop2.is_static);
            assert!(prop2.value.is_some());
        } else {
            panic!("statement not a StructDeclaration");
        }
    }

    #[test]
    fn test_interface_declaration() {
        let input = r#"
        interface MyInterface {
            fn doSomething(a, b);
            fn doAnotherThing();
        }
        "#;
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::InterfaceDeclaration(iface_decl) = &program.statements[0] {
            assert_eq!(iface_decl.name.value, "MyInterface");
            assert_eq!(iface_decl.method_signatures.len(), 2);

            let sig1 = &iface_decl.method_signatures[0];
            assert_eq!(sig1.name.value, "doSomething");
            assert_eq!(sig1.parameters.len(), 2);

            let sig2 = &iface_decl.method_signatures[1];
            assert_eq!(sig2.name.value, "doAnotherThing");
            assert_eq!(sig2.parameters.len(), 0);
        } else {
            panic!("statement not an InterfaceDeclaration");
        }
    }
}
