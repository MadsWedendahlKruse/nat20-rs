use hecs::{Entity, World};
use nat20_rs::{
    components::items::{
        equipment::{
            equipment::{EquipmentSlot, GeneralEquipmentSlot, HandSlot},
            weapon::WeaponType,
        },
        inventory::{Inventory, ItemContainer},
    },
    systems,
};
use strum::IntoEnumIterator;

use crate::table_with_columns;

#[derive(Debug, Clone)]
pub enum ContainerSlot {
    Inventory(usize),
    Loadout(EquipmentSlot),
}

#[derive(Debug, Clone)]
pub enum InteractMode {
    RightClick,
    DoubleClick,
    Drag,
}

#[derive(Debug, Clone)]
pub struct InteractEvent {
    pub entity: Entity,
    pub slot: ContainerSlot,
    pub mode: InteractMode,
}

impl InteractEvent {
    pub fn from_ui(ui: &imgui::Ui, entity: Entity, slot: ContainerSlot) -> Option<Self> {
        if ui.is_item_hovered() {
            if ui.is_mouse_released(imgui::MouseButton::Right) {
                return Some(Self {
                    entity,
                    slot,
                    mode: InteractMode::RightClick,
                });
            }

            if ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                return Some(Self {
                    entity,
                    slot,
                    mode: InteractMode::DoubleClick,
                });
            }

            if ui.is_mouse_dragging(imgui::MouseButton::Left) {
                return Some(Self {
                    entity,
                    slot,
                    mode: InteractMode::Drag,
                });
            }
        }
        None
    }
}

pub fn render_inventory(
    ui: &imgui::Ui,
    world: &mut World,
    entity: Entity,
) -> Option<InteractEvent> {
    ui.separator_with_text("Inventory");

    let inventory = systems::helpers::get_component::<Inventory>(world, entity);
    let mut event = None;

    for (i, item) in inventory.items().iter().enumerate() {
        let slot = ContainerSlot::Inventory(i);

        let item_name = item.item().name.clone();
        if ui.button(item_name.clone()) {
            // Handle item click (don't think we need to do anything here)
            println!("Clicked on item: {}", item_name);
        }

        if event.is_none() {
            event = InteractEvent::from_ui(ui, entity, slot);
        }
    }

    event
}

