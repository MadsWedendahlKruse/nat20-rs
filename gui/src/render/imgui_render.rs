use std::collections::HashMap;

use hecs::{Entity, World};
use imgui::{Id, TableColumnFlags, TableColumnSetup, TableFlags};
use nat20_rs::{
    components::{
        ability::{self, Ability, AbilityScoreSet},
        damage::{DamageRoll, DamageType},
        effects::effects::{Effect, EffectDuration},
        hit_points::HitPoints,
        id::{CharacterId, SpellId},
        items::{
            equipment::{
                equipment::{EquipmentSlot, GeneralEquipmentSlot, HandSlot},
                loadout::Loadout,
                weapon::{Weapon, WeaponType},
            },
            item::Item,
        },
        level::CharacterLevels,
        modifier::ModifierSet,
        proficiency::Proficiency,
        resource::ResourceMap,
        saving_throw::SavingThrowSet,
        skill::{Skill, SkillSet, skill_ability},
        spells::spellbook::Spellbook,
    },
    registry, systems,
};
use std::collections::HashSet;
use strum::IntoEnumIterator;

use crate::render;

pub trait ImguiRenderable {
    fn render(&self, ui: &imgui::Ui);
}

pub trait ImguiRenderableMut {
    fn render_mut(&mut self, ui: &imgui::Ui);
}

pub trait ImguiRenderableWithContext<C> {
    fn render_with_context(&self, ui: &imgui::Ui, context: &C);
}

pub trait ImguiRenderableMutWithContext<C> {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: &C);
}

// impl<T, C> ImguiRenderableWithContext<C> for T
// where
//     T: ImguiRenderable,
// {
//     fn render_with_context(&self, ui: &imgui::Ui, _context: &C) {
//         self.render(ui);
//     }
// }

fn table_column(name: &str) -> TableColumnSetup<&str> {
    TableColumnSetup {
        name,
        flags: TableColumnFlags::NO_HIDE,
        init_width_or_weight: 0.0,
        user_id: Id::default(),
    }
}

macro_rules! table_columns {
    ($ui:expr, $table_name:expr, $( $col_name:expr ),+ $(,)? ) => {{
    let columns = [ $( table_column($col_name) ),+ ];
    $ui.begin_table_header_with_flags(
        $table_name,
        columns,
        TableFlags::SIZING_FIXED_FIT
        | TableFlags::BORDERS
        | TableFlags::ROW_BG
        | TableFlags::NO_HOST_EXTEND_X,
    )
    }};
}

fn render_item_with_modifier(ui: &imgui::Ui, total: &str, modifiers: &ModifierSet) {
    ui.text(total);
    if ui.is_item_hovered() && !modifiers.is_empty() {
        ui.tooltip_text(modifiers.to_string());
    }
}

fn render_classes(ui: &imgui::Ui, world: &World, entity: Entity) {
    let mut class_strings = Vec::new();
    let character_levels = systems::helpers::get_component::<CharacterLevels>(world, entity);
    for (class_name, level_progression) in character_levels.all_classes() {
        let level = level_progression.level();
        let class_str = if let Some(subclass_name) = level_progression.subclass() {
            format!("Level {} {} {}", level, subclass_name.name, class_name)
        } else {
            format!("Level {} {}", level, class_name)
        };
        class_strings.push(class_str);
    }
    let all_classes = class_strings.join(", ");
    ui.text(all_classes);
}

fn render_health_bar(ui: &imgui::Ui, world: &World, entity: Entity) {
    let hit_points = systems::helpers::get_component::<HitPoints>(world, entity);
    let current = hit_points.current();
    let max = hit_points.max();
    let hp_fraction = current as f32 / max as f32;
    let hp_text = format!("{} / {}", current, max);

    // Reserve vertical space for the taller widget (the progress bar)
    let line_height = ui.text_line_height_with_spacing();
    let bar_height = line_height;
    let y_offset = (bar_height - line_height) * 0.5;
    ui.dummy([0.0, y_offset.max(0.0)]); // move down a little if needed

    // "HP" label
    ui.text("HP:");
    ui.same_line();

    // Style colors
    let foreground = ui.push_style_color(imgui::StyleColor::PlotHistogram, [0.7, 0.0, 0.0, 1.0]);
    let background = ui.push_style_color(imgui::StyleColor::FrameBg, [0.2, 0.0, 0.0, 1.0]);

    imgui::ProgressBar::new(hp_fraction)
        .size([150.0, bar_height])
        .overlay_text(&hp_text)
        .build(ui);

    // Pop the style colors
    foreground.pop();
    background.pop();
}

