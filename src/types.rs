use crate::bag::Bag;
use beancount_parser as parser;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub},
    path::{Path, PathBuf},
    str::FromStr,
};

/// A whole Ledger, containing multiple beancount files.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Ledger<D> {
    pub files: Vec<(PathBuf, BeancountFile<D>)>,
}

impl<D> Ledger<D> {
    /// Reads the beancount ledger, starting at given path and following all includes.
    ///
    /// It uses given read_to_string function to read the content at given path. To read from standard
    /// file system, tokio::fs::read_to_string can be used.
    pub async fn read<F, R>(
        start_path: impl AsRef<Path>,
        read_to_string: F,
    ) -> anyhow::Result<Ledger<D>>
    where
        D: Decimal,
        F: Fn(PathBuf) -> R,
        R: Future<Output = anyhow::Result<String>>,
    {
        let mut queue: Vec<PathBuf> = vec![start_path.as_ref().into()];
        let mut files: Vec<(PathBuf, BeancountFile<D>)> = Vec::new();
        // TODO: parallelize it (as we could be reading all files at once).
        while !queue.is_empty() {
            let p = queue.pop().unwrap();
            let b = parser::parse::<D>(&read_to_string(p.clone()).await?)?;
            for incl in &b.includes {
                let mut x = p.clone();
                x.pop();
                x.push(incl);
                queue.push(x);
            }
            files.push((p, b.into()));
        }
        Ok(Ledger { files })
    }

