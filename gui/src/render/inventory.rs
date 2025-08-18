use hecs::{Entity, World};
use nat20_rs::{
    components::items::{
        equipment::{slots::EquipmentSlot, weapon::WeaponKind},
        inventory::{Inventory, ItemContainer, ItemInstance},
        item::{Item, ItemRarity},
    },
    systems,
};
use strum::IntoEnumIterator;

use crate::{
    render::{
        text::{TextKind, item_rarity_color},
        utils::ImguiRenderableWithContext,
    },
    table_with_columns,
};

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

fn render_item_button(ui: &imgui::Ui, item: &Item) -> bool {
    let words = item.name.split_whitespace();
    // Render first three lettes of each word
    let short_name = words
        .map(|word| word.chars().take(3).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    let background_color = item_rarity_color(&item.rarity);
    let highlight_color = background_color.map(|c| (c * 1.2).min(1.0));

    let mut style_tokens = vec![
        ui.push_style_color(imgui::StyleColor::Button, background_color),
        ui.push_style_color(imgui::StyleColor::ButtonHovered, highlight_color),
    ];
    // Common is white, so we don't need to change the text color
    if item.rarity == ItemRarity::Common {
        style_tokens.push(ui.push_style_color(imgui::StyleColor::Text, [0.0, 0.0, 0.0, 1.0]));
    }

    let clicked = ui.button_with_size(short_name, [30.0, 30.0]);

    style_tokens.into_iter().for_each(|token| token.pop());

    clicked
}

static INVENTORY_ITEMS_PER_ROW: usize = 8;

pub fn render_inventory(
    ui: &imgui::Ui,
    world: &mut World,
    entity: Entity,
) -> Option<InteractEvent> {
    ui.separator_with_text("Inventory");

    let inventory = systems::helpers::get_component::<Inventory>(world, entity);
    let mut event = None;
    let items = inventory.items();
    let rows = (items.len() + INVENTORY_ITEMS_PER_ROW) / INVENTORY_ITEMS_PER_ROW;
    let total_items = rows * INVENTORY_ITEMS_PER_ROW;
    for i in 0..total_items {
        if i < items.len() {
            let slot = ContainerSlot::Inventory(i);

            let item_name = items[i].item().name.clone();
            if render_item_button(ui, items[i].item()) {
                // Handle item click (don't think we need to do anything here)
                println!("Clicked on item: {}", item_name);
            }

            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    items[i].render_with_context(ui, (world, entity));
                });
            }

            if event.is_none() {
                event = InteractEvent::from_ui(ui, entity, slot);
            }
        } else {
            // Render empty button for unused slots
            ui.button_with_size(format!("##{}", i), [30.0, 30.0]);
        }

        if (i + 1) % INVENTORY_ITEMS_PER_ROW != 0 && i + 1 < total_items {
            ui.same_line();
        }
    }

    event
}

fn render_loadout(ui: &imgui::Ui, world: &mut World, entity: Entity) -> Option<InteractEvent> {
    let loadout = systems::loadout::loadout(world, entity);
    let mut event = None;

    if let Some(table) = table_with_columns!(ui, "Loadout", "Slot", "Item") {
        for slot in EquipmentSlot::iter() {
            // Slot column
            ui.table_next_column();
            ui.text(slot.to_string());
            // Item column
            ui.table_next_column();
            let item = loadout.item_in_slot(&slot);
            if let Some(item) = item {
                if render_item_button(ui, item.item()) {
                    // Handle item click (don't think we need to do anything here)
                    println!("Clicked on loadout item: {}", item.item().name);
                }

                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        // TODO: Consider implementing a dedicated render method for EquipmentInstance
                        let item_instance: ItemInstance = item.clone().into();
                        item_instance.render_with_context(ui, (world, entity));
                    });
                }

                if event.is_none() {
                    event = InteractEvent::from_ui(ui, entity, ContainerSlot::Loadout(slot));
                }
            } else {
                // Render empty button for unused slots
                ui.button_with_size(format!("##{}", slot), [30.0, 30.0]);
            }
        }

        table.end();
    } else {
        ui.text("No loadout available.");
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
                if let Some(item) = result {
                    println!("Unequipped item: {:?}", item);
                    systems::inventory::add(world, entity, item);
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

impl ImguiRenderableWithContext<(&World, Entity)> for ItemInstance {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&World, Entity)) {
        match self {
            ItemInstance::Weapon(weapon) => {
                weapon.render_with_context(ui, context);
            }
            ItemInstance::Armor(armor) => {
                armor.render_with_context(ui, context);
            }
            _ => {
                ui.text("Placeholder tooltip :^)");
            }
        }
    }
}
