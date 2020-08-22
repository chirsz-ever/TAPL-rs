use anyhow::{format_err, Context as _};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Ast {
    ID(Rc<str>),
    Abs { var: Rc<str>, expr: Rc<Ast> },
    App(Rc<Ast>, Rc<Ast>),
}

#[derive(Debug, Clone)]
pub enum Term {
    Var(usize),
    Abs { var: Rc<str>, expr: Rc<Term> },
    App(Rc<Term>, Rc<Term>),
}

#[derive(Debug, Clone)]
pub enum Binder {
    NameBind,
    TermBind(Rc<Term>),
}

#[derive(Debug)]
pub enum Command {
    Eval(Ast),
    SymbolBind(Rc<str>),
    Bind(Rc<str>, Ast),
}

#[derive(Debug, Clone)]
pub struct CtxNode {
    pub var: Rc<str>,
    pub bind: Binder,
    pub next: Option<Rc<CtxNode>>,
}

#[derive(Debug, Clone)]
pub struct Context(Option<Rc<CtxNode>>);

impl Context {
    pub fn cons(var: &Rc<str>, bind: &Binder, ctx: &Context) -> Context {
        Context(Some(Rc::new(CtxNode {
            var: var.clone(),
            bind: bind.clone(),
            next: ctx.0.clone(),
        })))
    }

    pub fn new() -> Context {
        Context(None)
    }
}

use Binder::*;
use Term::*;

fn name2index(name: &str, ctx: &Context) -> Option<usize> {
    let mut pctxn = &ctx.0;
    let mut idx = 0;
    while let Some(ctxn) = pctxn {
        if ctxn.var.deref() == name {
            return Some(idx);
        }
        pctxn = &ctxn.next;
        idx += 1;
    }
    None
}

pub fn make_dbi(ast: &Ast, ctx: &Context) -> anyhow::Result<Term> {
    let ret = match ast {
        Ast::ID(varname) => {
            let idx = name2index(&varname, ctx)
                .with_context(|| format!("can't find bound to {:?}", varname))?;
            Term::Var(idx)
        }
        Ast::Abs { var, expr } => {
            let texpr = make_dbi(&expr, &Context::cons(var, &NameBind, ctx))?;
            Term::Abs {
                var: var.clone(),
                expr: Rc::new(texpr),
            }
        }
        Ast::App(a1, a2) => {
            let t1 = make_dbi(a1, ctx)?;
            let t2 = make_dbi(a2, ctx)?;
            Term::App(Rc::new(t1), Rc::new(t2))
        }
    };
    Ok(ret)
}

impl Term {
    pub fn abs(var: &Rc<str>, expr: Term) -> Term {
        Term::Abs {
            var: var.clone(),
            expr: Rc::new(expr),
        }
    }

    pub fn app(t1: Term, t2: Term) -> Term {
        Term::App(Rc::new(t1), Rc::new(t2))
    }

    fn map_ref(&self, c: usize, convar: &impl Fn(usize, usize) -> Term) -> Term {
        match self {
            Abs { var, expr } => Term::abs(var, expr.map_ref(c + 1, convar)),
            App(t1, t2) => Term::app(t1.map_ref(c, convar), t2.map_ref(c, convar)),
            Var(idx) => convar(c, *idx),
        }
    }

    fn map(&self, c: usize, convar: impl Fn(usize, usize) -> Term) -> Term {
        self.map_ref(c, &convar)
    }

    #[inline]
    pub fn shift(&self, d: usize) -> Term {
        if d == 0 {
            return self.clone();
        }
        self.map(0, move |c, idx| {
            if idx >= c {
                Term::Var(idx + d)
            } else {
                Term::Var(idx)
            }
        })
    }

    // [j->s]self
    #[inline]
    pub fn subst(&self, j: usize, s: &Term) -> Term {
        self.map(0, move |c, idx| {
            if idx == j + c {
                s.shift(c)
            } else {
                Term::Var(idx)
            }
        })
    }

    // pub fn subst_top1(&self, v: &Term) -> Term {
    //     // shift(-1, subst(0, shift(1, v), t))
    //     let t1 = self.subst(0, &v.shift(1));
    //     self.map(0, move |c, idx| {
    //         if idx >= c {
    //             Term::Var(idx - 1)
    //         } else {
    //             Term::Var(idx)
    //         }
    //     })
    // }

    #[inline]
    pub fn subst_top(&self, v: &Term) -> Term {
        self.map(0, move |c, idx| {
            if idx == c {
                v.shift(c)
            } else if idx > c {
                Term::Var(idx - 1)
            } else {
                Term::Var(idx)
            }
        })
    }
}

fn index2bind(ctx: &Context, mut n: usize) -> Option<(Rc<str>, Binder)> {
    let mut pctxn = &ctx.0;
    while let Some(ctxn) = pctxn {
        if n == 0 {
            return Some((ctxn.var.clone(), ctxn.bind.clone()));
        }
        pctxn = &ctxn.next;
        n -= 1;
    }
    None
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct pw<'a, 'b>(pub &'a Term, pub &'b Context);

fn write_var(f: &mut fmt::Formatter<'_>, idx: usize, ctx: &Context) -> fmt::Result {
    match index2bind(ctx, idx) {
        Some((varname, _)) => write!(f, "{}", varname),
        None => write!(f, "[bad index {}]", idx),
    }
}

fn get_freshname(var: &str, ctx: &Context) -> Rc<str> {
    match name2index(var, ctx) {
        Some(_) => get_freshname(&format!("{}'", var), ctx),
        None => format!("{}", var).into(),
    }
}

impl fmt::Display for pw<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pw(t, ctx) = self;
        match t {
            Var(idx) => write_var(f, *idx, ctx),
            Abs { var, expr } => {
                let freshname = get_freshname(&var, ctx);
                write!(
                    f,
                    "λ{}. {}",
                    freshname,
                    pw(expr, &Context::cons(&freshname, &NameBind, ctx))
                )
            }
            App(t1, t2) => {
                match t1.deref() {
                    Var(..) | App(_, _) => write!(f, "{}", pw(t1, ctx))?,
                    Abs { .. } => write!(f, "({})", pw(t1, ctx))?,
                }
                write!(f, " ")?;
                match t2.deref() {
                    Var(idx) => write_var(f, *idx, ctx),
                    Abs { .. } | App(_, _) => write!(f, "({})", pw(&t2, ctx)),
                }
            }
        }
    }
}

pub fn eval(t: &Term, ctx: &Context) -> anyhow::Result<Term> {
    let ret = match t {
        Var(idx) => match index2bind(ctx, *idx) {
            Some((_, NameBind)) => Var(*idx),
            Some((_, TermBind(bind))) => bind.shift(idx + 1),
            None => return Err(format_err!("Bad index {}", idx)),
        },
        Abs { .. } => t.clone(),
        App(t1, t2) => match eval(&t1, ctx)? {
            Abs { var: _, expr } => {
                let t2_1 = eval(&t2, ctx)?;
                eval(&expr.subst_top(&t2_1), ctx)?
            }
            t1_1 => Term::app(t1_1, Term::clone(&t2)),
        },
    };
    Ok(ret)
}
