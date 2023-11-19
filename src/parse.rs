use crate::types::*;
use beancount_parser as parser;

#[deprecated]
pub fn parse(content: &str) -> anyhow::Result<BeancountFile<rust_decimal::Decimal>> {
    let beancount = match parser::parse::<rust_decimal::Decimal>(content) {
        Ok(b) => b,
        Err(err) => anyhow::bail!("failed to parse the beancount file: {:?}", err),
    };
    Ok(beancount.into())
}

impl<D> From<parser::BeancountFile<D>> for BeancountFile<D>
where
    D: Decimal,
{
    fn from(f: parser::BeancountFile<D>) -> Self {
        BeancountFile::new(
            f.includes,
            f.directives.into_iter().map(|d| d.into()).collect(),
        )
    }
}

impl<D> From<parser::Directive<D>> for Directive<D>
where
    D: Decimal,
{
    fn from(d: parser::Directive<D>) -> Self {
        Directive {
            date: from_parser_date_to_chrono_date(&d.date),
            content: d.content.into(),
            metadata: d
                .metadata
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.into()))
                .collect(),
        }
    }
}

impl<D> From<parser::metadata::Value<D>> for MetadataValue<D> {
    fn from(v: parser::metadata::Value<D>) -> Self {
        match v {
            parser::metadata::Value::String(x) => MetadataValue::String(x),
            parser::metadata::Value::Number(x) => MetadataValue::Number(x),
            parser::metadata::Value::Currency(x) => MetadataValue::Currency(x.into()),
            _ => unimplemented!("given metadata value type is not supported yet"),
        }
    }
}

impl<D> From<parser::DirectiveContent<D>> for DirectiveContent<D>
where
    D: Decimal,
{
    fn from(v: parser::DirectiveContent<D>) -> Self {
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

impl From<parser::Pad> for Pad {
    fn from(v: parser::Pad) -> Self {
        Self {
            account: v.account.into(),
            source_account: v.source_account.into(),
        }
    }
}

impl From<parser::Open> for Open {
    fn from(v: parser::Open) -> Self {
        Self {
            account: v.account.into(),
            currencies: v.currencies.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<parser::Event> for Event {
    fn from(v: parser::Event) -> Self {
        Self {
            name: v.name.to_owned(),
            value: v.value,
        }
    }
}

impl From<parser::Close> for Close {
    fn from(v: parser::Close) -> Self {
        Self {
            account: v.account.into(),
        }
    }
}

impl From<parser::Account> for Account {
    fn from(v: parser::Account) -> Self {
        Account(v.as_str().to_owned())
    }
}

impl<D> From<parser::Price<D>> for Price<D> {
    fn from(v: parser::Price<D>) -> Self {
        Self {
            currency: v.currency.into(),
            amount: v.amount.into(),
        }
    }
}

impl<D> From<parser::Balance<D>> for Balance<D> {
    fn from(v: parser::Balance<D>) -> Self {
        Self {
            account: v.account.into(),
            amount: v.amount.into(),
        }
    }
}

impl<D> From<parser::Transaction<D>> for Transaction<D>
where
    D: Decimal,
{
    fn from(v: parser::Transaction<D>) -> Self {
        let mut t = Self {
            flag: v.flag,
            payee: v.payee.map(String::from),
            narration: v.narration.map(String::from),
            tags: v.tags.into_iter().map(|x| x.to_string()).collect(),
            links: v.links.into_iter().map(|x| x.to_string()).collect(),
            postings: v.postings.into_iter().map(|x| x.into()).collect(),
            balanced: true,
        };
        if t.book().is_err() {
            t.balanced = false
        }
        t
    }
}

impl<D> From<parser::Posting<D>> for Posting<D> {
    fn from(v: parser::Posting<D>) -> Self {
        Self {
            flag: v.flag,
            account: v.account.into(),
            amount: v.amount.map(|x| x.into()),
            cost: v.cost.map(|x| x.into()),
            price: v.price.map(|x| x.into()),
            metadata: v
                .metadata
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.into()))
                .collect(),
            autocomputed: false,
        }
    }
}

impl<D> From<parser::Cost<D>> for Cost<D> {
    fn from(v: parser::Cost<D>) -> Self {
        Self {
            amount: v.amount.map(|x| x.into()),
            date: v.date.map(|d| from_parser_date_to_chrono_date(&d)),
        }
    }
}

fn from_parser_date_to_chrono_date(d: &parser::Date) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(d.year.into(), d.month.into(), d.day.into()).unwrap()
}

impl<D> From<parser::Amount<D>> for Amount<D> {
    fn from(v: parser::Amount<D>) -> Self {
        Self {
            value: v.value,
            currency: v.currency.into(),
        }
    }
}

impl<D> From<parser::PostingPrice<D>> for PostingPrice<D> {
    fn from(v: parser::PostingPrice<D>) -> Self {
        match v {
            parser::PostingPrice::Unit(x) => PostingPrice::Unit(x.into()),
            parser::PostingPrice::Total(x) => PostingPrice::Total(x.into()),
        }
    }
}

impl From<parser::Currency> for Currency {
    fn from(v: parser::Currency) -> Self {
        Currency(v.as_str().to_owned())
    }
}