fn render_loadout(ui: &imgui::Ui, world: &mut World, entity: Entity) -> Option<InteractEvent> {
    let loadout = systems::loadout::loadout(world, entity);
    let mut event = None;

    ui.separator_with_text("Weapons");
    if let Some(table) = table_with_columns!(ui, "Weapons", "Hand", "Weapon") {
        for weapon_type in WeaponType::iter() {
            // Render separator for each weapon type
            ui.table_next_row_with_flags(imgui::TableRowFlags::empty());
            ui.table_next_column();
            ui.text_colored([0.7, 0.7, 0.7, 1.0], weapon_type.to_string());
            ui.table_next_column();

            for hand in HandSlot::iter() {
                ui.table_next_column();
                ui.text(hand.to_string());
                ui.table_next_column();
                if let Some(weapon) = loadout.weapon_in_hand(&weapon_type, &hand) {
                    ui.text(weapon.equipment().item.name.to_string());
                    // if ui.is_item_hovered() {
                    //     ui.tooltip(|| {
                    //         weapon.render_with_context(ui, context);
                    //     });
                    // }

                    let equipment_slot = match weapon_type {
                        WeaponType::Melee => EquipmentSlot::Melee(hand),
                        WeaponType::Ranged => EquipmentSlot::Ranged(hand),
                    };
                    let slot = ContainerSlot::Loadout(equipment_slot);

                    if event.is_none() {
                        event = InteractEvent::from_ui(ui, entity, slot);
                    }
                }
            }
        }

        table.end();
    }

    ui.separator_with_text("Equipment");
    if let Some(table) = table_with_columns!(ui, "Equipment", "Slot", "Item") {
        // Armor is technically not considered equipment, but we can sneak
        // it in here for now
        ui.table_next_column();
        ui.text(format!("{}", EquipmentSlot::Armor));
        ui.table_next_column();
        if let Some(armor) = loadout.armor() {
            ui.text(armor.item().name.to_string());
            if event.is_none() {
                let slot = ContainerSlot::Loadout(EquipmentSlot::Armor);
                event = InteractEvent::from_ui(ui, entity, slot);
            }
        }
        for slot in GeneralEquipmentSlot::iter() {
            // TODO: Maybe we should handle rings differently in the engine?
            // Special handling for the ring slots
            if matches!(slot, GeneralEquipmentSlot::Ring(_)) {
                continue;
            }

            ui.table_next_column();
            ui.text(slot.to_string());
            ui.table_next_column();

            if let Some(item) = loadout.item_in_slot(&slot) {
                ui.text(item.item.name.to_string());

                if event.is_none() {
                    let equipment_slot = EquipmentSlot::General(slot);
                    event =
                        InteractEvent::from_ui(ui, entity, ContainerSlot::Loadout(equipment_slot));
                }
            }
        }
        // Render ring slots separately
        for ring_number in 0..2 {
            let slot = GeneralEquipmentSlot::Ring(ring_number);
            ui.table_next_column();
            ui.text(slot.to_string());
            ui.table_next_column();
            if let Some(item) = loadout.item_in_slot(&slot) {
                ui.text(item.item.name.to_string());
                if event.is_none() {
                    let equipment_slot = EquipmentSlot::General(slot);
                    event =
                        InteractEvent::from_ui(ui, entity, ContainerSlot::Loadout(equipment_slot));
                }
            }
        }

        table.end();
    }

    event
}

pub fn render_loadout_inventory(ui: &imgui::Ui, world: &mut World, entity: Entity) {
    if let Some(event) = render_loadout(ui, world, entity) {
        // Handle loadout interaction event
        println!("Loadout interaction: {:?}", event);

        let ContainerSlot::Loadout(slot) = event.slot else {
            return;
        };

        match event.mode {
            InteractMode::RightClick => {
                // Handle right-click on loadout item
                println!("Right-clicked on loadout item: {:?}", event.slot);
            }

            InteractMode::DoubleClick => {
                let result = systems::inventory::unequip(world, entity, &slot);
                if let Ok(Some(item)) = result {
                    println!("Unequipped item: {:?}", item);
                    systems::inventory::add(world, entity, item);
                } else {
                    println!("Failed to unequip item from slot: {:?}", slot);
                }
            }

            InteractMode::Drag => {
                // Handle drag on loadout item
                println!("Dragging loadout item: {:?}", event.slot);
            }
        }
    }

    if let Some(event) = render_inventory(ui, world, entity) {
        // Handle inventory interaction event
        println!("Inventory interaction: {:?}", event);

        let ContainerSlot::Inventory(index) = event.slot else {
            return;
        };
        let item = systems::helpers::get_component::<Inventory>(world, entity)
            .items()
            .get(index)
            .cloned()
            .unwrap();

        match event.mode {
            InteractMode::RightClick => {
                // Handle right-click on inventory item
                println!("Right-clicked on inventory item: {:?}", item.item().name);
            }

            InteractMode::DoubleClick => {
                // Try to equip the item
                let result = systems::inventory::equip(world, entity, item);
                match result {
                    Ok(unequipped_items) => {
                        systems::inventory::remove(world, entity, index);
                        for unequipped_item in unequipped_items {
                            println!("Unequipped item: {:?}", unequipped_item);
                            systems::inventory::add(world, entity, unequipped_item);
                        }
                    }
                    Err(err) => {
                        println!("Failed to equip item: {:?}", err);
                    }
                }
            }

            InteractMode::Drag => {
                // Handle drag on inventory item
                println!("Dragging inventory item: {:?}", item.item().name);
            }
        }
    }
}
