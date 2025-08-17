use crate::components::id::ItemId;

#[derive(Debug, Clone, PartialEq)]
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
    pub value: u32,
    pub rarity: ItemRarity,
}

impl Item {
    pub fn new(
        id: ItemId,
        name: String,
        description: String,
        weight: f32,
        value: u32,
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
            value: 0,
            rarity: ItemRarity::Common,
        }
    }
}
