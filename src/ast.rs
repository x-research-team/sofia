use crate::token::Token;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Program(p) => write!(f, "{}", p),
            Node::Statement(s) => write!(f, "{}", s),
            Node::Expression(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
        }
    }
    pub fn string(&self) -> String {
        let mut s = String::new();
        for stmt in &self.statements {
            s.push_str(&stmt.to_string());
        }
        s
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
    ClassDeclaration(ClassDeclaration),
    InterfaceDeclaration(InterfaceDeclaration),
    StructDeclaration(StructDeclaration),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Let(s) => write!(f, "{}", s),
            Statement::Return(s) => write!(f, "{}", s),
            Statement::Expression(s) => write!(f, "{}", s),
            Statement::Block(s) => write!(f, "{}", s),
            Statement::ClassDeclaration(s) => write!(f, "{}", s),
            Statement::InterfaceDeclaration(s) => write!(f, "{}", s),
            Statement::StructDeclaration(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    Boolean(BooleanLiteral),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    If(IfExpression),
    FunctionLiteral(FunctionLiteral),
    Call(CallExpression),
    StringLiteral(StringLiteral),
    ArrayLiteral(ArrayLiteral),
    New(NewExpression),
    This(ThisExpression),
    Super(SuperExpression),
    PropertyAccess(PropertyAccessExpression),
    MethodCall(MethodCallExpression),
    Match(MatchExpression),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Identifier(i) => write!(f, "{}", i.value),
            Expression::IntegerLiteral(i) => write!(f, "{}", i.value),
            Expression::Boolean(b) => write!(f, "{}", b.value),
            Expression::Prefix(p) => write!(f, "({}{})", p.operator, p.right),
            Expression::Infix(i) => write!(f, "({} {} {})", i.left, i.operator, i.right),
            Expression::If(i) => write!(f, "{}", i),
            Expression::FunctionLiteral(fl) => write!(f, "{}", fl),
            Expression::Call(c) => write!(f, "{}", c),
            Expression::Match(m) => write!(f, "{}", m),
            Expression::StringLiteral(s) => write!(f, "{}", s.value),
            Expression::ArrayLiteral(a) => {
                let elements: Vec<String> = a.elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Expression::New(n) => write!(f, "{}", n),
            Expression::This(t) => write!(f, "{}", t),
            Expression::Super(s) => write!(f, "{}", s),
            Expression::PropertyAccess(p) => write!(f, "{}", p),
            Expression::MethodCall(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

impl fmt::Display for LetStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} = {};",
            self.token.literal, self.name.value, self.value
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}

impl fmt::Display for ReturnStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {};", self.token.literal, self.return_value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}

impl fmt::Display for ExpressionStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BooleanLiteral {
    pub token: Token,
    pub value: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl fmt::Display for IfExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence)?;
        if let Some(alt) = &self.alternative {
            write!(f, " else {}", alt)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl fmt::Display for FunctionLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params: Vec<String> = self.parameters.iter().map(|p| p.value.clone()).collect();
        write!(
            f,
            "{}({}) {}",
            self.token.literal,
            params.join(", "),
            self.body
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl fmt::Display for CallExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args: Vec<String> = self.arguments.iter().map(|a| a.to_string()).collect();
        write!(f, "{}({})", self.function, args.join(", "))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AccessModifier {
    Public,
    Private,
}

impl fmt::Display for AccessModifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccessModifier::Public => write!(f, "public"),
            AccessModifier::Private => write!(f, "private"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub super_class: Option<Identifier>,
    pub interfaces: Vec<Identifier>,
    pub properties: Vec<PropertyDeclaration>,
    pub methods: Vec<MethodDeclaration>,
}

impl fmt::Display for ClassDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("class {} ", self.name.value));
        if let Some(sc) = &self.super_class {
            s.push_str(&format!("extends {} ", sc.value));
        }
        if !self.interfaces.is_empty() {
            let interfaces: Vec<String> = self.interfaces.iter().map(|i| i.value.clone()).collect();
            s.push_str(&format!("implements {} ", interfaces.join(", ")));
        }
        s.push_str("{\n");
        for prop in &self.properties {
            s.push_str(&format!("    {}\n", prop));
        }
        for method in &self.methods {
            s.push_str(&format!("    {}\n", method));
        }
        s.push_str("}");
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InterfaceDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub method_signatures: Vec<MethodSignatureDeclaration>,
}

impl fmt::Display for InterfaceDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("interface {} {{\n", self.name.value));
        for sig in &self.method_signatures {
            s.push_str(&format!("    {};\n", sig));
        }
        s.push_str("}");
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub properties: Vec<PropertyDeclaration>,
}

impl fmt::Display for StructDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push_str(&format!("struct {} {{\n", self.name.value));
        for prop in &self.properties {
            s.push_str(&format!("    {};\n", prop));
        }
        s.push_str("}");
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PropertyDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub value: Option<Expression>,
    pub access_modifier: AccessModifier,
    pub is_static: bool,
}

