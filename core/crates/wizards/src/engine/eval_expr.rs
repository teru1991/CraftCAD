use crate::reasons::{WizardReason, WizardReasonCode};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn tokenize(expr: &str) -> Result<Vec<Tok>, WizardReason> {
    let mut toks = vec![];
    let mut i = 0usize;
    let chars: Vec<char> = expr.chars().collect();
    if chars.len() > 256 {
        return Err(WizardReason::new(
            WizardReasonCode::WizardDslInvalid,
            "expr too long",
        ));
    }
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Mul);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Div);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            _ => {
                if c.is_ascii_digit() || c == '.' {
                    let start = i;
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    let n: f64 = s.parse().map_err(|_| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            format!("invalid number: {s}"),
                        )
                    })?;
                    toks.push(Tok::Num(n));
                } else if is_ident_start(c) {
                    let start = i;
                    i += 1;
                    while i < chars.len() && is_ident(chars[i]) {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    toks.push(Tok::Ident(s));
                } else {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        format!("invalid char in expr: {c}"),
                    ));
                }
            }
        }
        if toks.len() > 128 {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                "expr too many tokens",
            ));
        }
    }
    Ok(toks)
}

fn to_rpn(toks: &[Tok]) -> Result<Vec<Tok>, WizardReason> {
    let mut out = vec![];
    let mut st: Vec<Tok> = vec![];

    fn prec(t: &Tok) -> i32 {
        match t {
            Tok::Plus | Tok::Minus => 1,
            Tok::Mul | Tok::Div => 2,
            _ => 0,
        }
    }

    for t in toks {
        match t {
            Tok::Num(_) | Tok::Ident(_) => out.push(t.clone()),
            Tok::Plus | Tok::Minus | Tok::Mul | Tok::Div => {
                while let Some(top) = st.last() {
                    let is_op = matches!(top, Tok::Plus | Tok::Minus | Tok::Mul | Tok::Div);
                    if is_op && prec(top) >= prec(t) {
                        out.push(st.pop().expect("operator present"));
                    } else {
                        break;
                    }
                }
                st.push(t.clone());
            }
            Tok::LParen => st.push(Tok::LParen),
            Tok::RParen => {
                let mut found = false;
                while let Some(top) = st.pop() {
                    if matches!(top, Tok::LParen) {
                        found = true;
                        break;
                    }
                    out.push(top);
                }
                if !found {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        "unbalanced parentheses",
                    ));
                }
            }
        }
        if out.len() > 256 {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                "expr rpn too large",
            ));
        }
    }

    while let Some(t) = st.pop() {
        if matches!(t, Tok::LParen | Tok::RParen) {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                "unbalanced parentheses",
            ));
        }
        out.push(t);
    }

    Ok(out)
}

pub fn eval_number_expr(expr: &str, inputs: &BTreeMap<String, Value>) -> Result<f64, WizardReason> {
    let toks = tokenize(expr)?;
    let rpn = to_rpn(&toks)?;
    let mut st: Vec<f64> = vec![];

    for t in rpn {
        match t {
            Tok::Num(n) => st.push(n),
            Tok::Ident(k) => {
                let v = inputs.get(&k).and_then(|v| v.as_f64()).ok_or_else(|| {
                    WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        format!("unknown identifier in expr: {k}"),
                    )
                })?;
                st.push(v);
            }
            Tok::Plus => {
                let (a, b) = pop2(&mut st)?;
                st.push(a + b);
            }
            Tok::Minus => {
                let (a, b) = pop2(&mut st)?;
                st.push(a - b);
            }
            Tok::Mul => {
                let (a, b) = pop2(&mut st)?;
                st.push(a * b);
            }
            Tok::Div => {
                let (a, b) = pop2(&mut st)?;
                if b == 0.0 {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        "division by zero",
                    ));
                }
                st.push(a / b);
            }
            Tok::LParen | Tok::RParen => unreachable!(),
        }
        if st.len() > 64 {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                "expr stack overflow",
            ));
        }
    }

    if st.len() != 1 {
        return Err(WizardReason::new(
            WizardReasonCode::WizardDslInvalid,
            "expr did not reduce to single value",
        ));
    }

    Ok(st[0])
}

fn pop2(st: &mut Vec<f64>) -> Result<(f64, f64), WizardReason> {
    let b = st.pop().ok_or_else(|| {
        WizardReason::new(WizardReasonCode::WizardDslInvalid, "expr stack underflow")
    })?;
    let a = st.pop().ok_or_else(|| {
        WizardReason::new(WizardReasonCode::WizardDslInvalid, "expr stack underflow")
    })?;
    Ok((a, b))
}
