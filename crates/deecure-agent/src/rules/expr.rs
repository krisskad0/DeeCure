// SPDX-License-Identifier: Apache-2.0

//! The small condition language used in rule files.
//!
//! ```text
//! root.used_percent >= 93
//! item.active == 'failed' and item.load == 'loaded'
//! not (item.health >= 80) or cpu.usage > 95
//! ```
//!
//! Values are numbers, text in quotes, `true`, `false`, and fact names. Comparisons:
//! `<`, `<=`, `>`, `>=`, `==`, `!=`. Logic: `and`, `or`, `not`, and brackets.

use std::fmt;

/// A value a fact or a literal can have.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A number.
    Number(f64),
    /// A piece of text.
    Text(String),
    /// Yes or no.
    Bool(bool),
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "a number",
            Value::Text(_) => "text",
            Value::Bool(_) => "true/false",
        }
    }

    /// How the value appears in a finding's text: numbers get at most one decimal.
    pub fn display(&self) -> String {
        match self {
            Value::Number(n) if (n - n.round()).abs() < 0.05 => format!("{n:.0}"),
            Value::Number(n) => format!("{n:.1}"),
            Value::Text(text) => text.clone(),
            Value::Bool(true) => "yes".to_owned(),
            Value::Bool(false) => "no".to_owned(),
        }
    }
}

/// Something that can answer "what is the value of this fact?".
pub trait Lookup {
    /// The value of a fact, or `None` if it is not known on this computer right now.
    fn get(&self, name: &str) -> Option<Value>;
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `==`
    Eq,
    /// `!=`
    Ne,
}

/// A parsed condition.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A fixed value.
    Literal(Value),
    /// A fact, such as `root.used_percent`.
    Fact(String),
    /// `not …`
    Not(Box<Expr>),
    /// `… and …`
    And(Box<Expr>, Box<Expr>),
    /// `… or …`
    Or(Box<Expr>, Box<Expr>),
    /// `… >= …` and friends.
    Compare(Box<Expr>, Op, Box<Expr>),
}

/// Why a condition could not be decided.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// A fact it needs is not known right now.
    Missing(String),
    /// The condition mixes types, such as comparing text with a number. A bug in the rule.
    Type(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Missing(name) => write!(f, "'{name}' is not known"),
            EvalError::Type(why) => f.write_str(why),
        }
    }
}

impl Expr {
    /// Works out the value of the condition.
    pub fn eval(&self, facts: &dyn Lookup) -> Result<Value, EvalError> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Fact(name) => facts
                .get(name)
                .ok_or_else(|| EvalError::Missing(name.clone())),
            Expr::Not(inner) => Ok(Value::Bool(!inner.eval_bool(facts)?)),
            // A side that is surely false decides `and`, even if the other side is unknown.
            Expr::And(a, b) => match (a.eval_bool(facts), b.eval_bool(facts)) {
                (Ok(false), _) | (_, Ok(false)) => Ok(Value::Bool(false)),
                (Ok(true), Ok(true)) => Ok(Value::Bool(true)),
                (Err(e), _) | (_, Err(e)) => Err(e),
            },
            // Likewise, a side that is surely true decides `or`.
            Expr::Or(a, b) => match (a.eval_bool(facts), b.eval_bool(facts)) {
                (Ok(true), _) | (_, Ok(true)) => Ok(Value::Bool(true)),
                (Ok(false), Ok(false)) => Ok(Value::Bool(false)),
                (Err(e), _) | (_, Err(e)) => Err(e),
            },
            Expr::Compare(a, op, b) => {
                compare(&a.eval(facts)?, *op, &b.eval(facts)?).map(Value::Bool)
            }
        }
    }

    /// Works out the condition and insists the answer is true or false.
    pub fn eval_bool(&self, facts: &dyn Lookup) -> Result<bool, EvalError> {
        match self.eval(facts)? {
            Value::Bool(b) => Ok(b),
            other => Err(EvalError::Type(format!(
                "a condition must be true or false, not {}",
                other.type_name()
            ))),
        }
    }

    /// Every fact name the condition uses.
    pub fn fact_names(&self) -> Vec<&str> {
        match self {
            Expr::Literal(_) => Vec::new(),
            Expr::Fact(name) => vec![name.as_str()],
            Expr::Not(inner) => inner.fact_names(),
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Compare(a, _, b) => {
                let mut names = a.fact_names();
                names.extend(b.fact_names());
                names
            }
        }
    }
}

