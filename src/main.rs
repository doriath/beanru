use beanru::types::{Currency, Ledger};
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;

/// Program for processing beancount files.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Normalizes beancount file
    ///
    /// It reads the beancount file, and then writes it in a standard format used by other
    /// subcommands.
    Normalize {
        input: String,

        #[arg(short, long, default_value_t = false)]
        in_place: bool,
    },
    /// Checks if all transactions are properly balanced.
    Check { input: String },
    /// Performs stock split.
    StockSplit {
        /// The path to beancount file.
        input: String,
        /// The commodity that is being split.
        commodity: String,
        /// The ratio of the split. For example, if set to 2, it means that every 1 share of the
        /// stock now becomes 2.
        ratio: rust_decimal::Decimal,
        #[arg(short, long, default_value_t = false)]
        in_place: bool,
    },
    Closing {
        /// The path to beancount file.
        input: String,
        #[arg(short, long, default_value_t = 15)]
        days: i64,
        #[arg(short, long, default_value_t = false)]
        in_place: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Normalize { input, in_place } => {
            let content = std::fs::read_to_string(&input)?;
            let beancount = beanru::parse(&content)?;
            if in_place {
                std::fs::write(&input, beancount.to_string())?;
            } else {
                println!("{}", beancount);
            }
        }
        Commands::Check { input } => {
            let project: Ledger<Decimal> =
                beanru::read(input, tokio::fs::read_to_string).await?;
            beanru::check(&project)?;
        }
        Commands::StockSplit {
            input,
            commodity,
            ratio,
            in_place,
        } => {
            let content = std::fs::read_to_string(&input)?;
            let mut beancount = beanru::parse(&content)?;
            beanru::split_stock(&mut beancount, &Currency(commodity), ratio)?;
            if in_place {
                std::fs::write(&input, beancount.to_string())?;
            } else {
                println!("{}", beancount);
            }
        }
        Commands::Closing {
            input,
            days,
            in_place,
        } => {
            let content = std::fs::read_to_string(&input)?;
            let mut beancount = beanru::parse(&content)?;
            beanru::closing(&mut beancount, days)?;
            if in_place {
                std::fs::write(&input, beancount.to_string())?;
            } else {
                println!("{}", beancount);
            }
        }
    }
    Ok(())
}
