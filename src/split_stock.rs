use crate::types::*;

pub fn split_stock(
    file: &mut BeancountFile<rust_decimal::Decimal>,
    commodity: &Currency,
    ratio: rust_decimal::Decimal,
) -> anyhow::Result<()> {
    for directive in &mut file.directives {
        match &mut directive.content {
            DirectiveContent::Balance(x) => {
                if x.amount.currency != *commodity {
                    continue;
                }
                x.amount.value *= ratio;
            }
            DirectiveContent::Close(_) => (),
            DirectiveContent::Commodity(_) => (),
            DirectiveContent::Event(_) => (),
            DirectiveContent::Open(_) => (),
            DirectiveContent::Pad(_) => (),
            DirectiveContent::Price(price) => {
                if price.currency != *commodity {
                    continue;
                }
                price.amount.value /= ratio;
            }
            DirectiveContent::Transaction(t) => {
                for mut posting in &mut t.postings {
                    split_stock_posting(&mut posting, commodity, ratio)?;
                }
            }
        }
    }

    Ok(())
}

pub fn split_stock_posting(
    posting: &mut Posting<rust_decimal::Decimal>,
    commodity: &Currency,
    ratio: rust_decimal::Decimal,
) -> anyhow::Result<()> {
    let mut amount = match &mut posting.amount {
        Some(amount) => amount,
        None => return Ok(()),
    };
    if amount.currency != *commodity {
        return Ok(());
    }
    amount.value *= ratio;
    if let Some(cost) = &mut posting.cost {
        if let Some(cost_amount) = &mut cost.amount {
            cost_amount.value /= ratio;
        }
    }
    if let Some(PostingPrice::Unit(price)) = &mut posting.price {
        price.value /= ratio;
    }
    Ok(())
}
