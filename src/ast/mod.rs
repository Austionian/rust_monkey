use crate::token::Token;
use core::panic;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait TokenLiteral {
    fn token_literal(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    ExpressStatement(Expression),
    BlockStatement(BlockStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    // Token ie prefix, Right
    PrefixExpression((Token, Box<Expression>)),
    // Token, Left, Right
    InfixExpression((Token, Box<Expression>, Box<Expression>)),
    IdentExpression(Token),
    IntExpression(Token),
    StringExpression(Token),
    BoolExpression(Token),
    // Token, condition, consequence, alternative
    IfExpression(
        Box<Expression>,
        Box<BlockStatement>,
        Option<Box<BlockStatement>>,
    ),
    // Token, idents, body
    FunctionLiteral(Token, Vec<Token>, BlockStatement),
    // Token ie function, arguments
    CallExpression(Box<Expression>, Vec<Expression>),
    ArrayExpression(Vec<Expression>),
    // left, index
    IndexExpression(Box<Expression>, Box<Expression>),
    HashLiteral(Map),
    UnknownExpression(Token),
}

#[derive(Debug, Clone, Eq)]
pub struct Map {
    pub pairs: HashMap<Expression, Expression>,
}

impl Hash for Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Expression::StringExpression(t) => t.hash(state),
            Expression::BoolExpression(t) => t.hash(state),
            Expression::IntExpression(t) => t.hash(state),
            Expression::InfixExpression(t) => t.0.hash(state),
            _ => panic!("not allowed, {}", self.to_string()),
        };
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }

    fn ne(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::UnknownExpression(Token::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub token: Token,
    pub name: Token,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub token: Token,
    pub value: Expression,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        for s in self.statements.iter() {
            buffer.push_str(&s.to_string());
        }
        write!(f, "{buffer}")
    }
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        for s in self.statements.iter() {
            buffer.push_str(&s.to_string());
        }

        write!(f, "{buffer}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        match self {
            Self::LetStatement(s) => {
                buffer.push_str(&format!(
                    "{} {} = {};",
                    s.token.token_literal(),
                    s.name.token_literal(),
                    s.value
                ));
            }
            Self::ReturnStatement(s) => buffer.push_str(&s.to_string()),
            Self::ExpressStatement(s) => buffer.push_str(&s.to_string()),
            Self::BlockStatement(s) => buffer.push_str(&s.to_string()),
        };

        write!(f, "{buffer}")
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        match self {
            Self::IntExpression(t) => buffer.push_str(&t.token_literal()),
            Self::InfixExpression(t) => {
                buffer.push_str(&format!("({} {} {})", t.1, t.0.token_literal(), t.2))
            }
            Self::IdentExpression(t) => buffer.push_str(&t.token_literal()),
            Self::PrefixExpression(t) => {
                buffer.push_str(&format!("({}{})", t.0.token_literal(), t.1))
            }
            Self::BoolExpression(t) => buffer.push_str(&t.token_literal()),
            Self::IfExpression(condition, consequnce, alternative) => {
                buffer.push_str(&format!("if {} {}", condition, consequnce));

                if let Some(alt) = alternative {
                    buffer.push_str(&format!("else {}", alt.to_string()));
                }
            }
            Self::FunctionLiteral(t, params, body) => {
                buffer.push_str(&format!(
                    "{} ({}) ",
                    t.token_literal(),
                    params
                        .iter()
                        .map(|p| p.token_literal())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                buffer.push_str(body.to_string().as_str());
            }
            Self::CallExpression(func, args) => {
                buffer.push_str(&format!(
                    "{}({})",
                    func,
                    args.iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            Self::StringExpression(t) => buffer.push_str(&t.token_literal()),
            Self::ArrayExpression(array) => {
                buffer.push_str(&format!(
                    "[{}]",
                    array
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            Self::IndexExpression(left, index) => {
                buffer.push_str(&format!("({}[{}])", left, index))
            }
            Self::HashLiteral(map) => {
                buffer.push_str(&format!(
                    "{{{}}}",
                    map.pairs
                        .iter()
                        .map(|(k, v)| format!("{k}: {v},"))
                        .collect::<String>()
                ));
            }
            Self::UnknownExpression(t) => buffer.push_str(&t.token_literal()),
        }

        write!(f, "{buffer}")
    }
}

impl Display for LetStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} = ;",
            self.token.token_literal(),
            self.name.token_literal()
        )
    }
}

impl Display for ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ;", self.token.token_literal())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    impl Program {
        fn new() -> Self {
            Program { statements: vec![] }
        }
    }

    #[test]
    fn test_string() {
        let mut program = Program::new();

        program
            .statements
            .push(Statement::LetStatement(LetStatement {
                token: Token::LET,
                name: Token::IDENT("test".to_string()),
                value: Expression::default(),
            }));

        assert_eq!(program.to_string(), "let test = fn;");
    }

    #[test]
    fn test_iden() {
        let mut program = Program::new();

        program
            .statements
            .push(Statement::LetStatement(LetStatement {
                token: Token::LET,
                name: Token::IDENT("myVar".to_string()),
                value: Expression::IdentExpression(Token::IDENT("anotherVar".to_string())),
            }));

        assert_eq!(program.to_string(), "let myVar = anotherVar;");
    }

    #[test]
    fn test_prefix() {
        let mut program = Program::new();
        program
            .statements
            .push(Statement::ExpressStatement(Expression::PrefixExpression((
                Token::BANG,
                Box::new(Expression::PrefixExpression((
                    Token::MINUS,
                    Box::new(Expression::IntExpression(Token::IDENT("a".to_string()))),
                ))),
            ))));

        assert_eq!(program.to_string(), "(!(-a))");
    }
}