impl fmt::Display for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        if self.is_static {
            s.push_str("static ");
        }
        s.push_str(&format!("{} {}", self.access_modifier, self.name.value));
        if let Some(val) = &self.value {
            s.push_str(&format!(" = {}", val));
        }
        s.push(';');
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub access_modifier: AccessModifier,
    pub is_static: bool,
}

impl fmt::Display for MethodDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params: Vec<String> = self.parameters.iter().map(|p| p.value.clone()).collect();
        let mut s = String::new();
        if self.is_static {
            s.push_str("static ");
        }
        s.push_str(&format!(
            "{} {}({}) {}",
            self.access_modifier,
            self.name.value,
            params.join(", "),
            self.body
        ));
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodSignatureDeclaration {
    pub token: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
}

impl fmt::Display for MethodSignatureDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params: Vec<String> = self.parameters.iter().map(|p| p.value.clone()).collect();
        write!(f, "{}({})", self.name.value, params.join(", "))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewExpression {
    pub token: Token,
    pub class_name: Identifier,
    pub arguments: Vec<Expression>,
}

impl fmt::Display for NewExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args: Vec<String> = self.arguments.iter().map(|a| a.to_string()).collect();
        write!(f, "new {}({})", self.class_name.value, args.join(", "))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThisExpression {
    pub token: Token,
}

impl fmt::Display for ThisExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "this")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SuperExpression {
    pub token: Token,
}

impl fmt::Display for SuperExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "super")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PropertyAccessExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub property: Identifier,
}

impl fmt::Display for PropertyAccessExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}.{})", self.left, self.property.value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodCallExpression {
    pub token: Token,
    pub object: Box<Expression>,
    pub method: Identifier,
    pub arguments: Vec<Expression>,
}

impl fmt::Display for MethodCallExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args: Vec<String> = self.arguments.iter().map(|a| a.to_string()).collect();
        write!(
            f,
            "{}.{}({})",
            self.object,
            self.method.value,
            args.join(", ")
        )
    }
}

/// Представляет выражение `match`.
#[derive(Debug, PartialEq, Clone)]
pub struct MatchExpression {
    pub token: Token,
    pub value: Box<Expression>,
    pub arms: Vec<MatchArm>,
}

impl fmt::Display for MatchExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arms: Vec<String> = self.arms.iter().map(|arm| arm.to_string()).collect();
        write!(f, "match {} {{\n{}\n}}", self.value, arms.join("\n"))
    }
}

/// Представляет ветвь `match` выражения.
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Добавляем опциональный гард
    pub consequence: BlockStatement,
}

impl fmt::Display for MatchArm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard_str = if let Some(guard_expr) = &self.guard {
            format!(" if {}", guard_expr)
        } else {
            "".to_string()
        };
        write!(f, "    {}{} => {}", self.pattern, guard_str, self.consequence)
    }
}

/// Представляет шаблон сопоставления в `match` выражении.
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Literal(Expression),    // Например, 1, "hello", true
    Identifier(Identifier), // Например, x
    Range(RangePattern),    // Например, 1..5
    Tuple(Vec<Pattern>),    // Например, (1, x, "test")
    Struct(StructPattern),  // Например, Point { x: 0, y }
    Wildcard,               // Например, _
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pattern::Literal(expr) => write!(f, "{}", expr),
            Pattern::Identifier(ident) => write!(f, "{}", ident.value),
            Pattern::Range(range_pattern) => write!(f, "{}", range_pattern),
            Pattern::Tuple(patterns) => {
                let p_str: Vec<String> = patterns.iter().map(|p| p.to_string()).collect();
                write!(f, "({})", p_str.join(", "))
            }
            Pattern::Struct(struct_pattern) => write!(f, "{}", struct_pattern),
            Pattern::Wildcard => write!(f, "_"),
        }
    }
}

/// Представляет шаблон диапазона, например `1..5` или `a..=b`.
#[derive(Debug, PartialEq, Clone)]
pub struct RangePattern {
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub inclusive: bool, // true для ..=, false для ..
}

impl fmt::Display for RangePattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.inclusive {
            write!(f, "{}..={}", self.start, self.end)
        } else {
            write!(f, "{}..{}", self.start, self.end)
        }
    }
}

/// Представляет шаблон структуры, например `Point { x: 0, y }`.
#[derive(Debug, PartialEq, Clone)]
pub struct StructPattern {
    pub name: Identifier,
    pub fields: Vec<(Identifier, Option<Pattern>)>, // (имя_поля, опциональный_шаблон_значения)
}

