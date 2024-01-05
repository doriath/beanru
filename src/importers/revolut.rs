//! Beancount Importer for Revolut.
//!
//! How to import:
//! 1. Login to Revolut (Mobile App)
//! 2. Click 3 dots (More)
//! 3. Click `Statement`
//! 4. Select `Excel`
//! 5. Click `Get statement`
//! 6. Save to cloud storage (e.g. Google Drive)
//! 7. Repeat for every account (currenc)
//! 8. Download all statements from cloud storage locally and run the importer.
//!
//! Sample use:
//!
//! fn main() {
//!   let mut bank = PathBuf::from(std::env::var("HOME").unwrap());
//!   let mut revolut = bank.clone();
//!   revolut.push("revolut");
//!   for file in std::fs::read_dir(revolut)? {
//!       let imported = revolut::import(&file?.path(), "Assets:CH:Revolut", "Expenses:CH:Revolut:Fees")?;
//!       println!("{}", imported);
//!   }
//! }
use crate::types::Currency;
use crate::types::{
    Account, Amount, BeancountFile, Directive, DirectiveContent, Posting, Transaction,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{collections::HashSet, path::Path};

#[derive(Deserialize)]
pub struct Record {
    // #[serde(rename(deserialize = "Type"))]
    // typ: String,
    // #[serde(rename(deserialize = "Product"))]
    // product: String,
    #[serde(rename(deserialize = "Started Date"))]
    pub start_date: String,
    #[serde(rename(deserialize = "Completed Date"))]
    pub completed_date: String,
    #[serde(rename(deserialize = "Description"))]
    pub description: String,
    #[serde(rename(deserialize = "Amount"))]
    pub amount: String,
    #[serde(rename(deserialize = "Fee"))]
    pub fee: String,
    #[serde(rename(deserialize = "Currency"))]
    pub currency: String,
    #[serde(rename(deserialize = "State"))]
    pub state: String,
    // #[serde(rename(deserialize = "Balance"))]
    // balance: String,
}

pub fn import(
    path: &Path,
    account_prefix: &str,
    fee_account: &str,
) -> anyhow::Result<BeancountFile<Decimal>> {
    let mut file = BeancountFile::default();
    let mut rdr = csv::Reader::from_path(path)?;
    for record in rdr.deserialize() {
        let record: Record = record?;
        if record.state != "COMPLETED" {
            continue;
        }
        let currency = Currency(record.currency.clone());
        let account = Account(format!("{}:{}", account_prefix, record.currency));
        let expenses_account = Account(fee_account.into());
        let (date, _) = chrono::NaiveDate::parse_and_remainder(&record.start_date, "%Y-%m-%d")?;
        let id = format!(
            "id-revolut-{:x}",
            md5::compute(format!(
                "{}-{}-{}-{}-{}",
                record.start_date,
                record.completed_date,
                record.description,
                record.amount,
                record.currency,
            ))
        );

        let mut postings = vec![Posting {
            account,
            amount: Some(Amount {
                value: Decimal::from_str(&record.amount).unwrap()
                    - Decimal::from_str(&record.fee).unwrap(),
                currency: currency.clone(),
            }),
            flag: None,
            cost: None,
            price: None,
            metadata: Default::default(),
            autocomputed: false,
        }];
        if record.fee != "0.00" {
            postings.push(Posting {
                account: expenses_account,
                flag: None,
                amount: Some(Amount {
                    value: Decimal::from_str(&record.fee).unwrap(),
                    currency: currency.clone(),
                }),
                cost: None,
                price: None,
                metadata: Default::default(),
                autocomputed: false,
            });
        }

        file.directives.push(Directive {
            date,
            content: DirectiveContent::Transaction(Transaction {
                narration: Some(record.description),
                links: HashSet::from([id.clone()]),
                postings,
                flag: None,
                payee: None,
                tags: Default::default(),
                balanced: false,
            }),
            metadata: Default::default(),
        });
    }
    Ok(file)
}
