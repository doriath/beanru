use crate::types::*;

pub fn closing<D: Decimal>(file: &mut BeancountFile<D>) -> anyhow::Result<()> {
    let mut _closing_trans: Vec<&mut Directive<D>> = file
        .directives
        .iter_mut()
        .filter(|d| contains_closing_posting(d))
        .collect();

    Ok(())
}

fn contains_closing_posting<D: Decimal>(d: &Directive<D>) -> bool {
    let t = match &d.content {
        DirectiveContent::Transaction(t) => t,
        _ => return false,
    };
    let closing = Account("Assets:Closing".into());
    t.postings.iter().any(|p| p.account == closing)
}
