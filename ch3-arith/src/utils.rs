use std::fmt;

#[derive(Debug, Clone)]
pub enum Term {
    True,
    False,
    IfThenElse(Box<Term>, Box<Term>, Box<Term>),
    Zero,
    Succ(Box<Term>),
    Pred(Box<Term>),
    IsZero(Box<Term>),
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nature(usize),
}

use Term::*;
use Value::*;

pub fn eval(t: &Term) -> Result<Value, &'static str> {
    let v = match t {
        True => Bool(true),
        False => Bool(false),
        Zero => Nature(0),
        IfThenElse(t1, t2, t3) => {
            let v = eval(&t1)?;
            match v {
                Bool(true) => eval(&t2)?,
                Bool(false) => eval(&t3)?,
                _ => Err("non-bool value at if condition")?,
            }
        }
        Succ(t1) => {
            let v = eval(&t1)?;
            match v {
                Nature(nv) => Nature(nv + 1),
                _ => Err("non-number value after succ")?,
            }
        }
        Pred(t1) => {
            let v = eval(&t1)?;
            match v {
                Nature(0) => Nature(0),
                Nature(nv) => Nature(nv - 1),
                _ => Err("non-number value after pred")?,
            }
        }
        IsZero(t1) => {
            let v = eval(&t1)?;
            match v {
                Nature(0) => Bool(true),
                _ => Bool(false),
            }
        }
    };
    Ok(v)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bool(b) => write!(f, "{}", b),
            Nature(n) => write!(f, "{}", n),
        }
    }
}
