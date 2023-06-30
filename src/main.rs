use bean::types::Currency;
use clap::{Parser, Subcommand};

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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Normalize { input, in_place } => {
            let content = std::fs::read_to_string(&input)?;
            let beancount = bean::parse(&content)?;
            if in_place {
                std::fs::write(&input, beancount.to_string())?;
            } else {
                println!("{}", beancount);
            }
        }
        Commands::Check { input } => {
            let content = std::fs::read_to_string(input)?;
            let beancount = bean::parse(&content)?;
            bean::check(&beancount)?;
        }
        Commands::StockSplit {
            input,
            commodity,
            ratio,
            in_place,
        } => {
            let content = std::fs::read_to_string(&input)?;
            let mut beancount = bean::parse(&content)?;
            bean::split_stock(&mut beancount, &Currency(commodity), ratio)?;
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
            let mut beancount = bean::parse(&content)?;
            bean::closing(&mut beancount, days)?;
            if in_place {
                std::fs::write(&input, beancount.to_string())?;
            } else {
                println!("{}", beancount);
            }
        }
    }
    Ok(())
}