fn proficiency_icon(proficiency: &Proficiency) -> &'static str {
    match proficiency {
        Proficiency::None => "",
        Proficiency::Half => "½",
        Proficiency::Proficient => "*",
        Proficiency::Expertise => "**",
    }
}

fn render_proficiency(ui: &imgui::Ui, proficiency: &Proficiency, extra_text: &str) {
    if proficiency != &Proficiency::None {
        ui.text(proficiency_icon(proficiency));
        if ui.is_item_hovered() {
            ui.tooltip_text(format!("{}{}", proficiency, extra_text));
        }
    }
}

// TODO: Probably only works if all entities are characters
impl ImguiRenderableMut for (&mut World, Entity) {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        let (world, entity) = self;
        let id = systems::helpers::get_component::<CharacterId>(world, *entity).to_string();
        ui.text(format!("ID: {}", id));
        render_classes(ui, world, *entity);
        render_health_bar(ui, world, *entity);

        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{}", id)) {
            if let Some(tab) = ui.tab_item("Abilities") {
                systems::helpers::get_component::<AbilityScoreSet>(world, *entity)
                    .render_with_context(ui, &(world, *entity));
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Skills") {
                systems::helpers::get_component::<SkillSet>(world, *entity)
                    .render_with_context(ui, &(world, *entity));
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Inventory") {
                let mut wielding_both_hands = HashMap::new();
                for weapon_type in WeaponType::iter() {
                    wielding_both_hands.insert(
                        weapon_type.clone(),
                        systems::helpers::get_component::<Loadout>(world, *entity)
                            .is_wielding_weapon_with_both_hands(&weapon_type),
                    );
                }

                let context = LoadoutRenderContext {
                    ability_scores: systems::helpers::get_component_clone::<AbilityScoreSet>(
                        world, *entity,
                    ),
                    wielding_both_hands,
                };

                systems::helpers::get_component_mut::<Loadout>(world, *entity)
                    .render_mut_with_context(ui, &context);

                tab.end();
            }

            if let Some(tab) = ui.tab_item("Spellbook") {
                systems::helpers::get_component_mut::<Spellbook>(world, *entity).render_mut(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Effects") {
                systems::effects::effects(world, *entity).render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Resources") {
                systems::helpers::get_component::<ResourceMap>(world, *entity).render(ui);
                tab.end();
            }

            tab_bar.end();
        }
    }
}

impl ImguiRenderableWithContext<(&World, Entity)> for AbilityScoreSet {
    fn render_with_context(&self, ui: &imgui::Ui, context: &(&World, Entity)) {
        if let Some(table) = table_columns!(
            ui,
            "Abilities",
            "", // Empty column for saving throw proficiency
            "Ability",
            "Score",
            "Modifier",
            "Saving Throw"
        ) {
            for ability in Ability::iter() {
                // Saving throw proficiency column
                ui.table_next_column();
                let saving_throws =
                    systems::helpers::get_component::<SavingThrowSet>(context.0, context.1);
                let proficiency = saving_throws.get(ability).proficiency();
                render_proficiency(ui, proficiency, " (Saving Throw)");
                // Ability name
                ui.table_next_column();
                ui.text(ability.to_string());
                // Ability score
                ui.table_next_column();
                let ability_score = self.get(ability);
                render_item_with_modifier(
                    ui,
                    &ability_score.total().to_string(),
                    &ability_score.modifiers,
                );
                // Ability modifier
                ui.table_next_column();
                let ability_modifier = ability_score.ability_modifier().total();
                let total = if ability_modifier >= 0 {
                    format!("+{}", ability_modifier)
                } else {
                    format!("{}", ability_modifier)
                };
                render_item_with_modifier(ui, &total, &ability_score.ability_modifier());
                // Saving throw
                ui.table_next_column();
                let result = saving_throws.check(ability, context.0, context.1);
                let modifiers = &result.modifier_breakdown;
                let total = modifiers.total();
                let total_str = if total >= 0 {
                    format!("+{}", total)
                } else {
                    format!("{}", total)
                };
                render_item_with_modifier(ui, &total_str, &modifiers);
            }

            table.end();
        }
    }
}

