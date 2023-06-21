use crate::types::*;

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
        if !t.balanced {
            println!("Transaction not balanced:\n{}", d);
        }
    }
    Ok(())
}
