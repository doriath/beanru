use std::collections::{HashMap, HashSet};

use crate::{bag::Bag, types::*};

fn priority<D: Decimal>(d: &Directive<D>) -> i32 {
    match d.content {
        DirectiveContent::Balance(_) => 0,
        _ => 1,
    }
}

/// Tries to fix the balance statements.
pub fn fix_balance<D: Decimal>(ledger: &Ledger<D>) -> anyhow::Result<()> {
    let mut directives: Vec<&Directive<D>> = Vec::new();
    for (_, file) in &ledger.files {
        for d in &file.directives {
            directives.push(d);
        }
    }
    directives.sort_by(|a, b| (a.date, priority(a)).cmp(&(b.date, priority(b))));

    let mut failed: Vec<(&Directive<D>, &Balance<D>, D, bool)> = Vec::new();
    let mut accounts: HashMap<Account, Bag<D>> = HashMap::new();
    let mut padded: HashSet<Account> = HashSet::new();
    for d in directives {
        match &d.content {
            DirectiveContent::Balance(balance) => {
                let bag: Bag<D> = accounts.get(&balance.account).cloned().unwrap_or_default();
                let x = bag
                    .commodities()
                    .get(&balance.amount.currency)
                    .cloned()
                    .unwrap_or_default();
                let mut y: D = x.clone() - balance.amount.value.clone();
                if y < 0.into() {
                    y = -y;
                }
                // TODO: improve the error margin
                let c15: D = 40.into();
                let c1000: D = 1000.into();
                if padded.contains(&balance.account) {
                    let mut bag = Bag::new();
                    bag += balance.amount.clone();
                    accounts.insert(balance.account.clone(), bag);
                    padded.remove(&balance.account);
                    continue;
                }
                if y > (c15 / c1000) {
                    failed.push((d, balance, x.clone(), false));
                } else {
                    for f in &mut failed {
                        if f.1.account == balance.account {
                            f.3 = true
                        }
                    }
                }
            }
            DirectiveContent::Pad(pad) => {
                padded.insert(pad.account.clone());
            }
            DirectiveContent::Transaction(t) => {
                for p in &t.postings {
                    if let Some(a) = &p.amount {
                        *accounts.entry(p.account.clone()).or_default() += a.clone();
                    }
                }
            }
            _ => continue,
        }
    }
    for f in failed {
        println!(
            "Balance failed for {} {}, expected {} got {} {}",
            f.0.date,
            f.1.account,
            f.1.amount,
            f.2,
            if f.3 { "(but fixed later)" } else { "" }
        );
    }

    Ok(())
}
