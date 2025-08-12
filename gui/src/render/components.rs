use std::{collections::HashMap, vec};

use glutin::context;
use hecs::{Entity, World};
use nat20_rs::{
    components::{
        ability::{Ability, AbilityScore, AbilityScoreMap},
        actions::{
            action::{ActionKindResult, ActionResult},
            targeting::TargetTypeInstance,
        },
        d20_check::{D20CheckDC, D20CheckResult, RollMode},
        damage::{
            AttackRollResult, DamageComponentMitigation, DamageComponentResult,
            DamageMitigationEffect, DamageMitigationResult, DamageResistances, DamageRoll,
            DamageRollResult, MitigationOperation,
        },
        effects::effects::{Effect, EffectDuration},
        hit_points::HitPoints,
        id::{FeatId, RaceId, SpellId, SubraceId},
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
        proficiency::{Proficiency, ProficiencyLevel},
        race::Race,
        resource::ResourceMap,
        saving_throw::SavingThrowSet,
        skill::{Skill, SkillSet, skill_ability},
        spells::spellbook::{SpellSlotsMap, Spellbook},
    },
    entities::character::CharacterTag,
    registry, systems,
};
use std::collections::HashSet;
use strum::IntoEnumIterator;

use crate::{
    render::{
        text::{TextKind, TextSegment, TextSegments, indent_text},
        utils::{
            ImguiRenderable, ImguiRenderableMut, ImguiRenderableMutWithContext,
            ImguiRenderableWithContext, SELECTED_BUTTON_COLOR, render_empty_button,
        },
    },
    table_with_columns,
};

pub enum ModifierSetRenderMode {
    Line,
    List(u8),
    Hoverable,
}

fn sign(value: i32) -> &'static str {
    if value >= 0 { "+" } else { "-" }
}

impl ImguiRenderableWithContext<ModifierSetRenderMode> for ModifierSet {
    fn render_with_context(&self, ui: &imgui::Ui, mode: ModifierSetRenderMode) {
        match mode {
            ModifierSetRenderMode::Line => {
                if self.is_empty() {
                    return;
                }
                let mut segments = Vec::new();
                for (source, value) in self.iter() {
                    if value == &0 {
                        continue;
                    }
                    segments.push((
                        format!("{} {}", sign(*value), value.abs()),
                        TextKind::Normal,
                    ));
                    segments.push((format!("({})", source), TextKind::Details));
                }
                TextSegments::new(segments).render(ui);
            }

            ModifierSetRenderMode::List(indent_level) => {
                for (source, value) in self.iter() {
                    if value == &0 {
                        continue;
                    }
                    TextSegments::new(vec![
                        (format!("{}{}", sign(*value), value.abs()), TextKind::Normal),
                        (source.to_string(), TextKind::Details),
                    ])
                    .with_indent(indent_level)
                    .render(ui);
                }
            }

            ModifierSetRenderMode::Hoverable => {
                let total = format!("{}{}", sign(self.total()), self.total().abs());
                ui.text(total);
                if self.is_empty() {
                    return;
                }
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text(format!("Total: {}", self.total()));
                        self.render_with_context(ui, ModifierSetRenderMode::List(1));
                    });
                }
            }
        }
    }
}

impl ImguiRenderable for CharacterLevels {
    fn render(&self, ui: &imgui::Ui) {
        let mut class_strings = Vec::new();
        for (class_name, level_progression) in self.all_classes() {
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
}

impl ImguiRenderable for HitPoints {
    fn render(&self, ui: &imgui::Ui) {
        let current = self.current();
        let max = self.max();
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
        let foreground =
            ui.push_style_color(imgui::StyleColor::PlotHistogram, [0.7, 0.0, 0.0, 1.0]);
        let background = ui.push_style_color(imgui::StyleColor::FrameBg, [0.2, 0.0, 0.0, 1.0]);

        imgui::ProgressBar::new(hp_fraction)
            .size([150.0, bar_height])
            .overlay_text(&hp_text)
            .build(ui);

        // Pop the style colors
        foreground.pop();
        background.pop();
    }
}

fn proficiency_icon(proficiency: &ProficiencyLevel) -> &'static str {
    match proficiency {
        ProficiencyLevel::None => "",
        ProficiencyLevel::Half => "½",
        ProficiencyLevel::Proficient => "*",
        ProficiencyLevel::Expertise => "**",
    }
}

impl ImguiRenderableWithContext<&str> for Proficiency {
    fn render_with_context(&self, ui: &imgui::Ui, context: &str) {
        let level = self.level();
        if level != &ProficiencyLevel::None {
            ui.text(proficiency_icon(level));
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    let mut segments = vec![
                        (format!("{}", level), TextKind::Normal),
                        (format!("({})", self.source()), TextKind::Details),
                    ];
                    if !context.is_empty() {
                        segments.push((context.to_string(), TextKind::Details));
                    }
                    TextSegments::new(segments).render(ui);
                });
            }
        }
    }
}

