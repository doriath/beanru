use crate::types::*;

pub fn closing<D: Decimal>(file: &mut BeancountFile<D>) -> anyhow::Result<()> {
    // TODO: figure out which closing accounts are already taken

    let mut closing_id = 0;
    let mut closing_accounts: Vec<(Account, Currency)> = Vec::new();

    let mut closing: Vec<&mut Directive<D>> = file
        .directives
        .iter_mut()
        .filter(|d| contains_closing_posting(d))
        .collect();
    closing.sort_by_key(|d| d.date);

    for i in 0..closing.len() {
        for j in (i + 1)..closing.len() {
            if closing[i].date != closing[j].date {
                continue;
            }
            let p1 = match closing_posting(closing[i]) {
                Some(p) => (*p).clone(),
                None => continue,
            };
            let p2: &Posting<D> = match closing_posting(closing[j]) {
                Some(p) => p,
                None => continue,
            };
            let a1 = match &p1.amount {
                Some(a) => a,
                None => continue,
            };
            let a2 = match &p2.amount {
                Some(a) => a,
                None => continue,
            };
            if a1.currency == a2.currency && a1.value == -a2.value.clone() {
                let account = Account(format!("Assets:Closing:{:06}", closing_id));
                closing_id += 1;

                closing_posting(closing[i]).unwrap().account = account.clone();
                closing_posting(closing[j]).unwrap().account = account.clone();
                closing_accounts.push((account.clone(), a1.currency.clone()));
                println!("{}\n{}", closing[i], closing[j]);
            }
        }
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