fn compare(a: &Value, op: Op, b: &Value) -> Result<bool, EvalError> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Ok(match op {
            Op::Lt => x < y,
            Op::Le => x <= y,
            Op::Gt => x > y,
            Op::Ge => x >= y,
            Op::Eq => x == y,
            Op::Ne => x != y,
        }),
        (Value::Text(_), Value::Text(_)) | (Value::Bool(_), Value::Bool(_)) => match op {
            Op::Eq => Ok(a == b),
            Op::Ne => Ok(a != b),
            _ => Err(EvalError::Type(format!(
                "{} can only be compared with == and !=",
                a.type_name()
            ))),
        },
        _ => Err(EvalError::Type(format!(
            "cannot compare {} with {}",
            a.type_name(),
            b.type_name()
        ))),
    }
}

/// A condition that could not be read, with the byte position of the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What is wrong.
    pub message: String,
    /// Where, counted in bytes from the start.
    pub at: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at character {})", self.message, self.at + 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Text(String),
    Name(String),
    Op(Op),
    And,
    Or,
    Not,
    True,
    False,
    Open,
    Close,
}

fn tokenize(src: &str) -> Result<Vec<(Token, usize)>, ParseError> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    let error = |message: &str, at| ParseError {
        message: message.to_owned(),
        at,
    };
    while i < bytes.len() {
        let start = i;
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
                continue;
            }
            b'(' | b')' => {
                tokens.push((if c == b'(' { Token::Open } else { Token::Close }, start));
                i += 1;
            }
            b'<' | b'>' | b'=' | b'!' => {
                let next_is_eq = bytes.get(i + 1) == Some(&b'=');
                let op = match (c, next_is_eq) {
                    (b'<', true) => Op::Le,
                    (b'<', false) => Op::Lt,
                    (b'>', true) => Op::Ge,
                    (b'>', false) => Op::Gt,
                    (b'=', true) => Op::Eq,
                    (b'!', true) => Op::Ne,
                    _ => return Err(error("use '==' to compare and '!=' for not equal", start)),
                };
                i += if next_is_eq { 2 } else { 1 };
                tokens.push((Token::Op(op), start));
            }
            b'\'' | b'"' => {
                let end = src[i + 1..]
                    .find(c as char)
                    .ok_or_else(|| error("text is missing its closing quote", start))?;
                tokens.push((Token::Text(src[i + 1..i + 1 + end].to_owned()), start));
                i += end + 2;
            }
            b'0'..=b'9' | b'-' | b'.' => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                let number = src[start..i]
                    .parse()
                    .map_err(|_| error("this is not a number", start))?;
                tokens.push((Token::Number(number), start));
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
                {
                    i += 1;
                }
                let word = &src[start..i];
                let token = match word {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ if word.ends_with('.') || word.contains("..") => {
                        return Err(error("a fact name cannot have an empty part", start));
                    }
                    _ => Token::Name(word.to_owned()),
                };
                tokens.push((token, start));
            }
            _ => return Err(error("unexpected character", start)),
        }
    }
    Ok(tokens)
}

