use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use strum::Display;
use uom::si::{f32::Mass, mass::kilogram};

use crate::components::{id::ItemId, items::money::MonetaryValue};

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub weight: Mass,
    #[serde_as(as = "DisplayFromStr")]
    pub value: MonetaryValue,
    pub rarity: ItemRarity,
}

impl Item {
    // TODO: Redundant?
    pub fn new(
        id: ItemId,
        name: String,
        description: String,
        weight: Mass,
        value: MonetaryValue,
        rarity: ItemRarity,
    ) -> Self {
        Self {
            id,
            name,
            description,
            weight,
            value,
            rarity,
        }
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: ItemId::from_str("item.default"),
            name: "Unnamed Item".to_string(),
            description: "No description provided.".to_string(),
            weight: Mass::new::<kilogram>(0.0),
            value: MonetaryValue::from_str("0 GP").unwrap(),
            rarity: ItemRarity::Common,
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::mass::pound;

    use crate::components::items::money::Currency;

    use super::*;

    #[test]
    fn item_default() {
        let item = Item::default();
        assert_eq!(item.name, "Unnamed Item");
        assert_eq!(item.description, "No description provided.");
        assert_eq!(item.weight, Mass::new::<kilogram>(0.0));
        assert_eq!(item.value.values.get(&Currency::Gold), Some(&0));
        assert_eq!(item.rarity, ItemRarity::Common);
    }

    #[test]
    fn item_new() {
        let id = ItemId::from_str("item.sword");
        let value = MonetaryValue::from_str("15 GP, 2 SP").unwrap();
        let item = Item::new(
            id.clone(),
            "Sword".to_string(),
            "A sharp blade.".to_string(),
            Mass::new::<pound>(3.5),
            value.clone(),
            ItemRarity::Rare,
        );
        assert_eq!(item.id, id);
        assert_eq!(item.name, "Sword");
        assert_eq!(item.description, "A sharp blade.");
        assert_eq!(item.weight, Mass::new::<pound>(3.5));
        assert_eq!(item.value, value);
        assert_eq!(item.rarity, ItemRarity::Rare);
    }
}
