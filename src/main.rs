use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use panscale::recipe;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scale a recipe and print it
    Scale {
        /// Recipe file, or stdin when left out
        file: Option<PathBuf>,
        /// Multiply every amount by this
        #[arg(long)]
        by: f64,
    },
}

fn read(file: Option<&PathBuf>) -> Result<String> {
    match file {
        Some(p) => std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display())),
        None => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).context("reading stdin")?;
            Ok(s)
        }
    }
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Scale { file, by } => {
            if !(by > 0.0 && by.is_finite()) {
                bail!("--by has to be a positive number");
            }
            print!("{}", recipe::scale(&read(file.as_ref())?, by));
        }
    }
    Ok(())
}