impl ImguiRenderable for Proficiency {
    fn render(&self, ui: &imgui::Ui) {
        self.render_with_context(ui, &"");
    }
}

impl ImguiRenderable for AbilityScore {
    fn render(&self, ui: &imgui::Ui) {
        ui.separator_with_text(self.ability.to_string());

        let modifier = self.ability_modifier().total();
        TextSegments::new(vec![
            (format!("Total: {}", self.total()), TextKind::Normal),
            (
                format!("(Modifier: {}{})", sign(modifier), modifier),
                TextKind::Details,
            ),
        ])
        .render(ui);

        TextSegments::new(vec![
            (self.base.to_string(), TextKind::Normal),
            ("Base".to_string(), TextKind::Details),
        ])
        .with_indent(1)
        .render(ui);

        self.modifiers
            .render_with_context(ui, ModifierSetRenderMode::List(1));
    }
}

impl ImguiRenderableWithContext<(&World, Entity)> for AbilityScoreMap {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&World, Entity)) {
        ui.separator_with_text("Abilities");

        let saving_throws = systems::helpers::get_component::<SavingThrowSet>(context.0, context.1);

        let style = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.5, 0.5]));
        for (i, ability) in Ability::iter().enumerate() {
            let ability_score = self.get(ability);
            let saving_throw_proficiency = saving_throws.get(ability).proficiency();

            if i > 0 {
                ui.same_line();
            }
            ui.button_with_size(
                format!("{}\n{}", ability.acronym(), ability_score.total()),
                [30.0, 30.0],
            );
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ability_score.render(ui);

                    ui.separator_with_text("Saving Throw");

                    if saving_throw_proficiency.level() != &ProficiencyLevel::None {
                        TextSegments::new(vec![
                            (
                                format!("{}", saving_throw_proficiency.level()),
                                TextKind::Normal,
                            ),
                            (
                                format!("({})", saving_throw_proficiency.source()),
                                TextKind::Details,
                            ),
                        ])
                        .render(ui);
                    }
                    let result = saving_throws.check(ability, context.0, context.1);
                    let modifiers = &result.modifier_breakdown;
                    let total = modifiers.total();
                    ui.text(format!("Bonus: {}{}", sign(total), total.abs()));
                    modifiers.render_with_context(ui, ModifierSetRenderMode::List(1));
                });
            }
        }
        style.pop();
    }
}

impl ImguiRenderableWithContext<(&World, Entity)> for SkillSet {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&World, Entity)) {
        // Empty column is for proficiency
        if let Some(table) = table_with_columns!(ui, "Skills", "", "Skill", "Bonus") {
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
                    ui.text_colored([0.7, 0.7, 0.7, 1.0], &label);
                    prev_ability = ability;

                    ui.table_next_column();
                }

                // Proficiency column
                ui.table_next_column();
                let proficiency = self.get(skill).proficiency();
                proficiency.render(ui);
                // Skill name
                ui.table_next_column();
                ui.text(skill.to_string());
                // Bonus column
                ui.table_next_column();
                // TODO: Avoid doing an actual skill check here every time
                let result = self.check(skill, context.0, context.1);
                result
                    .modifier_breakdown
                    .render_with_context(ui, ModifierSetRenderMode::Hoverable);
            }

            table.end();
        }
    }
}

static EMPTY_RESOURCE_ICON: &str = "X"; // Placeholder for empty resource icon
static FILLED_RESOURCE_ICON: &str = "O"; // Placeholder for filled resource icon

