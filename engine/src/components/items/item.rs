use strum::Display;

use crate::components::{id::ItemId, items::money::MonetaryValue};

#[derive(Debug, Clone, PartialEq, Display)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub value: MonetaryValue,
    pub rarity: ItemRarity,
}

impl Item {
    pub fn new(
        id: ItemId,
        name: String,
        description: String,
        weight: f32,
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
            weight: 0.0,
            value: MonetaryValue::from("0 GP"),
            rarity: ItemRarity::Common,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::items::money::Currency;

    use super::*;

    #[test]
    fn test_item_default() {
        let item = Item::default();
        assert_eq!(item.name, "Unnamed Item");
        assert_eq!(item.description, "No description provided.");
        assert_eq!(item.weight, 0.0);
        assert_eq!(item.value.values.get(&Currency::Gold), Some(&0));
        assert_eq!(item.rarity, ItemRarity::Common);
    }

    #[test]
    fn test_item_new() {
        let id = ItemId::from_str("item.sword");
        let value = MonetaryValue::from("15 GP, 2 SP");
        let item = Item::new(
            id.clone(),
            "Sword".to_string(),
            "A sharp blade.".to_string(),
            3.5,
            value.clone(),
            ItemRarity::Rare,
        );
        assert_eq!(item.id, id);
        assert_eq!(item.name, "Sword");
        assert_eq!(item.description, "A sharp blade.");
        assert_eq!(item.weight, 3.5);
        assert_eq!(item.value, value);
        assert_eq!(item.rarity, ItemRarity::Rare);
    }
}
