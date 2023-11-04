use crate::types::*;
use chrono::Duration;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Write;

pub fn closing<D: Decimal>(file: &mut BeancountFile<D>, days: i64) -> anyhow::Result<()> {
    let mut next_closing_id = last_closing_id(&file.directives) + 1;
    let mut closing_accounts: Vec<(Account, Currency)> = Vec::new();

    let mut directives: Vec<&mut Directive<D>> = file
        .directives
        .iter_mut()
        .filter(|d| contains_closing_posting(d))
        .collect();
    directives.sort_by_key(|d| d.date);

    println!("Found {} unmatched closing directives", directives.len());

    for i in 0..directives.len() {
        println!("============================================================");
        let best = find_best_matches(&directives, i, days);
        if best.is_empty() {
            println!("{}", directives[i]);
            println!("Could not find a matching directive");
        } else {
            println!("{}", directives[i]);
            println!("--------------------------");
            println!("Found {} matching directives, showing top 3", best.len());
            for j in 0..(std::cmp::min(3, best.len())) {
                println!("{}", directives[best[j]]);
            }

            if let Some(chosen) = ask_user(&best) {
                match_directives(
                    &mut directives,
                    &mut next_closing_id,
                    &mut closing_accounts,
                    i,
                    chosen,
                );
            }
        }
    }

    balance_new_closing_accounts(file, &closing_accounts);

    Ok(())
}

fn match_directives<D: Decimal>(
    directives: &mut [&mut Directive<D>],
    next_closing_id: &mut i32,
    closing_accounts: &mut Vec<(Account, Currency)>,
    a: usize,
    b: usize,
) {
    let account = Account(format!("Assets:Closing:{:06}", next_closing_id));
    *next_closing_id += 1;

    let amount_a = closing_posting(directives[a])
        .and_then(|p| p.amount.clone())
        .unwrap();
    let amount_b = closing_posting(directives[b])
        .and_then(|p| p.amount.clone())
        .unwrap();

    let p = closing_posting_mut(directives[a]).unwrap();
    p.account = account.clone();
    p.flag = None;
    if amount_a.currency != amount_b.currency {
        p.amount = Some(-amount_b.clone());
        p.price = Some(PostingPrice::Total(amount_a.abs()));
        p.autocomputed = false
    }

    let p = closing_posting_mut(directives[b]).unwrap();
    p.account = account.clone();
    p.flag = None;
    closing_accounts.push((account, amount_b.currency));
}

fn balance_new_closing_accounts<D: Decimal>(
    file: &mut BeancountFile<D>,
    closing_accounts: &[(Account, Currency)],
) {
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
}

fn find_best_matches<D: Decimal>(
    directives: &[&mut Directive<D>],
    start: usize,
    max_days: i64,
) -> Vec<usize> {
    let start_d: &Directive<D> = directives[start];
    let start_date = start_d.date;
    let start_a = match closing_posting(start_d).and_then(|p| p.amount.clone()) {
        Some(a) => -a,
        None => return Vec::new(),
    };
    let expected_currency = start_d
        .metadata
        .get("closing_currency_hint")
        .and_then(|h| match h {
            MetadataValue::String(x) => Some(Currency(x.clone())),
            _ => None,
        })
        .unwrap_or_else(|| start_a.currency.clone());
    println!("Expected currency: {}", expected_currency);

    let mut res = Vec::new();
    for (i, d) in directives.iter().enumerate() {
        let a = match closing_posting(d).and_then(|p| p.amount.clone()) {
            Some(a) => a,
            None => continue,
        };
        let diff = (start_date - d.date).abs();
        if diff > Duration::days(max_days) {
            continue;
        }
        if a.currency != expected_currency {
            continue;
        }
        if start_a != a && start_a.currency == expected_currency {
            continue;
        }
        res.push((i, diff));
    }
    res.sort_by_key(|x| x.1);
    res.into_iter().map(|x| x.0).collect()
}

fn ask_user(best: &[usize]) -> Option<usize> {
    let options = std::cmp::min(3, best.len());
    loop {
        print!("Option [{}/n]?: ", (1..options + 1).join("/"));
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "n" || input == "N" {
            return None;
        }
        if input.is_empty() || input == "1" {
            return Some(best[0]);
        }
        if input == "2" && best.len() <= 2 {
            return Some(best[1]);
        }
        if input == "3" && best.len() <= 3 {
            return Some(best[2]);
        }
    }
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

fn closing_posting_mut<D: Decimal>(d: &mut Directive<D>) -> Option<&mut Posting<D>> {
    let t = match &mut d.content {
        DirectiveContent::Transaction(t) => t,
        _ => panic!("directive is not a transaction"),
    };
    let closing = Account("Assets:Closing".into());
    t.postings.iter_mut().find(|p| p.account == closing)
}

fn closing_posting<D: Decimal>(d: &Directive<D>) -> Option<&Posting<D>> {
    let t = match &d.content {
        DirectiveContent::Transaction(t) => t,
        _ => panic!("directive is not a transaction"),
    };
    let closing = Account("Assets:Closing".into());
    t.postings.iter().find(|p| p.account == closing)
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

    // #[test]
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
