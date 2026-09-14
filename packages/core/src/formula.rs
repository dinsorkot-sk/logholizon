use crate::{error::AppError, repository::Field};
use serde_json::{Number, Value};
use std::collections::{HashMap, HashSet};

const MAX_EXPR: usize = 2048;
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Str(String),
    Ref(String),
    Id(String),
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    LParen,
    RParen,
    Comma,
}

fn err(message: impl Into<String>) -> anyhow::Error {
    AppError::BadRequest(message.into()).into()
}

fn lex(input: &str) -> anyhow::Result<Vec<Token>> {
    if input.len() > MAX_EXPR {
        return Err(err("formula expression is too long"));
    }
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => i += 1,
            '{' => {
                let s = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(err("unclosed field reference"));
                }
                let n: String = chars[s..i].iter().collect();
                if !valid_name(&n) {
                    return Err(err("invalid field reference"));
                }
                out.push(Token::Ref(n));
                i += 1;
            }
            '"' => {
                i += 1;
                let mut v = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                    }
                    v.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(err("unclosed string"));
                }
                out.push(Token::Str(v));
                i += 1;
            }
            '\'' => {
                i += 1;
                let mut v = String::new();
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                    }
                    v.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(err("unclosed string"));
                }
                out.push(Token::Str(v));
                i += 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let s = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let n: String = chars[s..i].iter().collect();
                out.push(Token::Num(n.parse().map_err(|_| err("invalid number"))?));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let s = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                out.push(Token::Id(chars[s..i].iter().collect()));
            }
            '+' => {
                out.push(Token::Plus);
                i += 1
            }
            '-' => {
                out.push(Token::Minus);
                i += 1
            }
            '*' => {
                out.push(Token::Star);
                i += 1
            }
            '/' => {
                out.push(Token::Slash);
                i += 1
            }
            '(' => {
                out.push(Token::LParen);
                i += 1
            }
            ')' => {
                out.push(Token::RParen);
                i += 1
            }
            ',' => {
                out.push(Token::Comma);
                i += 1
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::Ne);
                    i += 2
                } else {
                    out.push(Token::Not);
                    i += 1
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    i += 2
                } else {
                    i += 1
                }
                out.push(Token::Eq)
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::Le);
                    i += 2
                } else {
                    out.push(Token::Lt);
                    i += 1
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::Ge);
                    i += 2
                } else {
                    out.push(Token::Gt);
                    i += 1
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    out.push(Token::And);
                    i += 2
                } else {
                    return Err(err("expected &&"));
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    out.push(Token::Or);
                    i += 2
                } else {
                    return Err(err("expected ||"));
                }
            }
            _ => return Err(err(format!("unsupported character: {}", chars[i]))),
        }
    }
    Ok(out)
}