    /// Writes the beancount ledger, starting at given path and following all includes.
    ///
    /// It uses given read_to_string function to read the content at given path. To read from standard
    /// file system, tokio::fs::read_to_string can be used.
    pub async fn write<F, R>(&self, write: F) -> anyhow::Result<()>
    where
        D: Decimal,
        F: Fn(PathBuf, Vec<u8>) -> R,
        R: Future<Output = anyhow::Result<()>>,
    {
        // TODO: parallelize it
        for (path, file) in &self.files {
            write(path.clone(), file.to_string().into_bytes()).await?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct BeancountFile<D> {
    pub options: Vec<(String, String)>,
    pub includes: Vec<PathBuf>,
    pub directives: Vec<Directive<D>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Directive<D> {
    pub date: chrono::NaiveDate,
    pub content: DirectiveContent<D>,
    pub metadata: HashMap<String, MetadataValue<D>>,
}

// TODO: rename to Commodity
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency(pub String);

impl From<&str> for Currency {
    fn from(value: &str) -> Self {
        Currency(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValue<D> {
    String(String),
    Number(D),
    Currency(Currency),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pad {
    pub account: Account,
    pub source_account: Account,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Balance<D> {
    pub account: Account,
    pub amount: Amount<D>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction<D> {
    pub flag: Option<char>,
    pub payee: Option<String>,
    pub narration: Option<String>,
    pub tags: HashSet<String>,
    pub links: HashSet<String>,
    pub postings: Vec<Posting<D>>,
    pub balanced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting<D> {
    pub flag: Option<char>,
    pub account: Account,
    pub amount: Option<Amount<D>>,
    pub cost: Option<Cost<D>>,
    pub price: Option<PostingPrice<D>>,
    pub metadata: HashMap<String, MetadataValue<D>>,
    // True if the amount is autocomputed.
    pub autocomputed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost<D> {
    pub amount: Option<Amount<D>>,
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostingPrice<D> {
    Unit(Amount<D>),
    Total(Amount<D>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveContent<D> {
    Balance(Balance<D>),
    Close(Close),
    Commodity(Currency),
    Event(Event),
    Open(Open),
    Pad(Pad),
    Price(Price<D>),
    Transaction(Transaction<D>),
}

impl<D> DirectiveContent<D> {
    pub fn transaction_opt(&self) -> Option<&Transaction<D>> {
        match self {
            DirectiveContent::Transaction(t) => Some(t),
            _ => None,
        }
    }

    pub fn open_opt(&self) -> Option<&Open> {
        match self {
            DirectiveContent::Open(o) => Some(o),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price<D> {
    pub currency: Currency,
    pub amount: Amount<D>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Amount<D> {
    pub value: D,
    pub currency: Currency,
}

impl<D> Amount<D>
where
    D: Decimal,
{
    pub fn abs(mut self) -> Self {
        if self.value < 0.into() {
            self.value = -self.value;
        }
        self
    }
}

impl<D> Neg for Amount<D>
where
    D: Decimal,
{
    type Output = Amount<D>;

    fn neg(self) -> Self::Output {
        Amount {
            value: -self.value,
            currency: self.currency,
        }
    }
}

impl<'a, D> Neg for &'a Amount<D>
where
    D: Decimal,
{
    type Output = Amount<D>;

    fn neg(self) -> Self::Output {
        Amount {
            value: -self.value.clone(),
            currency: self.currency.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Open {
    pub account: Account,
    pub currencies: HashSet<Currency>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Close {
    pub account: Account,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account(pub String);

pub trait Decimal:
    Clone
    + Default
    + Debug
    + Display
    + Hash
    + From<i32>
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + PartialEq
    + Eq
    + PartialOrd
    + FromStr
{
}

impl<D> Decimal for D where
    D: Clone
        + Default
        + Debug
        + Display
        + Hash
        + From<i32>
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + PartialEq
        + Eq
        + PartialOrd
        + FromStr
{
}

impl<D> Transaction<D>
where
    D: Decimal,
{
    pub fn book(&mut self) -> anyhow::Result<()> {
        let mut bag = Bag::new();
        let mut postings_no_amount: Vec<&mut Posting<D>> = Vec::new();
        for posting in &mut self.postings {
            match posting_amount_to_balance(posting) {
                Some(amount) => {
                    bag += amount;
                }
                None => postings_no_amount.push(posting),
            };
        }
        anyhow::ensure!(
            postings_no_amount.len() <= 1,
            "more than one posting without amount"
        );
        let non_zero_amounts = bag
            .currencies
            .iter()
            .filter(|x| *x.1 != 0.into())
            .collect::<Vec<_>>();
        anyhow::ensure!(
            non_zero_amounts.len() <= 1,
            "more than one currency does not balance to 0"
        );

        if postings_no_amount.len() == 1 {
            if non_zero_amounts.is_empty() {
                anyhow::bail!("posting with no amount in transaction that is balanced");
            }
            postings_no_amount[0].amount = Some(Amount {
                value: -non_zero_amounts[0].1.clone(),
                currency: non_zero_amounts[0].0.clone(),
            });
            postings_no_amount[0].autocomputed = true;
        } else if !non_zero_amounts.is_empty() {
            anyhow::bail!("transaction does not balance to 0");
        }
        Ok(())
    }
}

pub(crate) fn posting_amount_to_balance<D: Decimal>(posting: &Posting<D>) -> Option<Amount<D>> {
    let amount = match &posting.amount {
        Some(amount) => amount,
        None => return None,
    };
    // TODO: consider using Neg crate to perform the negation.
    let sign: D = if amount.value > 0.into() {
        1.into()
    } else {
        (-1).into()
    };
    if let Some(cost) = &posting.cost {
        if let Some(cost_amount) = &cost.amount {
            return Some(Amount {
                // TODO: figure out a way to avoid doing clone here.
                value: amount.value.clone() * cost_amount.value.clone(),
                currency: cost_amount.currency.clone(),
            });
        }
    }
    match &posting.price {
        Some(PostingPrice::Unit(price_amount)) => {
            return Some(Amount {
                value: amount.value.clone() * price_amount.value.clone(),
                currency: price_amount.currency.clone(),
            })
        }
        Some(PostingPrice::Total(price_amount)) => {
            return Some(Amount {
                value: price_amount.value.clone() * sign,
                currency: price_amount.currency.clone(),
            })
        }
        _ => (),
    }
    Some(amount.clone())
}