impl ImguiRenderableWithContext<(&World, Entity)> for SkillSet {
    fn render_with_context(&self, ui: &imgui::Ui, context: &(&World, Entity)) {
        // Empty column is for proficiency
        if let Some(table) = table_columns!(ui, "Skills", "", "Skill", "Bonus") {
            // Skills are ordered by ability, so if the ability changes, we can
            // render a separator. Since the first skill is Athletics, we just
            // have to start with anything other than Strength.
            let mut prev_ability = Ability::Charisma;

            for skill in Skill::iter() {
                let ability = skill_ability(skill);

                // If the ability has changed, render a separator
                if ability != prev_ability {
                    ui.table_next_row_with_flags(imgui::TableRowFlags::empty());
                    ui.table_next_column();
                    ui.table_next_column();

                    let label = format!("{}", ability);
                    // ui.set_cursor_pos([ui.cursor_pos()[0] + 10.0, ui.cursor_pos()[1]]);
                    ui.text_colored([0.7, 0.7, 0.7, 1.0], &label);
                    prev_ability = ability;

                    ui.table_next_column();
                }

                // Proficiency column
                ui.table_next_column();
                let proficiency = self.get(skill).proficiency();
                render_proficiency(ui, proficiency, "");
                // Skill name
                ui.table_next_column();
                ui.text(skill.to_string());
                // Bonus column
                ui.table_next_column();
                // TODO: Avoid doing an actual skill check here every time
                let result = self.check(skill, context.0, context.1);
                render_item_with_modifier(
                    ui,
                    &result.modifier_breakdown.total().to_string(),
                    &result.modifier_breakdown,
                );
            }

            table.end();
        }
    }
}

static EMPTY_RESOURCE_ICON: &str = "X"; // Placeholder for empty resource icon
static FILLED_RESOURCE_ICON: &str = "O"; // Placeholder for filled resource icon

impl ImguiRenderable for ResourceMap {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_columns!(ui, "Resources", "Resource", "Count", "Recharge") {
            for (resource_id, resource) in self.iter() {
                // Resource ID column
                ui.table_next_column();
                ui.text(resource_id.to_string());
                // Current uses column
                ui.table_next_column();
                let current = resource.current_uses();
                let max = resource.max_uses();
                ui.text(format!("{}/{}", current, max));
                // let mut text = String::new();
                // for i in (0..max).rev() {
                //     if i < current {
                //         text.push_str(FILLED_RESOURCE_ICON);
                //     } else {
                //         text.push_str(EMPTY_RESOURCE_ICON);
                //     }
                // }
                // ui.text(text);
                // Recharge column
                ui.table_next_column();
                ui.text(format!("{}", resource.recharge_rule()));
            }
            table.end();
        }
    }
}

fn roman_numeral(level: u8) -> &'static str {
    match level {
        0 => "Cantrips",
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        _ => "?",
    }
}

