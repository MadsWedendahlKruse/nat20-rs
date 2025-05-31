use std::collections::HashMap;

use crate::{
    combat::{
        action::{CombatAction, CombatActionProvider},
        damage::{
            DamageMitigationEffect, DamageMitigationResult, DamageResistances, DamageRoll,
            DamageRollResult, DamageSource, MitigationOperation,
        },
    },
    effects::effects::Effect,
    items::equipment::{
        armor::Armor,
        equipment::{EquipmentItem, GeneralEquipmentSlot, HandSlot},
        loadout::{Loadout, TryEquipError},
        weapon::{Weapon, WeaponCategory, WeaponType},
    },
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

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum CharacterClass {
    Fighter,
    Rogue,
    Wizard,
    Cleric,
    Warlock,
    // Add more as needed
}

#[derive(Debug)]
pub struct Character {
    id: CharacterId,
    pub name: String,
    pub class_levels: HashMap<CharacterClass, u8>,
    max_hp: i32,
    current_hp: i32,
    ability_scores: AbilityScoreSet,
    skills: SkillSet,
    saving_throws: SavingThrowSet,
    resistances: DamageResistances,
    // TODO: Might have to make this more granular later (not just martial/simple)
    // TODO: Should it just be a bool? Not sure if you can have expertise in a weapon
    pub weapon_proficiencies: HashMap<WeaponCategory, Proficiency>,
    /// Equipped items
    loadout: Loadout,
    spellbook: Spellbook,
    effects: Vec<Effect>,
}

impl Character {
    pub fn new(name: &str, class_levels: HashMap<CharacterClass, u8>, max_hp: i32) -> Self {
        Self {
            id: CharacterId::new_v4(),
            name: name.to_string(),
            class_levels,
            max_hp,
            current_hp: max_hp,
            ability_scores: AbilityScoreSet::new(),
            skills: create_skill_set(),
            saving_throws: create_saving_throw_set(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: HashMap::new(),
            loadout: Loadout::new(),
            spellbook: Spellbook::new(),
            effects: Vec::new(),
        }
    }

    pub fn id(&self) -> CharacterId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_class_level(&mut self, class: CharacterClass, levels: u8) {
        *self.class_levels.entry(class).or_insert(0) += levels;
    }

    pub fn total_level(&self) -> u8 {
        self.class_levels.values().copied().sum()
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

    pub fn max_hp(&self) -> i32 {
        self.max_hp
    }

    pub fn hp(&self) -> i32 {
        self.current_hp
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
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
        self.current_hp = (self.current_hp - mitigation_result.total).max(0);
        Some(mitigation_result)
    }

    pub fn heal(&mut self, amount: i32) {
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
        Character::new("John Doe", HashMap::new(), 20)
    }
}
