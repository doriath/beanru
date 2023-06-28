use crate::types::*;
use chrono::{Duration, NaiveDate};
use std::collections::HashMap;

pub fn closing<D: Decimal>(file: &mut BeancountFile<D>) -> anyhow::Result<()> {
    // TODO: figure out which closing accounts are already taken

    let mut closing_id = 0;
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
            .filter(|j| date_within(&directives[i].date, &directives[**j].date, 15))
            .cloned()
            .collect();
        if m.len() != 1 {
            continue;
        }
        let j = m[0];

        let mj: Vec<usize> = matching
            .iter()
            .filter(|k| date_within(&directives[j].date, &directives[**k].date, 15))
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
    Assets:Closing:000000

2022-01-01 *
    Assets:Bank1 5 CHF
    Assets:Closing:000000

2000-01-01 open Assets:Closing:000000 CHF
2099-01-01 balance Assets:Closing:000000 0 CHF
"#;
        let mut got = parse(input).unwrap();
        closing(&mut got).unwrap();
        assert_eq!(parse(expected).unwrap(), got);
    }
}