impl ImguiRenderableMut for Spellbook {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        ui.separator_with_text("Cantrips");
        for spell_id in self.all_spells() {
            let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
            if spell.is_cantrip() {
                if ui.button(spell_id.to_string()) {
                    // Maybe click to inspect?
                }
                // ui.same_line();
            }
        }
        ui.separator_with_text("Prepared Spells");
        let prepared_spells: HashSet<SpellId> = self.prepared_spells().clone();
        let mut rendered = 0;
        for spell_id in &prepared_spells {
            let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
            if ui.button(format!(
                "{} ({})",
                spell_id,
                roman_numeral(spell.base_level())
            )) {
                self.unprepare_spell(spell_id);
            }
            rendered += 1;
        }
        for i in rendered..self.max_prepared_spells() {
            // Render fake "Empty" buttons for empty slots
            let disabled = ui.begin_disabled(true);
            let color = ui.push_style_color(imgui::StyleColor::Button, [0.3, 0.3, 0.3, 1.0]);
            // Use a unique ID for each button to avoid conflicts
            ui.button(format!("Empty##{}", i));
            color.pop();
            disabled.end();
        }

        ui.separator_with_text("All Spells");
        if let Some(table) = table_columns!(ui, "Spells", "Level", "Spells", "Slots") {
            // Group spells by level
            let mut spells_by_level: HashMap<u8, Vec<&SpellId>> = HashMap::new();
            let all_spells = self.all_spells().clone();
            for spell_id in &all_spells {
                let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
                spells_by_level
                    .entry(spell.base_level())
                    .or_default()
                    .push(spell_id);
            }

            let max_level = spells_by_level.keys().max().cloned().unwrap_or(0);

            for level in 1..=max_level {
                // Level column
                ui.table_next_column();
                ui.text(roman_numeral(level));

                // Spells column
                ui.table_next_column();
                if let Some(spells) = spells_by_level.get(&level) {
                    for spell_id in spells {
                        // let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
                        let label = spell_id.to_string();
                        let is_prepared = self.is_spell_prepared(spell_id);

                        // You can set different colors here based on "prepared" status
                        let style_color = if is_prepared {
                            ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.6, 0.2, 1.0])
                        } else {
                            ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.2, 0.6, 1.0])
                        };

                        if ui.button(label) {
                            self.prepare_spell(spell_id);
                        }

                        style_color.pop();

                        ui.same_line();
                    }
                }

                // Slots column
                ui.table_next_column();
                let slots = self.spell_slots_for_level(level);
                ui.text(format!("{}/{}", slots.current(), slots.maximum()));
            }
            table.end();
        }
    }
}

impl ImguiRenderable for Vec<Effect> {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_columns!(ui, "Effects", "Effect", "Source") {
            // Sort by duration
            let mut sorted_effects = self.clone();
            sorted_effects.sort_by_key(|effect| effect.duration().clone());

            let mut prev_duration: Option<EffectDuration> = None;

            for effect in &sorted_effects {
                // If the duration has changed, render a separator
                if prev_duration.is_none() || effect.duration() != prev_duration.as_ref().unwrap() {
                    ui.table_next_row_with_flags(imgui::TableRowFlags::empty());
                    ui.table_next_column();

                    let label = format!("{}", effect.duration());
                    // ui.set_cursor_pos([ui.cursor_pos()[0] + 10.0, ui.cursor_pos()[1]]);
                    ui.text_colored([0.7, 0.7, 0.7, 1.0], &label);
                    prev_duration = Some(effect.duration().clone());

                    ui.table_next_column();
                }

                // Effect ID column
                ui.table_next_column();
                ui.text(effect.id().to_string());
                // Source column
                ui.table_next_column();
                ui.text(effect.source().to_string());
            }
            table.end();
        }
    }
}

struct LoadoutRenderContext {
    ability_scores: AbilityScoreSet,
    wielding_both_hands: HashMap<WeaponType, bool>,
}

