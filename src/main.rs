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
        #[arg(long, conflicts_with = "serves", required_unless_present = "serves")]
        by: Option<f64>,
        /// Scale to feed this many, using the recipe's own "Serves N" line
        #[arg(long)]
        serves: Option<f64>,
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
        Cmd::Scale { file, by, serves } => {
            let text = read(file.as_ref())?;
            let k = match (by, serves) {
                (Some(k), _) => k,
                (None, Some(want)) => {
                    let Some(have) = recipe::servings(&text) else {
                        bail!("the recipe doesn't say how many it serves; use --by instead");
                    };
                    want / have
                }
                (None, None) => unreachable!("clap requires one of them"),
            };
            if !(k > 0.0 && k.is_finite()) {
                bail!("the scale has to be a positive number, not {k}");
            }
            print!("{}", recipe::scale(&text, k));
        }
    }
    Ok(())
}