impl ImguiRenderable for ResourceMap {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_with_columns!(ui, "Resources", "Resource", "Count", "Recharge") {
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

fn spell_level_roman_numeral(level: u8) -> &'static str {
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
                spell_level_roman_numeral(spell.base_level())
            )) {
                self.unprepare_spell(spell_id);
            }
            rendered += 1;
        }
        for i in rendered..self.max_prepared_spells() {
            render_empty_button(ui, &format!("Empty##{}", i));
        }

        ui.separator_with_text("All Spells");
        if let Some(table) = table_with_columns!(ui, "Spells", "Level", "Spells", "Slots") {
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
                ui.text(spell_level_roman_numeral(level));

                // Spells column
                ui.table_next_column();
                if let Some(spells) = spells_by_level.get(&level) {
                    for spell_id in spells {
                        // let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
                        let label = spell_id.to_string();
                        let is_prepared = self.is_spell_prepared(spell_id);

                        // You can set different colors here based on "prepared" status
                        let style_color =
                            if is_prepared {
                                Some(ui.push_style_color(
                                    imgui::StyleColor::Button,
                                    SELECTED_BUTTON_COLOR,
                                ))
                            } else {
                                None
                            };

                        if ui.button(label) {
                            self.prepare_spell(spell_id);
                        }

                        if let Some(color) = style_color {
                            color.pop();
                        }

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

impl ImguiRenderable for SpellSlotsMap {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_with_columns!(ui, "Spell Slots", "Level", "Slots") {
            let mut sorted_levels: Vec<_> = self.keys().cloned().collect();
            sorted_levels.sort();
            for level in sorted_levels {
                let slots = self.get(&level).unwrap();
                ui.table_next_column();
                ui.text(spell_level_roman_numeral(level));
                ui.table_next_column();
                ui.text(format!("{}/{}", slots.current(), slots.maximum()));
            }
            table.end();
        }
    }
}

impl ImguiRenderable for Vec<Effect> {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_with_columns!(ui, "Effects", "Effect", "Source") {
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

impl ImguiRenderable for Vec<FeatId> {
    fn render(&self, ui: &imgui::Ui) {
        if let Some(table) = table_with_columns!(ui, "Feats", "Feat") {
            for feat in self {
                ui.table_next_column();
                ui.text(feat.to_string());
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Placeholder for feat details");
                    });
                }
            }
            table.end();
        }
    }
}

struct LoadoutRenderContext {
    ability_scores: AbilityScoreMap,
    wielding_both_hands: HashMap<WeaponType, bool>,
}

impl ImguiRenderableMutWithContext<&LoadoutRenderContext> for Loadout {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: &LoadoutRenderContext) {
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
        if let Some(table) = table_with_columns!(ui, "Equipment", "Slot", "Item") {
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

fn render_item_misc(ui: &imgui::Ui, item: &Item) {
    ui.text_colored([0.7, 0.7, 0.7, 1.0], &item.description);
    // Fake right-aligned text for weight and value
    let text = format!("{} kg, {} gold", item.weight, item.value);
    let text_width = ui.calc_text_size(&text)[0];
    let available_width = ui.content_region_avail()[0];
    ui.set_cursor_pos([available_width - text_width, ui.cursor_pos()[1] + 10.0]);
    ui.text(text);
}

impl ImguiRenderableWithContext<&LoadoutRenderContext> for Weapon {
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
            indent_text(ui, 1);
            TextSegment::new(
                &component.to_string(),
                TextKind::Damage(component.damage_type),
            )
            .render(ui);
        }
    }
}

impl ImguiRenderable for DamageComponentResult {
    fn render(&self, ui: &imgui::Ui) {
        TextSegments::new(vec![
            (
                &format!("{} {}", self.result.subtotal, self.damage_type),
                TextKind::Damage(self.damage_type),
            ),
            (
                &format!(
                    "({} ({}d{})",
                    self.result.rolls.iter().sum::<u32>(),
                    self.result.rolls.len(),
                    self.result.die_size as u32,
                ),
                TextKind::Details,
            ),
        ])
        .render(ui);
        if !self.result.modifiers.is_empty() {
            ui.same_line();
            self.result
                .modifiers
                .render_with_context(ui, ModifierSetRenderMode::Line);
        }
        ui.same_line();
        TextSegment::new(")", TextKind::Details).render(ui);
    }
}

impl ImguiRenderable for DamageRollResult {
    fn render(&self, ui: &imgui::Ui) {
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                ui.same_line();
                ui.text("+");
                ui.same_line();
            }
            component.render(ui);
        }
        ui.same_line();
        ui.text(format!("= {}", self.total));
    }
}

