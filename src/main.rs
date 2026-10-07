use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use panscale::{pan, recipe};

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
        #[arg(long, conflicts_with_all = ["serves", "pan"], required_unless_present_any = ["serves", "pan"])]
        by: Option<f64>,
        /// Scale to feed this many, using the recipe's own "Serves N" line
        #[arg(long, conflicts_with = "pan")]
        serves: Option<f64>,
        /// The pan you have, like "9x13" or "23cm round"
        #[arg(long, requires = "from_pan")]
        pan: Option<String>,
        /// The pan the recipe was written for
        #[arg(long)]
        from_pan: Option<String>,
    },
    /// Show how much bigger or smaller one pan is than another
    Pan {
        /// The pan the recipe was written for
        from: String,
        /// The pan you have
        to: String,
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

fn pan_factor(from: &str, to: &str) -> Result<f64> {
    let from = pan::parse_pan(from).map_err(anyhow::Error::msg)?;
    let to = pan::parse_pan(to).map_err(anyhow::Error::msg)?;
    Ok(pan::factor(from, to))
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Scale { file, by, serves, pan: to, from_pan } => {
            let text = read(file.as_ref())?;
            let k = match (by, serves, to, from_pan) {
                (Some(k), ..) => k,
                (None, None, Some(to), Some(from)) => pan_factor(&from, &to)?,
                (None, Some(want), ..) => {
                    let Some(have) = recipe::servings(&text) else {
                        bail!("the recipe doesn't say how many it serves; use --by instead");
                    };
                    want / have
                }
                _ => unreachable!("clap requires one of them"),
            };
            if !(k > 0.0 && k.is_finite()) {
                bail!("the scale has to be a positive number, not {k}");
            }
            print!("{}", recipe::scale(&text, k));
        }
        Cmd::Pan { from, to } => {
            let k = pan_factor(&from, &to)?;
            println!("{to} holds {:.2} times as much as {from}.", k);
        }
    }
    Ok(())
}
