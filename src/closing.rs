use crate::types::*;
use chrono::{Duration, NaiveDate};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub fn closing<D: Decimal>(file: &mut BeancountFile<D>, days: i64) -> anyhow::Result<()> {
    let mut closing_id = last_closing_id(&file.directives) + 1;
    let mut closing_accounts: Vec<(Account, Currency)> = Vec::new();

    let mut directives: Vec<&mut Directive<D>> = file
        .directives
        .iter_mut()
        .filter(|d| contains_closing_posting(d))
        .collect();
    directives.sort_by_key(|d| d.date);

    let mut amount_to_idx: HashMap<Amount<D>, Vec<usize>> = HashMap::new();

    for i in 0..directives.len() {
        let a = match closing_posting(directives[i]).and_then(|p| p.amount.clone()) {
            Some(a) => a,
            None => continue,
        };
        amount_to_idx.entry(a).or_default().push(i);
    }

    for i in 0..directives.len() {
        let a = match closing_posting(directives[i]).and_then(|p| p.amount.clone()) {
            Some(a) => a,
            None => continue,
        };
        let currency = a.currency.clone();
        let matching = match amount_to_idx.get(&-a) {
            Some(m) => m,
            None => continue,
        };
        let m: Vec<usize> = matching
            .iter()
            .filter(|j| date_within(&directives[i].date, &directives[**j].date, days))
            .cloned()
            .collect();
        if m.len() != 1 {
            println!("Too many transaction in close range, consider running with --days=N");
            continue;
        }
        let j = m[0];

        let mj: Vec<usize> = matching
            .iter()
            .filter(|k| date_within(&directives[j].date, &directives[**k].date, days))
            .cloned()
            .collect();
        if mj.len() != 1 {
            continue;
        }
        if closing_posting(directives[i]).is_none() || closing_posting(directives[j]).is_none() {
            continue;
        }

        let account = Account(format!("Assets:Closing:{:06}", closing_id));
        closing_id += 1;
        closing_posting(directives[i]).unwrap().account = account.clone();
        closing_posting(directives[j]).unwrap().account = account.clone();
        closing_accounts.push((account.clone(), currency));
        println!("{}\n{}", directives[i], directives[j]);
    }

    for (account, currency) in closing_accounts {
        file.directives.push(Directive {
            date: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            content: DirectiveContent::Open(Open {
                account: account.clone(),
                currencies: [currency.clone()].into_iter().collect(),
            }),
            metadata: Default::default(),
        });
        file.directives.push(Directive {
            date: chrono::NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(),
            content: DirectiveContent::Balance(Balance {
                account: account.clone(),
                amount: Amount {
                    value: 0.into(),
                    currency: currency.clone(),
                },
            }),
            metadata: Default::default(),
        });
    }

    Ok(())
}

fn last_closing_id<D>(directives: &[Directive<D>]) -> i32 {
    directives
        .iter()
        .filter_map(|d| d.content.open_opt())
        .filter_map(|p| parse_closing_id(&p.account))
        .max()
        .unwrap_or(0)
}

fn parse_closing_id(account: &Account) -> Option<i32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^Assets:Closing:(\d{6})$").unwrap();
    }
    RE.captures(&account.0)?
        .get(1)?
        .as_str()
        .parse::<i32>()
        .ok()
}

fn date_within(d1: &NaiveDate, d2: &NaiveDate, days: i64) -> bool {
    let mut dur = *d1 - *d2;
    if dur < Duration::zero() {
        dur = -dur;
    }
    dur <= Duration::days(days)
}

fn closing_posting<D: Decimal>(d: &mut Directive<D>) -> Option<&mut Posting<D>> {
    let t = match &mut d.content {
        DirectiveContent::Transaction(t) => t,
        _ => panic!("directive is not a transaction"),
    };
    let closing = Account("Assets:Closing".into());
    t.postings.iter_mut().find(|p| p.account == closing)
}

fn contains_closing_posting<D: Decimal>(d: &Directive<D>) -> bool {
    let t = match &d.content {
        DirectiveContent::Transaction(t) => t,
        _ => return false,
    };
    // TODO: make it configurable
    let closing = Account("Assets:Closing".into());
    t.postings.iter().any(|p| p.account == closing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_closing() {
        let input = r#"
2022-01-01 *
    Assets:Bank1 -5 CHF
    Assets:Closing

2022-01-01 *
    Assets:Bank1 5 CHF
    Assets:Closing
"#;
        let expected = r#"
2022-01-01 *
    Assets:Bank1 -5 CHF
    Assets:Closing:000001

2022-01-01 *
    Assets:Bank1 5 CHF
    Assets:Closing:000001

2000-01-01 open Assets:Closing:000001 CHF
2099-01-01 balance Assets:Closing:000001 0 CHF
"#;
        let mut got = parse(input).unwrap();
        closing(&mut got, 15).unwrap();
        assert_eq!(parse(expected).unwrap(), got);
    }

    #[test]
    fn test_parse_closing_id() {
        let cases = &[("Assets:Closing:000000", Some(0))];

        for (a, id) in cases {
            assert_eq!(parse_closing_id(&Account(a.to_string())), *id);
        }
    }
}
