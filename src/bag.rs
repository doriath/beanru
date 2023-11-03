use std::{collections::HashMap, ops::AddAssign};

use crate::types::{Amount, Currency, Decimal};

/// Bag of currencies.
///
/// The type is primarily used to accumulate many Amount types.
#[derive(Default, Debug, Clone)]
pub struct Bag<D> {
    // TODO: hide this
    pub currencies: HashMap<Currency, D>,
}

impl<D> Bag<D>
where
    D: Decimal,
{
    /// Returns true if the Bag is empty (has no currencies or values for all currencies is 0).
    pub fn is_zero(&self) -> bool {
        let zero: D = Default::default();
        !self.currencies.values().any(|a| *a != zero)
    }
}

impl<D> AddAssign<Amount<D>> for Bag<D>
where
    D: Decimal,
{
    fn add_assign(&mut self, rhs: Amount<D>) {
        *self.currencies.entry(rhs.currency.clone()).or_default() += rhs.value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn bag_is_zero_returns_true_when_empty() {
        let bag: Bag<rust_decimal::Decimal> = Default::default();
        assert!(bag.is_zero());
    }

    #[test]
    fn bag_is_zero_returns_false_when_not_empty() {
        let mut bag: Bag<rust_decimal::Decimal> = Default::default();
        bag += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        assert!(!bag.is_zero());
    }

    #[test]
    fn bag_is_zero_returns_true_when_all_are_zero() {
        let mut bag: Bag<rust_decimal::Decimal> = Default::default();
        bag += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        bag += Amount {
            value: (-1).into(),
            currency: "CHF".into(),
        };
        bag += Amount {
            value: 0.into(),
            currency: "USD".into(),
        };
        assert!(bag.is_zero());
    }

    #[test]
    fn bag_supports_add_assign() {
        let mut bag: Bag<rust_decimal::Decimal> = Default::default();
        bag += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        bag += Amount {
            value: 2.into(),
            currency: "USD".into(),
        };
    }
}
