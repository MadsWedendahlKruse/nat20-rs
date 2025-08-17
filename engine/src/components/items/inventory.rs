use crate::components::items::{
    equipment::{armor::Armor, equipment::EquipmentItem, weapon::Weapon},
    item::Item,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ItemInstance {
    Item(Item),
    Armor(Armor),
    Weapon(Weapon),
    Equipment(EquipmentItem),
}

pub trait ItemContainer {
    fn item(&self) -> &Item;
}

impl ItemContainer for ItemInstance {
    fn item(&self) -> &Item {
        match self {
            ItemInstance::Item(item) => item,
            ItemInstance::Armor(armor) => armor.item(),
            ItemInstance::Weapon(weapon) => &weapon.equipment().item,
            ItemInstance::Equipment(equipment) => &equipment.item,
        }
    }
}

macro_rules! impl_into_item_instance {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl Into<ItemInstance> for $ty {
                fn into(self) -> ItemInstance {
                    ItemInstance::$variant(self)
                }
            }
        )*
    };
}

impl_into_item_instance! {
    Item => Item,
    Armor => Armor,
    Weapon => Weapon,
    EquipmentItem => Equipment,
}

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    items: Vec<ItemInstance>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: ItemInstance) {
        self.items.push(item);
    }

    pub fn remove(&mut self, index: usize) -> Option<ItemInstance> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn items(&self) -> &[ItemInstance] {
        &self.items
    }

    /// Optional: find by name
    pub fn find_by_name(&self, name: &str) -> Option<&ItemInstance> {
        self.items.iter().find(|i| i.item().name == name)
    }
}