impl ImguiRenderable for D20CheckResult {
    fn render(&self, ui: &imgui::Ui) {
        let mut segments = vec![
            (self.selected_roll.to_string(), TextKind::Normal),
            ("(1d20)".to_string(), TextKind::Details),
        ];
        if self.advantage_tracker.roll_mode() != RollMode::Normal {
            segments.push((
                format!(
                    " ({}, {}, {:?})",
                    self.rolls[0],
                    self.rolls[1],
                    self.advantage_tracker.roll_mode()
                ),
                TextKind::Details,
            ));
        }
        if self.is_crit {
            segments.push(("(Critical Success!)".to_string(), TextKind::Normal));
        }
        if self.is_crit_fail {
            segments.push(("(Critical Failure!)".to_string(), TextKind::Normal));
        }
        TextSegments::new(segments).render(ui);
        if !self.modifier_breakdown.is_empty() {
            ui.same_line();
            self.modifier_breakdown
                .render_with_context(ui, ModifierSetRenderMode::Line);
        }
        ui.same_line();
        ui.text(format!("= {}", self.total));
    }
}

impl ImguiRenderable for AttackRollResult {
    fn render(&self, ui: &imgui::Ui) {
        self.roll_result.render(ui);
    }
}

impl ImguiRenderable for DamageComponentMitigation {
    fn render(&self, ui: &imgui::Ui) {
        let text_kind = TextKind::Damage(self.damage_type);

        if self.original.subtotal == self.after_mods {
            // No mitigation applied
            TextSegment::new(
                &format!("{} {}", self.original.subtotal, self.damage_type),
                text_kind,
            )
            .render(ui);
            return;
        }

        let mut amount = self.original.subtotal.to_string();
        for modifier in &self.modifiers {
            let explanation = match modifier.operation {
                MitigationOperation::FlatReduction(_) => format!("{}", modifier.source),
                _ => format!("{:?}", modifier.operation),
            };
            amount = format!("({} {} ({}))", amount, modifier.operation, explanation);
        }
        TextSegments::new(vec![
            (
                format!("{} {}", self.after_mods, self.damage_type),
                text_kind,
            ),
            (amount, TextKind::Details),
        ])
        .render(ui);
    }
}

impl ImguiRenderableWithContext<u8> for ActionResult {
    fn render_with_context(&self, ui: &imgui::Ui, context: u8) {
        let indent_level = context;

        let target_name = match &self.target {
            TargetTypeInstance::Entity(entity) => entity.name(),
            TargetTypeInstance::Point(point) => todo!(),
            TargetTypeInstance::Area(area_shape) => {
                todo!()
            }
            TargetTypeInstance::None => todo!(),
        };

        match &self.result {
            ActionKindResult::UnconditionalDamage {
                damage_roll,
                damage_taken,
            } => {
                damage_taken.render_with_context(
                    ui,
                    (&target_name, indent_level + 1, "took no damage", None),
                );
            }

            ActionKindResult::AttackRollDamage {
                attack_roll,
                armor_class,
                damage_roll,
                damage_taken,
            } => {
                damage_taken.render_with_context(
                    ui,
                    (
                        &target_name,
                        indent_level + 1,
                        "was not hit",
                        Some(attack_roll.clone()),
                    ),
                );

                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        TextSegment::new(format!("{}'s", target_name), TextKind::Target).render(ui);
                        ui.same_line();
                        ui.text("Armor Class:");
                        ui.same_line();
                        // TODO: New type for armor class
                        armor_class.render_with_context(ui, ModifierSetRenderMode::Line);

                        ui.text("");
                        ui.text("Attack Roll:");
                        ui.same_line();
                        attack_roll.render(ui);

                        if let Some(damage_taken) = damage_taken {
                            ui.text("");
                            ui.text("Damage Roll:");
                            ui.same_line();
                            damage_roll.render(ui);

                            ui.text("");
                            ui.text("Damage Taken:");
                            ui.same_line();
                            damage_taken.render(ui);
                        } else {
                            ui.text(format!("Attack did not hit. Attack roll ({}) was less than Armor Class ({})", 
                                attack_roll.roll_result.total, armor_class.total()));
                        }
                    });
                }
            }

            ActionKindResult::SavingThrowDamage {
                saving_throw_dc,
                saving_throw_result,
                half_damage_on_save,
                damage_roll,
                damage_taken,
            } => todo!(),

            ActionKindResult::UnconditionalEffect { effect, applied } => todo!(),

            ActionKindResult::SavingThrowEffect {
                saving_throw,
                effect,
                applied,
            } => todo!(),

            ActionKindResult::BeneficialEffect { effect, applied } => {
                TextSegments::new(vec![
                    (target_name, TextKind::Target),
                    ("gained effect", TextKind::Normal),
                    (&effect.to_string(), TextKind::Effect),
                ])
                .with_indent(indent_level + 1)
                .render(ui);
            }

            ActionKindResult::Healing { healing } => {
                TextSegments::new(vec![
                    (target_name, TextKind::Target),
                    ("was healed for", TextKind::Normal),
                    (&format!("{} HP", healing.subtotal), TextKind::Healing),
                ])
                .with_indent(indent_level + 1)
                .render(ui);
            }

            ActionKindResult::Utility => todo!(),

            ActionKindResult::Composite { actions } => todo!(),

            ActionKindResult::Custom {} => todo!(),
        }
    }
}

