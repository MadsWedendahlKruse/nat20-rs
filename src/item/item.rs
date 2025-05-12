#[derive(Debug, Clone)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub value: u32,
    pub rarity: ItemRarity,
}
