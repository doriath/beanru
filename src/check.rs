use crate::types::*;

/// Checks if given beancount file is correct.
///
/// TODO: Add other checks, like verifying that accounts are open and verify balance
/// directives.
pub fn check<D: Decimal>(ledger: &Ledger<D>) -> anyhow::Result<()> {
    for (_, file) in &ledger.files {
        for d in &file.directives {
            let t = match &d.content {
                DirectiveContent::Transaction(t) => t,
                _ => continue,
            };
            if !t.balanced {
                println!("Transaction not balanced:\n{}", d);
            }
        }
    }
    Ok(())
}