fn valid_name(s: &str) -> bool {
    !s.trim().is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn truth(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        _ => false,
    }
}
fn num(v: &Value) -> Option<f64> {
    v.as_f64()
}
fn val_num(n: f64) -> Value {
    Number::from_f64(n)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

struct Parser<'a> {
    t: &'a [Token],
    p: usize,
    vars: &'a HashMap<String, Value>,
    depth: usize,
}
impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.t.get(self.p)
    }
    fn take(&mut self) -> Option<Token> {
        let x = self.t.get(self.p).cloned();
        self.p += usize::from(x.is_some());
        x
    }
    fn parse(&mut self) -> anyhow::Result<Value> {
        let v = self.or()?;
        if self.peek().is_some() {
            return Err(err("unexpected formula token"));
        }
        Ok(v)
    }
    fn or(&mut self) -> anyhow::Result<Value> {
        let mut v = self.and()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.take();
            let r = self.and()?;
            v = Value::Bool(truth(&v) || truth(&r));
        }
        Ok(v)
    }
    fn and(&mut self) -> anyhow::Result<Value> {
        let mut v = self.eq()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.take();
            let r = self.eq()?;
            v = Value::Bool(truth(&v) && truth(&r));
        }
        Ok(v)
    }
    fn eq(&mut self) -> anyhow::Result<Value> {
        let mut v = self.cmp()?;
        loop {
            let equal = match self.peek() {
                Some(Token::Eq) => true,
                Some(Token::Ne) => false,
                _ => break,
            };
            self.take();
            let r = self.cmp()?;
            v = Value::Bool(if equal {
                compare(&v, &r)?
            } else {
                !compare(&v, &r)?
            });
        }
        Ok(v)
    }
    fn cmp(&mut self) -> anyhow::Result<Value> {
        let mut v = self.add()?;
        loop {
            let op = match self.peek() {
                Some(Token::Lt) => 0,
                Some(Token::Le) => 1,
                Some(Token::Gt) => 2,
                Some(Token::Ge) => 3,
                _ => break,
            };
            self.take();
            let r = self.add()?;
            v = Value::Bool(order(&v, &r, op)?);
        }
        Ok(v)
    }
    fn add(&mut self) -> anyhow::Result<Value> {
        let mut v = self.mul()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.take();
                    v = add(&v, &self.mul()?)?
                }
                Some(Token::Minus) => {
                    self.take();
                    v = bin_num(&v, &self.mul()?, |a, b| a - b)?
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn mul(&mut self) -> anyhow::Result<Value> {
        let mut v = self.unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.take();
                    v = bin_num(&v, &self.unary()?, |a, b| a * b)?
                }
                Some(Token::Slash) => {
                    self.take();
                    let r = self.unary()?;
                    let b = num(&r).ok_or_else(|| err("division requires numbers"))?;
                    if b == 0.0 {
                        return Err(err("division by zero"));
                    }
                    v = bin_num(&v, &r, |a, b| a / b)?
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn unary(&mut self) -> anyhow::Result<Value> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.take();
            let v = self.unary()?;
            return Ok(val_num(
                -num(&v).ok_or_else(|| err("unary minus requires a number"))?,
            ));
        }
        if matches!(self.peek(), Some(Token::Not)) {
            self.take();
            return Ok(Value::Bool(!truth(&self.unary()?)));
        }
        self.primary()
    }
    fn primary(&mut self) -> anyhow::Result<Value> {
        if self.depth > MAX_DEPTH {
            return Err(err("formula evaluation depth exceeded"));
        }
        match self.take() {
            Some(Token::Num(n)) => Ok(val_num(n)),
            Some(Token::Str(s)) => Ok(Value::String(s)),
            Some(Token::Ref(k)) => Ok(self.vars.get(&k).cloned().unwrap_or(Value::Null)),
            Some(Token::Id(name)) => {
                if name.eq_ignore_ascii_case("true") {
                    return Ok(Value::Bool(true));
                }
                if name.eq_ignore_ascii_case("false") {
                    return Ok(Value::Bool(false));
                }
                if name.eq_ignore_ascii_case("null") {
                    return Ok(Value::Null);
                }
                if !matches!(self.peek(), Some(Token::LParen)) {
                    return Err(err(format!("unknown identifier: {name}")));
                }
                self.take();
                let mut args = Vec::new();
                if !matches!(self.peek(), Some(Token::RParen)) {
                    loop {
                        args.push(self.or()?);
                        if !matches!(self.peek(), Some(Token::Comma)) {
                            break;
                        }
                        self.take();
                    }
                }
                if !matches!(self.take(), Some(Token::RParen)) {
                    return Err(err("expected )"));
                }
                function(&name, args)
            }
            Some(Token::LParen) => {
                let v = self.or()?;
                if !matches!(self.take(), Some(Token::RParen)) {
                    return Err(err("expected )"));
                }
                Ok(v)
            }
            _ => Err(err("expected value")),
        }
    }
}
fn compare(a: &Value, b: &Value) -> anyhow::Result<bool> {
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return Ok(x == y);
    }
    Ok(a == b)
}
fn order(a: &Value, b: &Value, op: u8) -> anyhow::Result<bool> {
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return Ok(match op {
            0 => x < y,
            1 => x <= y,
            2 => x > y,
            _ => x >= y,
        });
    }
    let (x, y) = (
        a.as_str()
            .ok_or_else(|| err("comparison requires same scalar types"))?,
        b.as_str()
            .ok_or_else(|| err("comparison requires same scalar types"))?,
    );
    Ok(match op {
        0 => x < y,
        1 => x <= y,
        2 => x > y,
        _ => x >= y,
    })
}
fn add(a: &Value, b: &Value) -> anyhow::Result<Value> {
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return Ok(val_num(x + y));
    }
    Ok(Value::String(format!("{}{}", display(a), display(b))))
}
fn bin_num<F: Fn(f64, f64) -> f64>(a: &Value, b: &Value, f: F) -> anyhow::Result<Value> {
    Ok(val_num(f(
        num(a).ok_or_else(|| err("operator requires numbers"))?,
        num(b).ok_or_else(|| err("operator requires numbers"))?,
    )))
}
fn display(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

fn function(name: &str, args: Vec<Value>) -> anyhow::Result<Value> {
    let n = name.to_ascii_lowercase();
    match n.as_str() {
        "if" => {
            if args.len() != 3 {
                return Err(err("if requires 3 arguments"));
            }
            Ok(if truth(&args[0]) {
                args[1].clone()
            } else {
                args[2].clone()
            })
        }
        "coalesce" => Ok(args
            .into_iter()
            .find(|v| !v.is_null())
            .unwrap_or(Value::Null)),
        "abs" => one_num(&args, |x| x.abs()),
        "round" => one_num(&args, |x| x.round()),
        "floor" => one_num(&args, |x| x.floor()),
        "ceil" => one_num(&args, |x| x.ceil()),
        "len" => {
            let s = one_str(&args)?;
            Ok(val_num(s.chars().count() as f64))
        }
        "upper" => Ok(Value::String(one_str(&args)?.to_uppercase())),
        "lower" => Ok(Value::String(one_str(&args)?.to_lowercase())),
        "trim" => Ok(Value::String(one_str(&args)?.trim().to_string())),
        "contains" => two_str(&args).map(|(a, b)| Value::Bool(a.contains(b))),
        "starts_with" => two_str(&args).map(|(a, b)| Value::Bool(a.starts_with(b))),
        "ends_with" => two_str(&args).map(|(a, b)| Value::Bool(a.ends_with(b))),
        "concat" => Ok(Value::String(
            args.iter().map(display).collect::<Vec<_>>().join(""),
        )),
        "year" => date_part(&args, 0),
        "month" => date_part(&args, 1),
        "day" => date_part(&args, 2),
        "date" => {
            let s = one_str(&args)?;
            if s.len() < 10 {
                return Err(err("date requires YYYY-MM-DD"));
            }
            Ok(Value::String(s[..10].to_string()))
        }
        "days_between" => {
            let (a, b) = two_str(&args)?;
            Ok(val_num(day_number(a)? - day_number(b)?))
        }
        _ => Err(err(format!("unknown formula function: {name}"))),
    }
}
fn one_num<F: Fn(f64) -> f64>(a: &[Value], f: F) -> anyhow::Result<Value> {
    if a.len() != 1 {
        return Err(err("function requires 1 argument"));
    }
    Ok(val_num(f(
        num(&a[0]).ok_or_else(|| err("argument must be a number"))?
    )))
}
fn one_str(a: &[Value]) -> anyhow::Result<&str> {
    if a.len() != 1 {
        return Err(err("function requires 1 argument"));
    }
    a[0].as_str()
        .ok_or_else(|| err("argument must be a string"))
}
fn two_str(a: &[Value]) -> anyhow::Result<(&str, &str)> {
    if a.len() != 2 {
        return Err(err("function requires 2 arguments"));
    }
    Ok((
        a[0].as_str()
            .ok_or_else(|| err("argument must be a string"))?,
        a[1].as_str()
            .ok_or_else(|| err("argument must be a string"))?,
    ))
}
fn date_part(a: &[Value], part: usize) -> anyhow::Result<Value> {
    let s = one_str(a)?;
    let p: Vec<&str> = s.split(['-', 'T', ' ']).collect();
    if p.len() < 3 {
        return Err(err("invalid ISO date"));
    }
    let x = p[part]
        .parse::<f64>()
        .map_err(|_| err("invalid ISO date"))?;
    Ok(val_num(x))
}
fn day_number(s: &str) -> anyhow::Result<f64> {
    let p: Vec<i64> = s[..10]
        .split('-')
        .map(|x| x.parse().map_err(|_| err("invalid ISO date")))
        .collect::<anyhow::Result<_>>()?;
    if p.len() != 3 {
        return Err(err("invalid ISO date"));
    }
    let (y, m, d) = (p[0], p[1], p[2]);
    let (y, m) = (y - (m <= 2) as i64, (m + 9) % 12);
    Ok((365 * y + y / 4 - y / 100 + y / 400 + (153 * m + 2) / 5 + d - 1) as f64)
}

fn template_dependencies(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = expr;
    while let Some(start) = rest.find('{') {
        let tail = &rest[start + 1..];
        let Some(end) = tail.find('}') else {
            break;
        };
        let name = tail[..end].trim();
        if valid_name(name) && !out.iter().any(|x: &String| x == name) {
            out.push(name.to_string());
        }
        rest = &tail[end + 1..];
    }
    out
}

pub fn dependencies(expr: &str) -> anyhow::Result<Vec<String>> {
    let mut set = HashSet::new();
    for t in lex(expr)? {
        if let Token::Ref(n) = t {
            set.insert(n);
        }
    }
    Ok(set.into_iter().collect())
}
pub fn validate_syntax(expr: &str) -> anyhow::Result<()> {
    let tokens = lex(expr)?;
    let mut depth = 0usize;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => {
                if depth == 0 {
                    return Err(err("unbalanced parentheses"));
                }
                depth -= 1;
            }
            Token::Id(name) => {
                let known = [
                    "if",
                    "coalesce",
                    "abs",
                    "round",
                    "floor",
                    "ceil",
                    "len",
                    "upper",
                    "lower",
                    "trim",
                    "contains",
                    "starts_with",
                    "ends_with",
                    "concat",
                    "year",
                    "month",
                    "day",
                    "date",
                    "days_between",
                    "true",
                    "false",
                    "null",
                ];
                if !known.iter().any(|x| x.eq_ignore_ascii_case(name))
                    && !matches!(tokens.get(i + 1), Some(Token::LParen))
                {
                    return Err(err(format!("unknown identifier: {name}")));
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(err("unbalanced parentheses"));
    }
    Ok(())
}
pub fn validate(expr: &str, fields: &HashSet<String>) -> anyhow::Result<()> {
    validate_syntax(expr)?;
    for name in dependencies(expr)? {
        if !fields.contains(&name) {
            return Err(err(format!("unknown formula field: {name}")));
        }
    }
    Ok(())
}

pub fn evaluate(expr: &str, payload: &HashMap<String, Value>) -> anyhow::Result<Value> {
    let tokens = lex(expr)?;
    let mut p = Parser {
        t: &tokens,
        p: 0,
        vars: payload,
        depth: 0,
    };
    p.parse()
}

pub fn apply(fields: &[Field], payload: &Value) -> anyhow::Result<Value> {
    let base = payload.as_object().cloned().unwrap_or_default();
    let names: HashSet<String> = fields.iter().map(|f| f.name.clone()).collect();
    let mut formulas: HashMap<String, String> = HashMap::new();
    let mut result = base.clone();
    let mut visiting = HashSet::new();
    let mut done = HashSet::new();
    for f in fields {
        if matches!(f.r#type.as_str(), "computed" | "formula") {
            let e = f.computed_expr.as_deref().unwrap_or("").trim();
            if e.is_empty() {
                return Err(err(format!(
                    "formula field requires an expression: {}",
                    f.name
                )));
            }
            if f.r#type == "computed" && (e.contains('[') || e.contains(']')) {
                let mut text = e.to_string();
                for dep in template_dependencies(e) {
                    text = text.replace(
                        &format!("{{{dep}}}"),
                        &display(base.get(&dep).unwrap_or(&Value::Null)),
                    );
                }
                result.insert(f.name.clone(), Value::String(text));
                done.insert(f.name.clone());
            } else {
                validate(e, &names)?;
                formulas.insert(f.name.clone(), e.to_string());
            }
        }
    }
    for name in formulas.keys().cloned().collect::<Vec<_>>() {
        resolve(
            &name,
            &formulas,
            &base,
            &mut result,
            &mut visiting,
            &mut done,
            0,
        )?;
    }
    Ok(Value::Object(result))
}
fn resolve(
    name: &str,
    formulas: &HashMap<String, String>,
    base: &serde_json::Map<String, Value>,
    result: &mut serde_json::Map<String, Value>,
    visiting: &mut HashSet<String>,
    done: &mut HashSet<String>,
    depth: usize,
) -> anyhow::Result<Value> {
    if depth > MAX_DEPTH {
        return Err(err("formula dependency depth exceeded"));
    }
    if let Some(v) = result.get(name) {
        if done.contains(name) {
            return Ok(v.clone());
        }
    }
    if !visiting.insert(name.to_string()) {
        return Err(err(format!("circular formula dependency: {name}")));
    }
    let expr = formulas
        .get(name)
        .ok_or_else(|| err(format!("unknown formula: {name}")))?;
    for dep in dependencies(expr)? {
        if formulas.contains_key(&dep) {
            let v = resolve(&dep, formulas, base, result, visiting, done, depth + 1)?;
            result.insert(dep, v);
        }
    }
    let vars: HashMap<String, Value> = result
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .chain(base.iter().map(|(k, v)| (k.clone(), v.clone())))
        .collect();
    let value = evaluate(expr, &vars)?;
    result.insert(name.to_string(), value.clone());
    visiting.remove(name);
    done.insert(name.to_string());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arithmetic_boolean() {
        let p = HashMap::from([("a".into(), val_num(4.0)), ("b".into(), val_num(2.0))]);
        assert_eq!(
            evaluate("({a}+{b})*2 >= 12 && {a} > {b}", &p).unwrap(),
            Value::Bool(true)
        );
    }
    #[test]
    fn strings_dates() {
        let p = HashMap::from([
            ("name".into(), Value::String("alice".into())),
            ("d".into(), Value::String("2026-09-09".into())),
        ]);
        assert_eq!(
            evaluate("upper({name})", &p).unwrap(),
            Value::String("ALICE".into())
        );
        assert_eq!(evaluate("year({d})", &p).unwrap(), val_num(2026.0));
    }
    #[test]
    fn dependency_and_cycle() {
        let fs = vec![
            field("a", "number", None),
            field("b", "formula", Some("{a}+2")),
            field("c", "computed", Some("{b}*3")),
        ];
        let v = apply(&fs, &serde_json::json!({"a":4})).unwrap();
        assert_eq!(v["c"], val_num(18.0));
        let cyc = vec![
            field("a", "formula", Some("{b}+1")),
            field("b", "formula", Some("{a}+1")),
        ];
        assert!(apply(&cyc, &serde_json::json!({})).is_err());
    }
    fn field(n: &str, t: &str, e: Option<&str>) -> Field {
        Field {
            id: n.into(),
            name: n.into(),
            label: n.into(),
            description: String::new(),
            r#type: t.into(),
            required: false,
            is_status: false,
            position: 0,
            ref_entity: None,
            computed_expr: e.map(str::to_string),
            is_unique: false,
            min_value: None,
            max_value: None,
            pattern: None,
            min_length: None,
            max_length: None,
            default_value: None,
            auto_number_prefix: None,
            auto_number_width: None,
            readonly: false,
            hidden: false,
            searchable: false,
            sortable: false,
            filterable: false,
            indexed: false,
            precision: None,
            help_text: String::new(),
            options: vec![],
        }
    }
}
