use std::{collections::HashMap, fmt::Display};

use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, EnumIter)]
pub enum Currency {
    Copper,
    Silver,
    Electrum,
    Gold,
    Platinum,
}

impl Currency {
    pub fn acronym(&self) -> &str {
        match self {
            Currency::Copper => "CP",
            Currency::Silver => "SP",
            Currency::Electrum => "EP",
            Currency::Gold => "GP",
            Currency::Platinum => "PP",
        }
    }

    pub fn to_gold(&self, amount: u32) -> f32 {
        let amount = amount as f32;
        match self {
            Currency::Copper => amount / 100.0,
            Currency::Silver => amount / 10.0,
            Currency::Electrum => amount / 2.0,
            Currency::Gold => amount,
            Currency::Platinum => amount * 10.0,
        }
    }
}

impl<T> From<T> for Currency
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        for currency in Currency::iter() {
            if currency.acronym().eq_ignore_ascii_case(s.as_ref()) {
                return currency;
            }
        }
        panic!("Invalid currency format: {}", s.as_ref());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonetaryValue {
    pub values: HashMap<Currency, u32>,
}

pub enum MonetaryValueError {
    InsufficientFunds,
}

impl MonetaryValue {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn add(&mut self, currency: Currency, amount: u32) {
        *self.values.entry(currency).or_insert(0) += amount;
    }

    pub fn remove(&mut self, currency: Currency, amount: u32) -> Result<(), MonetaryValueError> {
        if let Some(current_amount) = self.values.get_mut(&currency) {
            if *current_amount >= amount {
                *current_amount -= amount;
                if *current_amount == 0 {
                    self.values.remove(&currency);
                }
                return Ok(());
            }
        }
        Err(MonetaryValueError::InsufficientFunds)
    }

    pub fn total_in_gold(&self) -> f32 {
        self.values
            .iter()
            .map(|(currency, &amount)| currency.to_gold(amount))
            .sum()
    }
}

impl<T> From<T> for MonetaryValue
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        let mut values = HashMap::new();
        for part in s.as_ref().split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let mut parts = part.split_whitespace();
            let amount = parts.next().unwrap().parse::<u32>().unwrap_or(0);
            let currency = parts.next().unwrap_or("GP").into();
            values.insert(currency, amount);
        }
        Self { values }
    }
}

impl Display for MonetaryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        for (currency, &amount) in &self.values {
            if amount > 0 {
                parts.push(format!("{} {}", amount, currency.acronym()));
            }
        }
        write!(f, "{}", parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_acronym() {
        assert_eq!(Currency::Copper.acronym(), "CP");
        assert_eq!(Currency::Silver.acronym(), "SP");
        assert_eq!(Currency::Electrum.acronym(), "EP");
        assert_eq!(Currency::Gold.acronym(), "GP");
        assert_eq!(Currency::Platinum.acronym(), "PP");
    }

    #[test]
    fn test_currency_to_gold() {
        assert!((Currency::Copper.to_gold(100) - 1.0).abs() < f32::EPSILON);
        assert!((Currency::Silver.to_gold(10) - 1.0).abs() < f32::EPSILON);
        assert!((Currency::Electrum.to_gold(2) - 1.0).abs() < f32::EPSILON);
        assert!((Currency::Gold.to_gold(1) - 1.0).abs() < f32::EPSILON);
        assert!((Currency::Platinum.to_gold(1) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from("GP"), Currency::Gold);
        assert_eq!(Currency::from("gp"), Currency::Gold);
        assert_eq!(Currency::from("SP"), Currency::Silver);
        assert_eq!(Currency::from("pp"), Currency::Platinum);
    }

    #[test]
    #[should_panic(expected = "Invalid currency format")]
    fn test_currency_from_invalid_str() {
        let _ = Currency::from("ZZ");
    }

    #[test]
    fn test_item_value_from_str_single() {
        let value = MonetaryValue::from("10 GP");
        assert_eq!(value.values.get(&Currency::Gold), Some(&10));
    }

    #[test]
    fn test_item_value_from_str_multiple() {
        let value = MonetaryValue::from("5 GP, 20 SP, 100 CP");
        assert_eq!(value.values.get(&Currency::Gold), Some(&5));
        assert_eq!(value.values.get(&Currency::Silver), Some(&20));
        assert_eq!(value.values.get(&Currency::Copper), Some(&100));
    }

    #[test]
    fn test_item_value_from_str_with_whitespace() {
        let value = MonetaryValue::from("  3 GP ,  7 SP ");
        assert_eq!(value.values.get(&Currency::Gold), Some(&3));
        assert_eq!(value.values.get(&Currency::Silver), Some(&7));
    }

    #[test]
    fn test_item_value_from_str_default_currency() {
        let value = MonetaryValue::from("42");
        assert_eq!(value.values.get(&Currency::Gold), Some(&42));
    }
}
