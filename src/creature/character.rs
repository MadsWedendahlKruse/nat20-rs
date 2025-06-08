use std::collections::HashMap;

use crate::{
    combat::{
        action::{CombatAction, CombatActionProvider},
        damage::{
            DamageMitigationEffect, DamageMitigationResult, DamageResistances, DamageRoll,
            DamageRollResult, DamageSource, MitigationOperation,
        },
    },
    creature::{
        classes::class::{Class, SubclassName},
        level_up::{LevelUpError, LevelUpSession},
    },
    effects::effects::Effect,
    items::equipment::{
        armor::Armor,
        equipment::{EquipmentItem, GeneralEquipmentSlot, HandSlot},
        loadout::{Loadout, TryEquipError},
        weapon::{Weapon, WeaponCategory, WeaponType},
    },
    registry::classes::CLASS_REGISTRY,
    spells::{
        spell::{SnapshotError, SpellKindSnapshot, SpellSnapshot},
        spellbook::Spellbook,
    },
    stats::{
        ability::{Ability, AbilityScoreSet},
        d20_check::D20CheckResult,
        modifier::{ModifierSet, ModifierSource},
        proficiency::Proficiency,
        saving_throw::{create_saving_throw_set, SavingThrowSet},
        skill::{create_skill_set, Skill, SkillSet},
    },
    utils::id::CharacterId,
};

use super::{
    classes::{
        class::{ClassName, SpellcastingProgression},
        resources::Resource,
    },
    level_up::{LevelUpChoice, LevelUpSelection},
};

#[derive(Debug)]
pub struct Character {
    id: CharacterId,
    pub name: String,
    classes: HashMap<ClassName, u8>,
    subclasses: HashMap<ClassName, SubclassName>,
    latest_class: Option<ClassName>, // The class that was most recently leveled up
    max_hp: u32,
    current_hp: u32,
    ability_scores: AbilityScoreSet,
    skills: SkillSet,
    saving_throws: SavingThrowSet,
    resistances: DamageResistances,
    // TODO: Might have to make this more granular later (not just martial/simple)
    // TODO: Should it just be a bool (or a set even)? Not sure if you can have expertise in a weapon
    pub weapon_proficiencies: HashMap<WeaponCategory, Proficiency>,
    /// Equipped items
    loadout: Loadout,
    spellbook: Spellbook,
    resources: HashMap<String, Resource>,
    effects: Vec<Effect>,
}

