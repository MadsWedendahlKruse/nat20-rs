use crate::combat::damage::*;
use crate::dice::dice::*;
use crate::stats::ability::*;
use crate::stats::d20_check::*;
use crate::stats::modifier::*;
use crate::stats::proficiency::Proficiency;
use crate::stats::skill::*;

use std::collections::HashMap;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum CharacterClass {
    Fighter,
    Rogue,
    Wizard,
    Cleric,
    // Add more as needed
}

#[derive(Debug)]
pub struct Character {
    name: String,
    class_levels: HashMap<CharacterClass, u8>,
    max_hp: i32,
    current_hp: i32,
    ability_scores: HashMap<Ability, AbilityScore>,
    skills: HashMap<Skill, D20Check>,
    saving_throws: HashMap<Ability, D20Check>,
    resistances: DamageResistances,
}

impl Character {
    pub fn new(
        name: &str,
        class_levels: HashMap<CharacterClass, u8>,
        max_hp: i32,
        ability_scores: HashMap<Ability, AbilityScore>,
        skills: HashMap<Skill, D20Check>,
        saving_throws: HashMap<Ability, D20Check>,
        resistances: DamageResistances,
    ) -> Self {
        // TODO: Default values for ability scores and skills
        Self {
            name: name.to_string(),
            class_levels,
            max_hp,
            current_hp: max_hp,
            ability_scores,
            skills,
            saving_throws,
            resistances,
        }
    }

    pub fn add_class_level(&mut self, class: CharacterClass, levels: u8) {
        *self.class_levels.entry(class).or_insert(0) += levels;
    }

    pub fn total_level(&self) -> u8 {
        self.class_levels.values().copied().sum()
    }

    pub fn proficiency_bonus(&self) -> i32 {
        match self.total_level() {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            17..=20 => 6,
            _ => 2, // fallback default
        }
    }

    pub fn take_damage(&mut self, damage_roll_result: &DamageRollResult) -> DamageMitigationResult {
        let mitigation_result = self.resistances.apply(damage_roll_result);
        self.current_hp = (self.current_hp - mitigation_result.total).max(0);
        mitigation_result
    }

    pub fn heal(&mut self, amount: i32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn ability(&self, ability: Ability) -> &AbilityScore {
        self.ability_scores.get(&ability).unwrap()
    }

    pub fn ability_total(&self, ability: Ability) -> i32 {
        self.ability(ability).total()
    }

    pub fn ability_modifier(&self, ability: Ability) -> ModifierSet {
        self.ability_scores
            .get(&ability)
            .map(|a| a.ability_modifier())
            .unwrap()
    }

    pub fn skill_check(&self, skill: Skill) -> D20CheckResult {
        let mut skill_check = self.skills.get(&skill).unwrap().clone();
        skill_check
            .modifiers
            .add_modifier_set(&self.skill_modifier(skill));
        skill_check.perform()
    }

    pub fn skill_modifier(&self, skill: Skill) -> ModifierSet {
        let skill_check = self.skills.get(&skill).unwrap();
        let ability = skill_ability(skill);
        self.ability_check_modifier_set(ability, skill_check)
    }

    pub fn saving_throw(&self, ability: Ability) -> D20CheckResult {
        let mut saving_throw_check = self.saving_throws.get(&ability).unwrap().clone();
        saving_throw_check
            .modifiers
            .add_modifier_set(&self.saving_throw_modifier(ability));
        saving_throw_check.perform()
    }

    pub fn saving_throw_modifier(&self, ability: Ability) -> ModifierSet {
        let saving_throw_check = self.saving_throws.get(&ability).unwrap();
        self.ability_check_modifier_set(ability, saving_throw_check)
    }

    fn ability_check_modifier_set(&self, ability: Ability, d20_check: &D20Check) -> ModifierSet {
        let mut modifiers = d20_check.modifiers.clone();
        modifiers.add_modifier(
            ModifierSource::Ability(ability),
            self.ability_modifier(ability).total(),
        );
        modifiers.add_modifier(
            ModifierSource::Proficiency(d20_check.proficiency),
            d20_check.proficiency.bonus(self.proficiency_bonus()),
        );
        modifiers
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }
}

#[cfg(test)]
mod tests {
    use crate::combat::damage::DamageComponentResult;

    use super::*;

    #[test]
    fn test_character_creation() {
        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 3);
        class_levels.insert(CharacterClass::Wizard, 2);

        let mut abilities = HashMap::new();
        abilities.insert(Ability::Strength, AbilityScore::new(Ability::Strength, 16));

