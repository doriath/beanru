use beancount_parser_2 as parser;
use std::collections::{HashMap, HashSet};

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
pub struct Currency(pub String);

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

#[derive(Clone, Debug)]
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
pub struct Account(pub String);