impl ImguiRenderableMutWithContext<LoadoutRenderContext> for Loadout {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: &LoadoutRenderContext) {
        ui.separator_with_text("Weapons");
        if let Some(table) = table_columns!(ui, "Weapons", "Hand", "Weapon") {
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
                    if let Some(weapon) = self.weapon_in_hand(&weapon_type, &hand) {
                        ui.text(weapon.equipment().item.name.to_string());
                        if ui.is_item_hovered() {
                            ui.tooltip(|| {
                                weapon.render_with_context(ui, context);
                            });
                        }
                    }
                }
            }

            table.end();
        }

        ui.separator_with_text("Equipment");
        if let Some(table) = table_columns!(ui, "Equipment", "Slot", "Item") {
            // Armor is technically not considered equipment, but we can sneak
            // it in here for now
            ui.table_next_column();
            ui.text(format!("{}", EquipmentSlot::Armor));
            ui.table_next_column();
            if let Some(armor) = self.armor() {
                ui.text(armor.item().name.to_string());
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

                if let Some(item) = self.item_in_slot(&slot) {
                    ui.text(item.item.name.to_string());
                }
            }
            // Render ring slots separately
            for ring_number in 0..2 {
                let slot = GeneralEquipmentSlot::Ring(ring_number);
                ui.table_next_column();
                ui.text(slot.to_string());
                ui.table_next_column();
                if let Some(item) = self.item_in_slot(&slot) {
                    ui.text(item.item.name.to_string());
                }
            }

            table.end();
        }
    }
}

fn damage_type_color(damage_type: &DamageType) -> [f32; 4] {
    match damage_type {
        DamageType::Bludgeoning | DamageType::Piercing | DamageType::Slashing => {
            [0.8, 0.8, 0.8, 1.0]
        }
        DamageType::Fire => [1.0, 0.5, 0.0, 1.0],
        DamageType::Cold => [0.0, 1.0, 1.0, 1.0],
        DamageType::Lightning => [0.25, 0.25, 1.0, 1.0],
        DamageType::Acid => [0.0, 1.0, 0.0, 1.0],
        DamageType::Poison => [0.5, 0.9, 0.0, 1.0],
        DamageType::Force => [0.9, 0.0, 0.0, 1.0],
        DamageType::Necrotic => [0.5, 1.0, 0.25, 1.0],
        DamageType::Psychic => [1.0, 0.5, 1.0, 1.0],
        DamageType::Radiant => [1.0, 0.9, 0.0, 1.0],
        DamageType::Thunder => [0.5, 0.0, 1.0, 1.0],
    }
}

fn render_item_misc(ui: &imgui::Ui, item: &Item) {
    ui.text_colored([0.7, 0.7, 0.7, 1.0], &item.description);
    // Fake right-aligned text for weight and value
    let text = format!("{} kg, {} gold", item.weight, item.value);
    let text_width = ui.calc_text_size(&text)[0];
    let available_width = ui.content_region_avail()[0];
    ui.set_cursor_pos([available_width - text_width, ui.cursor_pos()[1] + 10.0]);
    ui.text(text);
}

impl ImguiRenderableWithContext<LoadoutRenderContext> for Weapon {
    fn render_with_context(&self, ui: &imgui::Ui, context: &LoadoutRenderContext) {
        ui.separator_with_text(&self.equipment().item.name);
        let damage_roll = self.damage_roll(
            &context.ability_scores,
            *context.wielding_both_hands.get(self.weapon_type()).unwrap(),
        );
        damage_roll.render(ui);
        ui.separator();
        ui.text(format!("{}", self.category()));
        for property in self.properties() {
            ui.text(format!("{}", property));
        }
        ui.separator();
        render_item_misc(ui, &self.equipment().item);
    }
}

impl ImguiRenderable for DamageRoll {
    fn render(&self, ui: &imgui::Ui) {
        let min_max_rolls = self.min_max_rolls();
        let min_damage = min_max_rolls
            .iter()
            .map(|(min_roll, _, _)| min_roll)
            .sum::<i32>();
        let max_damage = min_max_rolls
            .iter()
            .map(|(_, max_roll, _)| max_roll)
            .sum::<i32>();
        ui.text(format!("{}-{} Damage", min_damage, max_damage));

        let mut damage_components = vec![self.primary.clone()];
        damage_components.extend(self.bonus.clone());

        for component in damage_components {
            ui.text_colored(
                damage_type_color(&component.damage_type),
                format!("\t{}", component.to_string()),
            );
        }
    }
}
