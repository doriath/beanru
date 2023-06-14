use crate::*;
use itertools::Itertools;

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
                write!(f, "balance {} {}\n", x.account, x.amount)?;
            }
            DirectiveContent::Close(x) => {
                write!(f, "close {}\n", x.account)?;
            }
            DirectiveContent::Commodity(x) => {
                write!(f, "commodity {}\n", x)?;
            }
            DirectiveContent::Open(x) => {
                let mut currencies = x
                    .currencies
                    .clone()
                    .into_iter()
                    .map(|x| x.0)
                    .collect::<Vec<_>>();
                currencies.sort();
                write!(f, "open {} {}\n", x.account, currencies.join(","))?;
            }
            DirectiveContent::Pad(x) => {
                write!(f, "pad {} {}\n", x.account, x.source_account)?;
            }
            DirectiveContent::Price(x) => {
                write!(f, "price {} {}\n", x.currency, x.amount)?;
            }
            DirectiveContent::Transaction(t) => {
                match t.flag {
                    Some(beancount_parser_2::Flag::Completed) => {
                        write!(f, "*")?;
                    }
                    Some(beancount_parser_2::Flag::Incomplete) => {
                        write!(f, "!")?;
                    }
                    None => {
                        write!(f, "*")?;
                    }
                }
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
                write!(f, "\n")?;
            }
            _ => unimplemented!(),
        };
        for (key, value) in self.metadata.iter().sorted_by_key(|x| x.0) {
            write!(f, "  {}: {}\n", key, value)?;
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
        write!(f, "  {}", self.account)?;
        if let Some(amount) = &self.amount {
            write!(f, " {}", amount)?;
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
        write!(f, "\n")?;
        for (key, value) in self.metadata.iter().sorted_by_key(|x| x.0) {
            write!(f, "    {}: {}\n", key, value)?;
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
