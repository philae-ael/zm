#![feature(core_io_borrowed_buf, read_buf)]

use clap::Parser;

mod interpreter;
mod parser;
mod tokenizer;

#[derive(Debug, clap::Parser)]
struct Cli {
    filename: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let mut reader = args
        .filename
        .as_deref()
        .map(|filename| -> anyhow::Result<Box<dyn std::io::Read>> {
            Ok(std::fs::OpenOptions::new()
                .read(true)
                .open(filename)
                .map(Box::new)?)
        })
        .unwrap_or_else(|| Ok(Box::new(std::io::stdin().lock())))?;

    let mut toks = tokenizer::Tokenizer::new(&mut reader);
    let mut parser = parser::Parser::new(&mut toks);

    let ucode = parser.compile()?;
    interpreter::run(ucode);
    Ok(())
}
