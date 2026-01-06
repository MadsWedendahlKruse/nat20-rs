use std::str::FromStr;

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub weight: Mass,
    pub value: MonetaryValue,
    pub rarity: ItemRarity,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: ItemId::new("nat20_core", "item.default"),
            name: "Unnamed Item".to_string(),
            description: "No description provided.".to_string(),
            weight: Mass::new::<kilogram>(0.0),
            value: MonetaryValue::from_str("0 GP").unwrap(),
            rarity: ItemRarity::Common,
        }
    }
}
