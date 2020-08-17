#![allow(unused_imports)]

mod utils;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(untyped);

use anyhow::{format_err, Context as _};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use utils::*;
use Binder::*;

#[derive(Debug, StructOpt)]
#[structopt(author, about = "Rust implemention of untyped lambda-calculus")]
struct Opt {
    /// Activate interactive mode
    #[structopt(short)]
    interactive: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let mut ctx = Context::new();
    if let Some(ref input_path) = opt.input {
        let parser = untyped::CommandsParser::new();
        let input = std::fs::read_to_string(input_path)?;
        match parser.parse(&input) {
            Ok(commands) => {
                for cmd in commands {
                    if let Err(e) = do_command(cmd, &mut ctx) {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(e) => eprintln!("Parse Error: {}", e),
        }
        if opt.interactive {
            repl(&mut ctx);
        }
    } else {
        repl(&mut ctx);
    }
    Ok(())
}

fn repl(ctx: &mut Context) {
    let mut rl = Editor::<()>::new();
    let parser = untyped::REPLCommandParser::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                rl.add_history_entry(&line);
                match parser.parse(&line) {
                    Ok(cmd) => {
                        if let Err(e) = do_command(cmd, ctx) {
                            eprintln!("Error: {}", e);
                        }
                        //println!("context: {:?}", ctx);
                    }
                    Err(e) => eprintln!("Parse Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}

fn do_command(cmd: Command, ctx: &mut Context) -> anyhow::Result<()> {
    match cmd {
        Command::SymbolBind(varname) => {
            println!("{} is bounded", varname);
            *ctx = Context::cons(&varname, &NameBind, &ctx);
        }
        Command::Bind(varname, ast) => {
            let t = make_dbi(&ast, &ctx)?;
            let result = eval(&t, &ctx)?;
            println!("{} <- {}", varname, pw(&result, &ctx));
            *ctx = Context::cons(&varname, &TermBind(Rc::new(result)), &ctx);
        }
        Command::Eval(ast) => {
            let t = make_dbi(&ast, &ctx)?;
            let result = eval(&t, &ctx)?;
            println!("{}", pw(&result, &ctx));
        }
    }
    Ok(())
}
