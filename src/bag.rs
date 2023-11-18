use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
};

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
    /// Returns new empty bag of currencies.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the Bag is empty (has no currencies or values for all currencies is 0).
    pub fn is_zero(&self) -> bool {
        let zero: D = Default::default();
        !self.currencies.values().any(|a| *a != zero)
    }

    /// Removes entries with value 0 from the bag.
    pub fn trim(&mut self) {
        let zero: D = Default::default();
        self.currencies.retain(|_, v| *v != zero)
    }

    /// Returns the list of commodities and amounts currently stored in the bag.
    pub fn commodities(&self) -> &HashMap<Currency, D> {
        &self.currencies
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

impl<D> AddAssign<Bag<D>> for Bag<D>
where
    D: Decimal,
{
    fn add_assign(&mut self, rhs: Bag<D>) {
        for (commodity, amount) in rhs.currencies {
            *self.currencies.entry(commodity).or_default() += amount;
        }
    }
}

impl<D> AddAssign<&Bag<D>> for Bag<D>
where
    D: Decimal,
{
    fn add_assign(&mut self, rhs: &Bag<D>) {
        for (commodity, amount) in &rhs.currencies {
            *self.currencies.entry(commodity.clone()).or_default() += amount.clone();
        }
    }
}

impl<D> Add<Bag<D>> for Bag<D>
where
    D: Decimal,
{
    type Output = Bag<D>;

    fn add(self, rhs: Bag<D>) -> Bag<D> {
        let mut res = self;
        for (commodity, amount) in rhs.currencies {
            *res.currencies.entry(commodity).or_default() += amount;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn bag_supports_add_assign_with_amount() {
        let mut bag: Bag<rust_decimal::Decimal> = Default::default();
        bag += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        bag += Amount {
            value: 2.into(),
            currency: "USD".into(),
        };
        let expected: HashMap<Currency, rust_decimal::Decimal> = HashMap::from([
            (Currency("CHF".into()), 1.into()),
            (Currency("USD".into()), 2.into()),
        ]);
        assert_eq!(bag.currencies, expected);
    }

    #[test]
    fn bag_supports_add_with_bag() {
        let mut bag1: Bag<rust_decimal::Decimal> = Default::default();
        let mut bag2: Bag<rust_decimal::Decimal> = Default::default();
        bag1 += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        bag2 += Amount {
            value: 2.into(),
            currency: "USD".into(),
        };
        let bag = bag1 + bag2;
        let expected: HashMap<Currency, rust_decimal::Decimal> = HashMap::from([
            (Currency("CHF".into()), 1.into()),
            (Currency("USD".into()), 2.into()),
        ]);
        assert_eq!(bag.currencies, expected);
    }

    #[test]
    fn bag_supports_add_assign_with_bag() {
        let mut bag1: Bag<rust_decimal::Decimal> = Default::default();
        let mut bag2: Bag<rust_decimal::Decimal> = Default::default();
        bag1 += Amount {
            value: 1.into(),
            currency: "CHF".into(),
        };
        bag2 += Amount {
            value: 2.into(),
            currency: "USD".into(),
        };
        bag1 += bag2;
        let expected: HashMap<Currency, rust_decimal::Decimal> = HashMap::from([
            (Currency("CHF".into()), 1.into()),
            (Currency("USD".into()), 2.into()),
        ]);
        assert_eq!(bag1.currencies, expected);
    }
}