        let mut skills = HashMap::new();
        skills.insert(Skill::Athletics, D20Check::new(Proficiency::Proficient));

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            skills,
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.name, "Thorin");
        assert_eq!(character.max_hp, 20);
        assert_eq!(character.current_hp, 20);
    }

    #[test]
    fn test_character_total_level() {
        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 3);
        class_levels.insert(CharacterClass::Wizard, 2);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.total_level(), 5);
    }

    #[test]
    fn test_character_proficiency_bonus() {
        let class_levels = HashMap::new();
        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.proficiency_bonus(), 2);
    }

    #[test]
    fn test_character_skill_modifier() {
        let mut abilities = HashMap::new();
        let mut strength = AbilityScore::new(Ability::Strength, 17);
        strength
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Strength".to_string()), 2);
        abilities.insert(Ability::Strength, strength);

        let mut skills = HashMap::new();
        let mut athletics = D20Check::new(Proficiency::Proficient);
        athletics
            .modifiers
            .add_modifier(ModifierSource::Item("Athlete's Belt".to_string()), 1);
        skills.insert(Skill::Athletics, athletics);

        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 5);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            skills,
            HashMap::new(),
            DamageResistances::new(),
        );

        // 17 (base) + 2 (item) = 19
        assert_eq!(character.ability_total(Ability::Strength), 19);
        // Calculate the expected skill modifier
        // 4 (ability) + 1 (item) + 3 (proficiency) = 8
        assert_eq!(character.skill_modifier(Skill::Athletics).total(), 8);
        print!(
            "Athletics Modifier: {} = {:?}",
            character.skill_modifier(Skill::Athletics).total(),
            character.skill_modifier(Skill::Athletics)
        );
    }

    #[test]
    fn test_character_take_damage() {
        let mut resistances = DamageResistances::new();
        resistances.add_effect(
            DamageType::Fire,
            DamageMitigationEffect {
                source: ModifierSource::Item("Boots of Fire Resistance".to_string()),
                op: MitigationOp::Resistance,
            },
        );
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            resistances,
        );

        assert_eq!(character.current_hp, 20);

        let damage_roll_result = DamageRollResult {
            label: "Fireball".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Fire,
                result: DiceGroupRollResult {
                    label: "Fireball".to_string(),
                    rolls: vec![4, 4, 4, 4, 4, 4, 4, 4],
                    die_size: DieSize::D8,
                    modifiers: ModifierSet::new(),
                    subtotal: 32,
                },
            }],
            total: 32,
        };

        character.take_damage(&damage_roll_result);
        // Fire resistance halves the damage
        // 20 - (32 / 2) = 4
        assert_eq!(character.current_hp, 4);
    }

    #[test]
    fn test_character_heal() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        let damage_roll_result = DamageRollResult {
            label: "Arrow".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceGroupRollResult {
                    label: "Arrow".to_string(),
                    rolls: vec![5, 5],
                    die_size: DieSize::D6,
                    modifiers: ModifierSet::new(),
                    subtotal: 10,
                },
            }],
            total: 10,
        };

        character.take_damage(&damage_roll_result);
        character.heal(5);
        assert_eq!(character.current_hp, 15);
    }

    #[test]
    fn test_character_is_alive() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert!(character.is_alive());

        let damage_roll_result = DamageRollResult {
            label: "Power Word Kill".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceGroupRollResult {
                    label: "Power Word Kill".to_string(),
                    rolls: vec![100],
                    die_size: DieSize::D100,
                    modifiers: ModifierSet::new(),
                    subtotal: 100,
                },
            }],
            total: 100,
        };

        character.take_damage(&damage_roll_result);
        assert!(!character.is_alive());
    }

    #[test]
    fn test_character_saving_throw_modifier() {
        let mut abilities = HashMap::new();
        let mut strength = AbilityScore::new(Ability::Strength, 16);
        strength
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Strength".to_string()), 2);
        abilities.insert(Ability::Strength, strength);

        let mut saving_throws = HashMap::new();
        let mut strength_saving_throw = D20Check::new(Proficiency::Proficient);
        strength_saving_throw.modifiers.add_modifier(
            ModifierSource::Item("Strength Saving Throw Item".to_string()),
            3,
        );
        saving_throws.insert(Ability::Strength, strength_saving_throw);

        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 5);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            HashMap::new(),
            saving_throws,
            DamageResistances::new(),
        );

        // 4 (ability) + 3 (item) + 3 (proficiency) = 10
        assert_eq!(
            character.saving_throw_modifier(Ability::Strength).total(),
            10
        );
        println!(
            "Strength Saving Throw Modifier: {} = {:?}",
            character.saving_throw_modifier(Ability::Strength).total(),
            character.saving_throw_modifier(Ability::Strength)
        );
    }
}
