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
    Normalize { input: String },
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
    },
    Closing {
        /// The path to beancount file.
        input: String,
        #[arg(short, long, default_value_t = 15)]
        days: i64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Normalize { input } => {
            let ledger: Ledger<Decimal> = Ledger::read(input, tokio::fs::read_to_string).await?;
            ledger.write(tokio::fs::write).await?;
        }
        Commands::Check { input } => {
            let ledger: Ledger<Decimal> = Ledger::read(input, tokio::fs::read_to_string).await?;
            beanru::check(&ledger)?;
        }
        Commands::StockSplit {
            input,
            commodity,
            ratio,
        } => {
            let mut ledger: Ledger<Decimal> =
                Ledger::read(input, tokio::fs::read_to_string).await?;
            beanru::split_stock(&mut ledger, &Currency(commodity), ratio)?;
            ledger.write(tokio::fs::write).await?;
        }
        Commands::Closing { input, days } => {
            let mut ledger: Ledger<Decimal> =
                Ledger::read(input, tokio::fs::read_to_string).await?;
            beanru::closing(&mut ledger, days)?;
            ledger.write(tokio::fs::write).await?;
        }
    }
    Ok(())
}
