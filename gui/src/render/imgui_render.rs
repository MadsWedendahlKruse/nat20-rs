use std::collections::HashMap;

use imgui::{Id, TableColumnFlags, TableColumnSetup, TableFlags, TreeNodeFlags};
use nat20_rs::{
    creature::character::Character,
    effects::effects::{Effect, EffectDuration},
    items::equipment::{
        equipment::{EquipmentSlot, GeneralEquipmentSlot, HandSlot},
        loadout::Loadout,
        weapon::WeaponType,
    },
    registry,
    resources::resources::Resource,
    spells::spellbook::Spellbook,
    stats::{
        ability::{Ability, AbilityScoreSet},
        modifier::ModifierSet,
        proficiency::Proficiency,
        skill::{Skill, SkillSet, skill_ability},
    },
    utils::id::{ResourceId, SpellId},
};
use std::collections::HashSet;
use strum::IntoEnumIterator;

pub trait ImguiRenderable {
    fn render(&self, ui: &imgui::Ui);
}

pub trait ImguiRenderableMut {
    fn render_mut(&mut self, ui: &imgui::Ui);
}

pub trait ImguiRenderableWithContext<C> {
    fn render_with_context(&self, ui: &imgui::Ui, context: &C);
}

impl<T, C> ImguiRenderableWithContext<C> for T
where
    T: ImguiRenderable,
{
    fn render_with_context(&self, ui: &imgui::Ui, _context: &C) {
        self.render(ui);
    }
}

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

fn render_classes(ui: &imgui::Ui, character: &Character) {
    let mut class_strings = Vec::new();
    for (class_name, level) in character.classes() {
        let class_str = if let Some(subclass_name) = character.subclass(class_name) {
            format!("Level {} {} {}", level, subclass_name.name, class_name)
        } else {
            format!("Level {} {}", level, class_name)
        };
        class_strings.push(class_str);
    }
    let all_classes = class_strings.join(", ");
    ui.text(all_classes);
}

fn render_health_bar(ui: &imgui::Ui, character: &Character) {
    let current = character.hp();
    let max = character.max_hp();
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

impl ImguiRenderableWithContext<Character> for AbilityScoreSet {
    fn render_with_context(&self, ui: &imgui::Ui, context: &Character) {
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
                let proficiency = context.saving_throws().get(ability).proficiency();
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
                let result = context.saving_throw(ability);
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

impl ImguiRenderableWithContext<Character> for SkillSet {
    fn render_with_context(&self, ui: &imgui::Ui, context: &Character) {
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
                let proficiency = context.skills().get(skill).proficiency();
                render_proficiency(ui, proficiency, "");
                // Skill name
                ui.table_next_column();
                ui.text(skill.to_string());
                // Bonus column
                ui.table_next_column();
                let result = context.skill_check(skill);
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

impl ImguiRenderable for HashMap<ResourceId, Resource> {
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

impl ImguiRenderableMut for Loadout {
    fn render_mut(&mut self, ui: &imgui::Ui) {
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
                        ui.text(weapon.name().to_string());
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

impl ImguiRenderableMut for Character {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        ui.text(format!("ID: {}", self.id()));
        render_classes(ui, self);
        render_health_bar(ui, self);

        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{}", self.id())) {
            if let Some(tab) = ui.tab_item("Abilities") {
                self.ability_scores().render_with_context(ui, self);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Skills") {
                self.skills().render_with_context(ui, self);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Inventory") {
                self.loadout_mut().render_mut(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Spellbook") {
                self.spellbook_mut().render_mut(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Effects") {
                self.effects().render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Resources") {
                self.resources().render(ui);
                tab.end();
            }

            tab_bar.end();
        }
    }
}
