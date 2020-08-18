mod utils;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(arith);

use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

use std::io;
use std::path::PathBuf;

use utils::*;

#[derive(Debug, StructOpt)]
#[structopt(author, about = "Rust implemention of arith")]
struct Opt {
    /// Activate interactive mode
    #[structopt(short)]
    interactive: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    if let Some(ref input_path) = opt.input {
        let parser = arith::TermsParser::new();
        let input = std::fs::read_to_string(input_path)?;
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
        if opt.interactive {
            repl();
        }
    } else {
        repl();
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