impl fmt::Display for StructPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let field_strs: Vec<String> = self
            .fields
            .iter()
            .map(|(ident, pattern_opt)| {
                if let Some(pattern) = pattern_opt {
                    format!("{}: {}", ident.value, pattern)
                } else {
                    ident.value.clone()
                }
            })
            .collect();
        write!(f, "{} {{ {} }}", self.name.value, field_strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_match_expression_display() {
        let token = Token {
            token_type: TokenType::Match,
            literal: "match".to_string(),
        };
        let value = Box::new(Expression::Identifier(Identifier {
            token: Token {
                token_type: TokenType::Ident,
                literal: "x".to_string(),
            },
            value: "x".to_string(),
        }));
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Expression::IntegerLiteral(IntegerLiteral {
                    token: Token {
                        token_type: TokenType::Int,
                        literal: "1".to_string(),
                    },
                    value: 1,
                })),
                guard: None,
                consequence: BlockStatement {
                    token: Token {
                        token_type: TokenType::LBrace,
                        literal: "{".to_string(),
                    },
                    statements: vec![Statement::Expression(ExpressionStatement {
                        token: Token {
                            token_type: TokenType::Int,
                            literal: "10".to_string(),
                        },
                        expression: Expression::IntegerLiteral(IntegerLiteral {
                            token: Token {
                                token_type: TokenType::Int,
                                literal: "10".to_string(),
                            },
                            value: 10,
                        }),
                    })],
                },
            },
            MatchArm {
                pattern: Pattern::Identifier(Identifier {
                    token: Token {
                        token_type: TokenType::Ident,
                        literal: "y".to_string(),
                    },
                    value: "y".to_string(),
                }),
                guard: None,
                consequence: BlockStatement {
                    token: Token {
                        token_type: TokenType::LBrace,
                        literal: "{".to_string(),
                    },
                    statements: vec![Statement::Expression(ExpressionStatement {
                        token: Token {
                            token_type: TokenType::Int,
                            literal: "20".to_string(),
                        },
                        expression: Expression::IntegerLiteral(IntegerLiteral {
                            token: Token {
                                token_type: TokenType::Int,
                                literal: "20".to_string(),
                            },
                            value: 20,
                        }),
                    })],
                },
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                consequence: BlockStatement {
                    token: Token {
                        token_type: TokenType::LBrace,
                        literal: "{".to_string(),
                    },
                    statements: vec![Statement::Expression(ExpressionStatement {
                        token: Token {
                            token_type: TokenType::Int,
                            literal: "30".to_string(),
                        },
                        expression: Expression::IntegerLiteral(IntegerLiteral {
                            token: Token {
                                token_type: TokenType::Int,
                                literal: "30".to_string(),
                            },
                            value: 30,
                        }),
                    })],
                },
            },
        ];

        let match_expr = MatchExpression { token, value, arms };

        let expected = "match x {\n    1 => 10\n    y => 20\n    _ => 30\n}";
        assert_eq!(match_expr.to_string(), expected);
    }

    #[test]
    fn test_range_pattern_display() {
        let start = Box::new(Expression::IntegerLiteral(IntegerLiteral {
            token: Token {
                token_type: TokenType::Int,
                literal: "1".to_string(),
            },
            value: 1,
        }));
        let end = Box::new(Expression::IntegerLiteral(IntegerLiteral {
            token: Token {
                token_type: TokenType::Int,
                literal: "5".to_string(),
            },
            value: 5,
        }));

        let range_pattern_inclusive = RangePattern {
            start: start.clone(),
            end: end.clone(),
            inclusive: true,
        };
        assert_eq!(range_pattern_inclusive.to_string(), "1..=5");

        let range_pattern_exclusive = RangePattern {
            start,
            end,
            inclusive: false,
        };
        assert_eq!(range_pattern_exclusive.to_string(), "1..5");
    }

    #[test]
    fn test_struct_pattern_display() {
        let name = Identifier {
            token: Token {
                token_type: TokenType::Ident,
                literal: "Point".to_string(),
            },
            value: "Point".to_string(),
        };
        let fields = vec![
            (
                Identifier {
                    token: Token {
                        token_type: TokenType::Ident,
                        literal: "x".to_string(),
                    },
                    value: "x".to_string(),
                },
                Some(Pattern::Literal(Expression::IntegerLiteral(
                    IntegerLiteral {
                        token: Token {
                            token_type: TokenType::Int,
                            literal: "0".to_string(),
                        },
                        value: 0,
                    },
                ))),
            ),
            (
                Identifier {
                    token: Token {
                        token_type: TokenType::Ident,
                        literal: "y".to_string(),
                    },
                    value: "y".to_string(),
                },
                None,
            ),
        ];

        let struct_pattern = StructPattern { name, fields };
        assert_eq!(struct_pattern.to_string(), "Point { x: 0, y }");
    }

    #[test]
    fn test_tuple_pattern_display() {
        let patterns = vec![
            Pattern::Literal(Expression::IntegerLiteral(IntegerLiteral {
                token: Token {
                    token_type: TokenType::Int,
                    literal: "1".to_string(),
                },
                value: 1,
            })),
            Pattern::Identifier(Identifier {
                token: Token {
                    token_type: TokenType::Ident,
                    literal: "x".to_string(),
                },
                value: "x".to_string(),
            }),
            Pattern::Literal(Expression::StringLiteral(StringLiteral {
                token: Token {
                    token_type: TokenType::String,
                    literal: "test".to_string(),
                },
                value: "test".to_string(),
            })),
        ];

        let tuple_pattern = Pattern::Tuple(patterns);
        assert_eq!(tuple_pattern.to_string(), "(1, x, test)");
    }
}
