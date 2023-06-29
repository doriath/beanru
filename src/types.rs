use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub},
};

#[derive(Debug, PartialEq, Eq)]
pub struct BeancountFile<D> {
    pub directives: Vec<Directive<D>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Directive<D> {
    pub date: chrono::NaiveDate,
    pub content: DirectiveContent<D>,
    pub metadata: HashMap<String, MetadataValue<D>>,
}

// TODO: rename to Commodifty
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency(pub String);

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
{
}

impl<D> Decimal for D where
    D: Clone
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
{
}

impl<D> Transaction<D>
where
    D: Decimal,
{
    // TODO: Consider returning error when the postings withing transactions do not balance.
    // assert!(no_amount_count <= 1, "more than one posting without amount");
    pub fn book(&mut self) -> anyhow::Result<()> {
        let mut amounts: HashMap<Currency, D> = HashMap::new();
        let mut postings_no_amount: Vec<&mut Posting<D>> = Vec::new();
        for posting in &mut self.postings {
            match posting_amount_to_balance(posting) {
                Some(amount) => {
                    *amounts.entry(amount.currency.clone()).or_insert(0.into()) += amount.value
                }
                None => postings_no_amount.push(posting),
            };
        }
        anyhow::ensure!(
            postings_no_amount.len() <= 1,
            "more than one posting without amount"
        );
        let non_zero_amounts = amounts
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
