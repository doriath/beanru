use beancount_parser_2 as parser;
use std::collections::{HashMap, HashSet};

mod format;

#[derive(Debug)]
pub struct BeancountFile<D> {
    pub directives: Vec<Directive<D>>,
}

#[derive(Debug)]
pub struct Directive<D> {
    pub date: parser::Date,
    pub content: DirectiveContent<D>,
    pub metadata: HashMap<String, MetadataValue<D>>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency(String);

#[derive(Debug)]
pub enum MetadataValue<D> {
    String(String),
    Number(D),
    Currency(Currency),
}

#[derive(Debug)]
pub struct Pad {
    pub account: Account,
    pub source_account: Account,
}

#[derive(Debug)]
pub struct Balance<D> {
    pub account: Account,
    pub amount: Amount<D>,
}

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Transaction<D> {
    pub flag: Option<parser::Flag>,
    pub payee: Option<String>,
    pub narration: Option<String>,
    pub tags: HashSet<String>,
    pub links: HashSet<String>,
    pub postings: Vec<Posting<D>>,
}

#[derive(Debug)]
pub struct Posting<D> {
    pub flag: Option<parser::Flag>,
    pub account: Account,
    pub amount: Option<Amount<D>>,
    pub cost: Option<Cost<D>>,
    pub price: Option<PostingPrice<D>>,
    pub metadata: HashMap<String, MetadataValue<D>>,
}

#[derive(Debug)]
pub struct Cost<D> {
    pub amount: Option<Amount<D>>,
    pub date: Option<parser::Date>,
}

#[derive(Debug)]
pub enum PostingPrice<D> {
    Unit(Amount<D>),
    Total(Amount<D>),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Price<D> {
    pub currency: Currency,
    pub amount: Amount<D>,
}

#[derive(Debug)]
pub struct Amount<D> {
    pub value: D,
    pub currency: Currency,
}

#[derive(Debug)]
pub struct Open {
    pub account: Account,
    pub currencies: HashSet<Currency>,
}

#[derive(Debug)]
pub struct Close {
    pub account: Account,
}

#[derive(Debug)]
pub struct Account(String);

impl<D> From<parser::BeancountFile<'_, D>> for BeancountFile<D> {
    fn from(f: parser::BeancountFile<'_, D>) -> Self {
        BeancountFile {
            directives: f.directives.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl<D> From<parser::Directive<'_, D>> for Directive<D> {
    fn from(d: parser::Directive<'_, D>) -> Self {
        Directive {
            date: d.date,
            content: d.content.into(),
            metadata: d
                .metadata
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.into()))
                .collect(),
        }
    }
}

impl<D> From<parser::MetadataValue<'_, D>> for MetadataValue<D> {
    fn from(v: parser::MetadataValue<'_, D>) -> Self {
        match v {
            parser::MetadataValue::String(x) => MetadataValue::String(x.to_owned()),
            parser::MetadataValue::Number(x) => MetadataValue::Number(x),
            parser::MetadataValue::Currency(x) => MetadataValue::Currency(x.into()),
            _ => unimplemented!("given metadata value type is not supported yet"),
        }
    }
}

impl<D> From<parser::DirectiveContent<'_, D>> for DirectiveContent<D> {
    fn from(v: parser::DirectiveContent<'_, D>) -> Self {
        match v {
            parser::DirectiveContent::Balance(x) => DirectiveContent::Balance(x.into()),
            parser::DirectiveContent::Close(x) => DirectiveContent::Close(x.into()),
            parser::DirectiveContent::Commodity(x) => DirectiveContent::Commodity(x.into()),
            parser::DirectiveContent::Open(x) => DirectiveContent::Open(x.into()),
            parser::DirectiveContent::Pad(x) => DirectiveContent::Pad(x.into()),
            parser::DirectiveContent::Price(x) => DirectiveContent::Price(x.into()),
            parser::DirectiveContent::Transaction(x) => DirectiveContent::Transaction(x.into()),
            _ => unimplemented!(),
        }
    }
}

impl From<parser::Pad<'_>> for Pad {
    fn from(v: parser::Pad<'_>) -> Self {
        Self {
            account: v.account.into(),
            source_account: v.source_account.into(),
        }
    }
}

impl From<parser::Open<'_>> for Open {
    fn from(v: parser::Open<'_>) -> Self {
        Self {
            account: v.account.into(),
            currencies: v.currencies.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<parser::Event<'_>> for Event {
    fn from(v: parser::Event<'_>) -> Self {
        Self {
            name: v.name.to_owned(),
            value: v.value.to_owned(),
        }
    }
}

impl From<parser::Close<'_>> for Close {
    fn from(v: parser::Close<'_>) -> Self {
        Self {
            account: v.account.into(),
        }
    }
}

impl From<parser::Account<'_>> for Account {
    fn from(v: parser::Account<'_>) -> Self {
        Account(v.as_str().to_owned())
    }
}

impl<D> From<parser::Price<'_, D>> for Price<D> {
    fn from(v: parser::Price<'_, D>) -> Self {
        Self {
            currency: v.currency.into(),
            amount: v.amount.into(),
        }
    }
}

impl<D> From<parser::Balance<'_, D>> for Balance<D> {
    fn from(v: parser::Balance<'_, D>) -> Self {
        Self {
            account: v.account.into(),
            amount: v.amount.into(),
        }
    }
}

impl<D> From<parser::Transaction<'_, D>> for Transaction<D> {
    fn from(v: parser::Transaction<'_, D>) -> Self {
        Self {
            flag: v.flag,
            payee: v.payee.map(String::from),
            narration: v.narration.map(String::from),
            tags: v.tags.into_iter().map(|x| x.into()).collect(),
            links: v.links.into_iter().map(|x| x.into()).collect(),
            postings: v.postings.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl<D> From<parser::Posting<'_, D>> for Posting<D> {
    fn from(v: parser::Posting<'_, D>) -> Self {
        Self {
            flag: v.flag,
            account: v.account.into(),
            amount: v.amount.map(|x| x.into()),
            cost: v.cost.map(|x| x.into()),
            price: v.price.map(|x| x.into()),
            metadata: v
                .metadata
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.into()))
                .collect(),
        }
    }
}

impl<D> From<parser::Cost<'_, D>> for Cost<D> {
    fn from(v: parser::Cost<'_, D>) -> Self {
        Self {
            amount: v.amount.map(|x| x.into()),
            date: v.date,
        }
    }
}

impl<D> From<parser::Amount<'_, D>> for Amount<D> {
    fn from(v: parser::Amount<'_, D>) -> Self {
        Self {
            value: v.value,
            currency: v.currency.into(),
        }
    }
}

impl<D> From<parser::PostingPrice<'_, D>> for PostingPrice<D> {
    fn from(v: parser::PostingPrice<'_, D>) -> Self {
        match v {
            parser::PostingPrice::Unit(x) => PostingPrice::Unit(x.into()),
            parser::PostingPrice::Total(x) => PostingPrice::Total(x.into()),
        }
    }
}

impl From<parser::Currency<'_>> for Currency {
    fn from(v: parser::Currency<'_>) -> Self {
        Currency(v.as_str().to_owned())
    }
}

pub fn parse(content: &str) -> anyhow::Result<BeancountFile<rust_decimal::Decimal>> {
    let beancount = match parser::parse::<rust_decimal::Decimal>(&content) {
        Ok(b) => b,
        Err(err) => anyhow::bail!("failed to parse the beancount file: {:?}", err),
    };
    Ok(beancount.into())
}