/// Reads a condition such as `root.used_percent >= 93`.
pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(src)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        len: src.len(),
    };
    let expr = parser.or()?;
    match parser.tokens.get(parser.pos) {
        None => Ok(expr),
        Some((_, at)) => Err(ParseError {
            message: "unexpected extra text; did you forget 'and' or 'or'?".to_owned(),
            at: *at,
        }),
    }
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
    len: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(token, _)| token)
    }

    fn here(&self) -> usize {
        self.tokens.get(self.pos).map_or(self.len, |(_, at)| *at)
    }

    fn fail<T>(&self, message: &str) -> Result<T, ParseError> {
        Err(ParseError {
            message: message.to_owned(),
            at: self.here(),
        })
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.and()?;
        while self.peek() == Some(&Token::Or) {
            self.pos += 1;
            left = Expr::Or(Box::new(left), Box::new(self.and()?));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.not()?;
        while self.peek() == Some(&Token::And) {
            self.pos += 1;
            left = Expr::And(Box::new(left), Box::new(self.not()?));
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == Some(&Token::Not) {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.value()?;
        if let Some(Token::Op(op)) = self.peek() {
            let op = *op;
            self.pos += 1;
            return Ok(Expr::Compare(Box::new(left), op, Box::new(self.value()?)));
        }
        Ok(left)
    }

    fn value(&mut self) -> Result<Expr, ParseError> {
        let Some(token) = self.peek().cloned() else {
            return self.fail("the condition ends too early");
        };
        self.pos += 1;
        match token {
            Token::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            Token::Text(text) => Ok(Expr::Literal(Value::Text(text))),
            Token::True => Ok(Expr::Literal(Value::Bool(true))),
            Token::False => Ok(Expr::Literal(Value::Bool(false))),
            Token::Name(name) => Ok(Expr::Fact(name)),
            Token::Open => {
                let inner = self.or()?;
                if self.peek() != Some(&Token::Close) {
                    return self.fail("missing ')'");
                }
                self.pos += 1;
                Ok(inner)
            }
            _ => {
                self.pos -= 1;
                self.fail("expected a value, a fact name, or '('")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Facts(HashMap<&'static str, Value>);

    impl Lookup for Facts {
        fn get(&self, name: &str) -> Option<Value> {
            self.0.get(name).cloned()
        }
    }

    fn facts() -> Facts {
        Facts(HashMap::from([
            ("root.used_percent", Value::Number(94.2)),
            ("item.active", Value::Text("failed".into())),
            ("on_battery", Value::Bool(true)),
        ]))
    }

    fn check(src: &str) -> Result<bool, EvalError> {
        parse(src).unwrap().eval_bool(&facts())
    }

    #[test]
    fn comparisons() {
        assert_eq!(check("root.used_percent >= 93"), Ok(true));
        assert_eq!(check("root.used_percent < 93.0"), Ok(false));
        assert_eq!(check("item.active == 'failed'"), Ok(true));
        assert_eq!(check("item.active != \"failed\""), Ok(false));
        assert_eq!(check("on_battery == true"), Ok(true));
        assert_eq!(check("-1 < 0"), Ok(true));
    }

    #[test]
    fn logic_and_brackets() {
        assert_eq!(
            check("root.used_percent > 90 and item.active == 'failed'"),
            Ok(true)
        );
        assert_eq!(check("not (root.used_percent > 90) or false"), Ok(false));
        assert_eq!(
            check("false or true and false"),
            Ok(false),
            "and binds tighter than or"
        );
        assert_eq!(check("on_battery"), Ok(true));
    }

    #[test]
    fn unknown_facts_decide_only_when_they_must() {
        assert_eq!(
            check("battery.health < 60"),
            Err(EvalError::Missing("battery.health".into()))
        );
        assert_eq!(check("false and battery.health < 60"), Ok(false));
        assert_eq!(check("battery.health < 60 or true"), Ok(true));
        assert!(matches!(
            check("battery.health < 60 and true"),
            Err(EvalError::Missing(_))
        ));
    }

    #[test]
    fn type_mistakes_are_errors() {
        assert!(matches!(check("item.active > 3"), Err(EvalError::Type(_))));
        assert!(matches!(
            check("item.active < 'z'"),
            Err(EvalError::Type(_))
        ));
        assert!(matches!(
            check("root.used_percent"),
            Err(EvalError::Type(_))
        ));
    }

    #[test]
    fn syntax_errors_point_at_the_problem() {
        for (src, at) in [
            ("root.used_percent = 93", 18),
            ("(root.used_percent > 93", 23),
            ("root.used_percent > ", 20),
            ("x > 'open", 4),
            ("x > 1 y", 6),
            ("x.. > 1", 0),
            ("x > 1 & y", 6),
        ] {
            let error = parse(src).unwrap_err();
            assert_eq!(error.at, at, "{src}: {error}");
        }
    }

    #[test]
    fn fact_names_are_listed() {
        let expr = parse("a.b > 1 and not (c == 'x' or d)").unwrap();
        assert_eq!(expr.fact_names(), ["a.b", "c", "d"]);
    }

    #[test]
    fn display_rounds_numbers() {
        assert_eq!(Value::Number(13.84).display(), "13.8");
        assert_eq!(Value::Number(93.0).display(), "93");
        assert_eq!(Value::Number(79.97).display(), "80");
        assert_eq!(Value::Text("BAT0".into()).display(), "BAT0");
    }
}
