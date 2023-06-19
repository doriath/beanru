use crate::types::*;
use std::collections::HashMap;
use rust_decimal::Decimal;

/// Checks if given beancount file is correct.
///
/// TODO: Add other checks, like verifying that accounts are open and verify balance
/// directives.
pub fn check(file: &BeancountFile<rust_decimal::Decimal>) -> anyhow::Result<()> {
    for d in &file.directives {
        let t = match &d.content {
            DirectiveContent::Transaction(t) => t,
            _ => continue,
        };
        check_transaction(&t)?;
    }
    Ok(())
}

fn posting_amount_to_balance(
    posting: &Posting<rust_decimal::Decimal>,
) -> Option<Amount<rust_decimal::Decimal>> {
    let amount = match &posting.amount {
        Some(amount) => amount,
        None => return None,
    };
    let sign = if amount.value.is_sign_positive() {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    if let Some(cost) = &posting.cost {
        if let Some(cost_amount) = &cost.amount {
            return Some(Amount {
                value: amount.value * cost_amount.value,
                currency: cost_amount.currency.clone(),
            });
        }
    }
    match &posting.price {
        Some(PostingPrice::Unit(price_amount)) => {
            return Some(Amount {
                value: amount.value * price_amount.value,
                currency: price_amount.currency.clone(),
            })
        }
        Some(PostingPrice::Total(price_amount)) => {
            return Some(Amount {
                value: price_amount.value * sign,
                currency: price_amount.currency.clone(),
            })
        }
        _ => (),
    }
    Some(amount.clone())
}

fn check_transaction(t: &Transaction<rust_decimal::Decimal>) -> anyhow::Result<()> {
    let mut amounts: HashMap<Currency, rust_decimal::Decimal> = HashMap::new();
    let mut no_amount_count = 0;
    for posting in &t.postings {
        match posting_amount_to_balance(&posting) {
            Some(amount) => {
                *amounts.entry(amount.currency.clone()).or_insert(0.into()) += amount.value
            }
            None => no_amount_count += 1,
        };
    }
    anyhow::ensure!(no_amount_count <= 1, "more than one posting without amount");
    for (currency, amount) in amounts {
        if amount == 0.into() {
            continue;
        }
        if no_amount_count == 0 {
            println!(
                "Transaction does not balance:\n{:?}\n{}: {}",
                t, currency, amount
            );
        }
        no_amount_count -= 1;
    }
    // println!("{:?}", amounts);
    Ok(())
}