impl ImguiRenderableWithContext<(&str, u8)> for DamageComponentMitigation {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&str, u8)) {
        let (target_name, indent_level) = context;

        TextSegments::new(vec![
            (target_name, TextKind::Target),
            ("was hit for", TextKind::Normal),
            (
                &format!("{} {} damage", self.after_mods, self.damage_type),
                TextKind::Damage(self.damage_type),
            ),
        ])
        .with_indent(indent_level)
        .render(ui);
    }
}

impl ImguiRenderableWithContext<(&str, u8, &str, Option<AttackRollResult>)>
    for Option<DamageMitigationResult>
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        context: (&str, u8, &str, Option<AttackRollResult>),
    ) {
        let (target_name, indent_level, no_damage_text, attack_roll) = context;
        ui.group(|| match self {
            Some(result) => {
                for component in &result.components {
                    component.render_with_context(ui, (target_name, indent_level));
                }
            }
            None => {
                let mut segments = vec![
                    (target_name.to_string(), TextKind::Target),
                    (no_damage_text.to_string(), TextKind::Normal),
                ];
                if let Some(attack_roll) = attack_roll {
                    if attack_roll.roll_result.is_crit {
                        segments.push(("(Critical Hit!)".to_string(), TextKind::Details));
                    } else if attack_roll.roll_result.is_crit_fail {
                        segments.push(("(Critical Miss!)".to_string(), TextKind::Details));
                    }
                }
                TextSegments::new(segments)
                    .with_indent(indent_level)
                    .render(ui);
            }
        });
    }
}

impl ImguiRenderable for DamageMitigationResult {
    fn render(&self, ui: &imgui::Ui) {
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                ui.same_line();
                ui.text("+");
                ui.same_line();
            }
            component.render(ui);
        }
        ui.same_line();
        ui.text(format!("= {}", self.total));
    }
}

impl ImguiRenderable for D20CheckDC<Ability> {
    fn render(&self, ui: &imgui::Ui) {
        self.dc.render_with_context(ui, ModifierSetRenderMode::Line);
        ui.same_line();
        TextSegments::new(vec![
            (format!("({})", self.key), TextKind::Ability),
            (format!("= {}", self.dc.total()), TextKind::Normal),
        ])
        .render(ui);
    }
}

impl ImguiRenderable for D20CheckDC<Skill> {
    fn render(&self, ui: &imgui::Ui) {
        self.dc.render_with_context(ui, ModifierSetRenderMode::Line);
        ui.same_line();
        TextSegments::new(vec![
            (format!("({})", self.key), TextKind::Skill),
            (format!("= {}", self.dc.total()), TextKind::Normal),
        ])
        .render(ui);
    }
}

