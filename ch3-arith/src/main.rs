mod utils;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(arith);

use atty::Stream;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::io::{self, prelude::*};

use utils::*;

fn main() -> io::Result<()> {
    if atty::is(Stream::Stdin) {
        repl();
    } else {
        let parser = arith::TermsParser::new();
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        match parser.parse(&input) {
            Ok(terms) => {
                for term in terms {
                    match eval(&term) {
                        Ok(v) => println!("{}", v),
                        Err(e) => eprintln!("Eval error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}

fn repl() {
    let mut rl = Editor::<()>::new();
    let parser = arith::TermParser::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                match parser.parse(&line) {
                    Ok(term) => match eval(&term) {
                        Ok(v) => println!("{}", v),
                        Err(e) => eprintln!("Eval Error: {}", e),
                    },
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