impl Character {
    pub fn new(name: &str) -> Self {
        Self {
            id: CharacterId::new_v4(),
            name: name.to_string(),
            classes: HashMap::new(),
            subclasses: HashMap::new(),
            latest_class: None,
            max_hp: 0,
            current_hp: 0,
            ability_scores: AbilityScoreSet::new(),
            skills: create_skill_set(),
            saving_throws: create_saving_throw_set(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: HashMap::new(),
            loadout: Loadout::new(),
            spellbook: Spellbook::new(),
            resources: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn id(&self) -> CharacterId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn level_up(&mut self) -> LevelUpSession {
        LevelUpSession::new(self)
    }

    pub fn resolve_level_up_choice(
        &mut self,
        choice: LevelUpChoice,
        selection: LevelUpSelection,
    ) -> Result<Vec<LevelUpChoice>, LevelUpError> {
        let mut choices = Vec::new();

        match (&choice, &selection) {
            (LevelUpChoice::Class(classes), LevelUpSelection::Class(class_name)) => {
                if !classes.contains(&class_name) {
                    return Err(LevelUpError::InvalidSelection { choice, selection });
                }

                if let Some(class) = CLASS_REGISTRY.get(&class_name) {
                    // TODO: Do this at the end when subclass, feats etc. are selected

                    // 2: Adjust Hit Points and Hit Point Dice
                    self.update_hp(class);

                    // 3: Record New Class Features
                    choices.extend(self.increment_class_level(&class));

                    // 4: Adjust Proficiency Bonus
                    // This is handled by the `proficiency_bonus` method, so we don't need to do anything here.

                    // 5: Adjust Ability Modifiers
                    // TODO: This would happen when choosing certain Feats (choice mechanism?)
                } else {
                    return Err(LevelUpError::RegistryMissing(*class_name));
                }
            }

            (LevelUpChoice::Subclass(subclasses), LevelUpSelection::Subclass(subclass_name)) => {
                // Sanity check
                if !subclasses.contains(&subclass_name) {
                    return Err(LevelUpError::InvalidSelection { choice, selection });
                }

                self.subclasses
                    .insert(subclass_name.class.clone(), subclass_name.clone());

                // TODO: Subclass choices
            }

            _ => {
                return Err(LevelUpError::ChoiceSelectionMismatch { choice, selection });
            }
        }

        Ok(choices)
    }

    pub fn apply_latest_level(&mut self) {
        if let Some(class_name) = &self.latest_class {
            if let Some(class) = CLASS_REGISTRY.get(class_name) {
                self.apply_class_level(class);
            } else {
                panic!("Tried to apply level for unknown class: {:?}", class_name);
            }
        } else {
            panic!("No latest class set for level up");
        }
    }

    fn apply_class_level(&mut self, class: &Class) {
        let level = *self
            .classes
            .get(&class.name)
            .unwrap_or_else(|| panic!("Class {} not found in character's classes", class.name));

        let subclass_name = self
            .subclass(&class.name)
            .unwrap_or(&SubclassName {
                class: class.name.clone(),
                name: String::new(),
            })
            .clone();

        for effect in class.effects_by_level(level, &subclass_name.name) {
            self.add_effect(effect.clone());
        }

        for resource in class.resources_by_level(level, &subclass_name.name) {
            self.resources
                .entry(resource.kind.clone())
                .and_modify(|r| {
                    r.add_uses(resource.max_uses).unwrap();
                })
                .or_insert(resource);
        }
    }

    pub fn total_level(&self) -> u8 {
        self.classes.iter().map(|(_, level)| *level).sum()
    }

    fn increment_class_level(&mut self, class: &Class) -> Vec<LevelUpChoice> {
        let level = *self.classes.get(&class.name).unwrap_or(&0) + 1;

        // Add or update the class level
        if level == 1 {
            // If it's the first level, add the class to the list
            self.classes.insert(class.name.clone(), level);
        } else {
            // If it's an existing class, update its level
            if let Some(existing_level) = self.classes.get_mut(&class.name) {
                *existing_level = level;
            }
        }

        self.latest_class = Some(class.name.clone());

        class.level_up_choices(level)
    }

    pub fn proficiency_bonus(&self) -> u32 {
        match self.total_level() {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            17..=20 => 6,
            _ => 2, // fallback default
        }
    }

    pub fn classes(&self) -> &HashMap<ClassName, u8> {
        &self.classes
    }

    pub fn subclass(&self, class_name: &ClassName) -> Option<&SubclassName> {
        self.subclasses.get(class_name)
    }

    pub fn spellcaster_levels(&self) -> u8 {
        let mut spellcaster_levels = 0;
        for (class_name, levels) in &self.classes {
            if let Some(class) = CLASS_REGISTRY.get(&class_name) {
                let spellcasting_progression = class.spellcasting_progression(
                    // TODO: Not entirely sure why it's necessary to do it like this
                    self.subclass(class_name)
                        .as_deref()
                        .map_or("", |v| v.name.as_str()),
                );
                spellcaster_levels += match spellcasting_progression {
                    SpellcastingProgression::None => 0,
                    SpellcastingProgression::Full => *levels,
                    SpellcastingProgression::Half => (*levels) / 2,
                    SpellcastingProgression::Third => (*levels) / 3,
                };
            }
        }
        spellcaster_levels
    }

    pub fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub fn hp(&self) -> u32 {
        self.current_hp
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    fn update_hp(&mut self, class: &Class) {
        // TODO: Lot of type casting back and forth here
        let hp_bonus = if self.total_level() == 1 {
            class.hit_die as u32
        } else {
            class.hp_per_level as u32
        };
        let con_mod = self
            .ability_scores
            .get(Ability::Constitution)
            .ability_modifier()
            .total();
        let hp_increase = (hp_bonus as i32 + con_mod).max(1) as u32;
        self.max_hp += hp_increase;
        self.current_hp += hp_increase;
    }

    pub fn take_damage(&mut self, damage_source: &DamageSource) -> Option<DamageMitigationResult> {
        let mut resistances = self.resistances.clone();

        match damage_source {
            DamageSource::WeaponAttack {
                attack_roll_result,
                damage_roll_result,
            } => {
                if !self.loadout().does_attack_hit(&self, attack_roll_result) {
                    // If the attack misses, no damage is applied
                    return None;
                }
                self.take_damage_internal(damage_roll_result, &resistances)
            }

            DamageSource::Spell { snapshot } => {
                match &snapshot {
                    SpellKindSnapshot::Damage { damage_roll } => {
                        return self.take_damage_internal(damage_roll, &resistances);
                    }

                    SpellKindSnapshot::AttackRoll {
                        attack_roll,
                        damage_roll: damage,
                        damage_on_failure,
                    } => {
                        if !self.loadout().does_attack_hit(self, attack_roll) {
                            if let Some(damage_on_failure) = damage_on_failure {
                                return self.take_damage_internal(damage_on_failure, &resistances);
                            }
                            return None;
                        }
                        self.take_damage_internal(damage, &resistances)
                    }

                    SpellKindSnapshot::SavingThrowDamage {
                        saving_throw,
                        half_damage_on_save,
                        damage_roll,
                    } => {
                        let check_result = self.saving_throws.check_dc(saving_throw, self);
                        if check_result.success {
                            if *half_damage_on_save {
                                // Apply half damage on successful save
                                for component in damage_roll.components.iter() {
                                    resistances.add_effect(
                                        component.damage_type,
                                        DamageMitigationEffect {
                                            // TODO: Not sure if this is the best source
                                            source: ModifierSource::Ability(saving_throw.key),
                                            operation: MitigationOperation::Resistance,
                                        },
                                    );
                                }
                                return self.take_damage_internal(&damage_roll, &resistances);
                            }
                            // No damage on successful save
                            return None;
                        }
                        self.take_damage_internal(damage_roll, &resistances)
                    }

                    SpellKindSnapshot::Custom { damage_roll } => {
                        return self.take_damage_internal(damage_roll, &resistances);
                    }

                    _ => {
                        // TODO: Handle this in a more graceful way
                        panic!("Character::take_damage called with unsupported spell snapshot type: {:?}", snapshot);
                    }
                }
            }
        }
    }

    fn take_damage_internal(
        &mut self,
        damage_roll_result: &DamageRollResult,
        resistances: &DamageResistances,
    ) -> Option<DamageMitigationResult> {
        let mitigation_result = resistances.apply(damage_roll_result);
        self.current_hp = (self.current_hp as i32 - mitigation_result.total).max(0) as u32;
        Some(mitigation_result)
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn ability_scores(&self) -> &AbilityScoreSet {
        &self.ability_scores
    }

    pub fn ability_scores_mut(&mut self) -> &mut AbilityScoreSet {
        &mut self.ability_scores
    }

    pub fn skills(&self) -> &SkillSet {
        &self.skills
    }

    pub fn skills_mut(&mut self) -> &mut SkillSet {
        &mut self.skills
    }

    pub fn skill_check(&self, skill: Skill) -> D20CheckResult {
        self.skills.check(skill, self)
    }

    pub fn saving_throws(&self) -> &SavingThrowSet {
        &self.saving_throws
    }

    pub fn saving_throws_mut(&mut self) -> &mut SavingThrowSet {
        &mut self.saving_throws
    }

    pub fn saving_throw(&self, ability: Ability) -> D20CheckResult {
        self.saving_throws.check(ability, self)
    }

    pub fn loadout(&self) -> &Loadout {
        &self.loadout
    }

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        self.add_effects(armor.effects().clone());
        self.loadout.equip_armor(armor)
    }

    pub fn unequip_armor(&mut self) -> Option<Armor> {
        let unequiped_armor = self.loadout.unequip_armor();
        if let Some(armor) = &unequiped_armor {
            self.remove_effects(armor.effects());
        }
        unequiped_armor
    }

    pub fn armor_class(&self) -> ModifierSet {
        self.loadout.armor_class(self)
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Result<Option<EquipmentItem>, TryEquipError> {
        let unequipped_item = self.loadout.equip_item(slot, item)?;
        if let Some(item) = &unequipped_item {
            self.remove_effects(item.effects());
        }
        let effects = self.loadout().item_in_slot(slot).unwrap().effects().clone();
        self.add_effects(effects);
        Ok(unequipped_item)
    }

    pub fn unequip_item(&mut self, slot: GeneralEquipmentSlot) -> Option<EquipmentItem> {
        let unequipped_item = self.loadout.unequip_item(slot);
        if let Some(item) = &unequipped_item {
            self.remove_effects(item.effects());
        }
        unequipped_item
    }

    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        hand: HandSlot,
    ) -> Result<Vec<Weapon>, TryEquipError> {
        let unequipped_weapons = self.loadout.equip_weapon(weapon, hand)?;
        for weapon in &unequipped_weapons {
            self.add_effects(weapon.effects().clone());
        }
        Ok(unequipped_weapons)
    }

    pub fn unequip_weapon(&mut self, weapon_type: &WeaponType, hand: HandSlot) -> Option<Weapon> {
        let unequipped_weapon = self.loadout.unequip_weapon(weapon_type, hand);
        if let Some(weapon) = &unequipped_weapon {
            self.remove_effects(weapon.effects());
        }
        unequipped_weapon
    }

    pub fn attack_roll(&self, weapon_type: &WeaponType, hand: HandSlot) -> D20CheckResult {
        self.loadout.attack_roll(self, weapon_type, hand)
    }

    pub fn damage_roll(&self, weapon_type: &WeaponType, hand: HandSlot) -> DamageRoll {
        let weapon = self
            .loadout
            .weapon_in_hand(weapon_type, hand)
            .expect("Weapon should be equipped in the specified hand");
        weapon.damage_roll(self, hand)
    }

    pub fn spellbook(&self) -> &Spellbook {
        &self.spellbook
    }

    pub fn spellbook_mut(&mut self) -> &mut Spellbook {
        &mut self.spellbook
    }

    pub fn update_spell_slots(&mut self) {
        // TODO: Calculate caster level based on class levels
        let caster_level = self.total_level();
        self.spellbook.update_spell_slots(caster_level);
    }

    pub fn spell_snapshot(
        &self,
        spell_id: &str,
        level: u8,
    ) -> Option<Result<SpellSnapshot, SnapshotError>> {
        self.spellbook
            .get_spell(spell_id)
            .map(|spell| spell.snapshot(self, &level))
    }

    pub fn resistances(&self) -> &DamageResistances {
        &self.resistances
    }

    pub fn resistances_mut(&mut self) -> &mut DamageResistances {
        &mut self.resistances
    }

    pub fn effects(&self) -> &Vec<Effect> {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: Effect) {
        (effect.on_apply)(self);
        self.effects.push(effect);
    }

    pub fn add_effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            self.add_effect(effect);
        }
    }

    pub fn remove_effect(&mut self, effect: &Effect) {
        (effect.on_unapply)(self);
        self.effects.retain(|e| e != effect);
    }

    pub fn remove_effects(&mut self, effects: &Vec<Effect>) {
        for effect in effects {
            self.remove_effect(effect);
        }
    }
}

impl CombatActionProvider for Character {
    fn available_actions(&self) -> Vec<CombatAction> {
        let mut actions = Vec::new();

        for action in self.loadout.available_actions() {
            actions.push(action);
        }

        for action in self.spellbook.available_actions() {
            actions.push(action);
        }

        actions
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new("John Doe")
    }
}