pub fn render_race(ui: &imgui::Ui, world: &World, entity: Entity) {
    let race = systems::helpers::get_component::<Option<RaceId>>(world, entity);
    let subrace = systems::helpers::get_component::<Option<SubraceId>>(world, entity);
    let text = if subrace.is_some() {
        subrace.as_ref().unwrap().to_string()
    } else if race.is_some() {
        race.as_ref().unwrap().to_string()
    } else {
        "".to_string()
    };
    TextSegment::new(text, TextKind::Details).render(ui);
}

impl ImguiRenderable for DamageResistances {
    fn render(&self, ui: &imgui::Ui) {
        ui.separator_with_text("Restistances");

        if self.is_empty() {
            ui.text("None");
            return;
        }

        if let Some(table) = table_with_columns!(ui, "Resistances", "Type", "Effect") {
            for (damage_type, resistances) in self.effects.iter() {
                if resistances.is_empty() {
                    continue; // Skip empty resistances
                }

                // TODO: Multiple resistances for the same damage type?
                let effective_resistance = self.effective_resistance(*damage_type).unwrap();

                ui.table_next_column();
                TextSegment::new(damage_type.to_string(), TextKind::Damage(*damage_type))
                    .render(ui);

                ui.table_next_column();

                effective_resistance.render(ui);
            }
            table.end();
        }
    }
}

impl ImguiRenderable for DamageMitigationEffect {
    fn render(&self, ui: &imgui::Ui) {
        ui.text(format!("{:?}", self.operation));
        if ui.is_item_hovered() {
            ui.tooltip(|| {
                TextSegment::new(format!("{}", self.source), TextKind::Details).render(ui);
            });
        }
    }
}

pub enum CharacterRenderMode {
    Full,
    Compact,
}

impl ImguiRenderableWithContext<(&mut World, CharacterRenderMode)> for (Entity, CharacterTag) {
    fn render_with_context(&self, ui: &imgui::Ui, context: (&mut World, CharacterRenderMode)) {
        let (entity, _) = *self;
        let (world, mode) = context;

        match mode {
            CharacterRenderMode::Full => todo!(),
            CharacterRenderMode::Compact => {
                ui.text(systems::helpers::get_component::<String>(world, entity).to_string());
                systems::helpers::get_component::<CharacterLevels>(world, entity).render(ui);
                systems::helpers::get_component::<HitPoints>(world, entity).render(ui);
            }
        }
    }
}

impl ImguiRenderableMutWithContext<&mut World> for (Entity, CharacterTag) {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, world: &mut World) {
        let (entity, _) = *self;
        ui.text(format!("ID: {:?}", entity));

        render_race(ui, world, entity);
        systems::helpers::get_component::<CharacterLevels>(world, entity).render(ui);
        systems::helpers::get_component::<HitPoints>(world, entity).render(ui);
        systems::helpers::get_component::<AbilityScoreMap>(world, entity)
            .render_with_context(ui, (world, entity));
        systems::helpers::get_component::<DamageResistances>(world, entity).render(ui);

        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs{:?}", entity)) {
            if let Some(tab) = ui.tab_item("Skills") {
                systems::helpers::get_component::<SkillSet>(world, entity)
                    .render_with_context(ui, (world, entity));
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Inventory") {
                let mut wielding_both_hands = HashMap::new();
                for weapon_type in WeaponType::iter() {
                    wielding_both_hands.insert(
                        weapon_type.clone(),
                        systems::helpers::get_component::<Loadout>(world, entity)
                            .is_wielding_weapon_with_both_hands(&weapon_type),
                    );
                }

                let context = LoadoutRenderContext {
                    ability_scores: systems::helpers::get_component_clone::<AbilityScoreMap>(
                        world, entity,
                    ),
                    wielding_both_hands,
                };

                systems::helpers::get_component_mut::<Loadout>(world, entity)
                    .render_mut_with_context(ui, &context);

                tab.end();
            }

            if let Some(tab) = ui.tab_item("Spellbook") {
                systems::helpers::get_component_mut::<Spellbook>(world, entity).render_mut(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Resources") {
                systems::helpers::get_component::<ResourceMap>(world, entity).render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Effects") {
                systems::effects::effects(world, entity).render(ui);
                tab.end();
            }

            if let Some(tab) = ui.tab_item("Feats") {
                systems::helpers::get_component::<Vec<FeatId>>(world, entity).render(ui);
                tab.end();
            }

            tab_bar.end();
        }
    }
}
