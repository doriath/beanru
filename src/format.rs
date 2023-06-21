use crate::types::*;
use itertools::Itertools;
use std::collections::HashSet;

fn sorted_hashset(h: &HashSet<String>) -> Vec<String> {
    let mut v = h.clone().into_iter().collect::<Vec<_>>();
    v.sort();
    v
}

fn quote(s: &str) -> String {
    // TODO: do proper escaping
    format!("\"{}\"", s)
}

impl<D> std::fmt::Display for BeancountFile<D>
where
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for directive in &self.directives {
            write!(f, "{}", directive)?;
        }
        Ok(())
    }
}

impl<D> std::fmt::Display for Directive<D>
where
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{:02}-{:02} ",
            self.date.year, self.date.month, self.date.day
        )?;
        match &self.content {
            DirectiveContent::Balance(x) => {
                writeln!(f, "balance {} {}", x.account, x.amount)?;
            }
            DirectiveContent::Close(x) => {
                writeln!(f, "close {}", x.account)?;
            }
            DirectiveContent::Commodity(x) => {
                writeln!(f, "commodity {}", x)?;
            }
            DirectiveContent::Open(x) => {
                let mut currencies = x
                    .currencies
                    .clone()
                    .into_iter()
                    .map(|x| x.0)
                    .collect::<Vec<_>>();
                currencies.sort();
                write!(f, "open {}", x.account)?;
                if !currencies.is_empty() {
                    write!(f, " {}", currencies.join(","))?;
                }
                writeln!(f)?;
            }
            DirectiveContent::Pad(x) => {
                writeln!(f, "pad {} {}", x.account, x.source_account)?;
            }
            DirectiveContent::Price(x) => {
                writeln!(f, "price {} {}", x.currency, x.amount)?;
            }
            DirectiveContent::Transaction(t) => {
                write!(f, "{}", t.flag.unwrap_or('*'))?;
                if let Some(payee) = &t.payee {
                    write!(f, " {}", quote(payee))?;
                }
                if let Some(narration) = &t.narration {
                    write!(f, " {}", quote(narration))?;
                }
                for tag in sorted_hashset(&t.tags) {
                    write!(f, " #{}", tag)?;
                }
                for link in sorted_hashset(&t.links) {
                    write!(f, " ^{}", link)?;
                }
                writeln!(f)?;
            }
            _ => unimplemented!(),
        };
        for (key, value) in self.metadata.iter().sorted_by_key(|x| x.0) {
            writeln!(f, "  {}: {}", key, value)?;
        }
        if let DirectiveContent::Transaction(t) = &self.content {
            for posting in &t.postings {
                write!(f, "{}", posting)?;
            }
        }
        Ok(())
    }
}

impl<D> std::fmt::Display for Posting<D>
where
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.flag {
            Some(c) => write!(f, "  {} {}", c, self.account)?,
            None => write!(f, "  {}", self.account)?,
        }
        if let Some(amount) = &self.amount {
            if !self.autocomputed {
                write!(f, " {}", amount)?;
            }
        }
        match &self.price {
            Some(PostingPrice::Unit(amount)) => write!(f, " @ {}", amount)?,
            Some(PostingPrice::Total(amount)) => write!(f, " @@ {}", amount)?,
            None => (),
        };
        match &self.cost {
            Some(cost) => {
                if let Some(amount) = &cost.amount {
                    write!(f, " {{{}}}", amount)?;
                }
            }
            None => (),
        };
        writeln!(f)?;
        for (key, value) in self.metadata.iter().sorted_by_key(|x| x.0) {
            writeln!(f, "    {}: {}", key, value)?;
        }
        Ok(())
    }
}

impl<D> std::fmt::Display for MetadataValue<D>
where
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataValue::String(x) => write!(f, "{}", quote(x)),
            MetadataValue::Number(x) => write!(f, "{}", x),
            MetadataValue::Currency(x) => write!(f, "{}", x),
        }
    }
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<D> std::fmt::Display for Amount<D>
where
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}
