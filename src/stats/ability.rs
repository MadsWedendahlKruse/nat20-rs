use std::{collections::HashMap, fmt, hash::Hash};

use crate::stats::modifier;

use super::modifier::{ModifierSet, ModifierSource};

use strum::{EnumIter, IntoEnumIterator};

#[derive(EnumIter, Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct AbilityScore {
    pub ability: Ability,
    pub base: i32,
    pub modifiers: ModifierSet,
}

impl AbilityScore {
    pub fn default(ability: Ability) -> Self {
        Self {
            ability,
            base: 10,
            modifiers: ModifierSet::new(),
        }
    }

    pub fn new(ability: Ability, base: i32) -> Self {
        Self {
            ability,
            base,
            modifiers: ModifierSet::new(),
        }
    }

    pub fn ability_modifier(&self) -> ModifierSet {
        let mut ability_modifiers = self.modifiers.clone();
        ability_modifiers.scale_modifiers(0.5);
        let base_modifier = (self.base - 10) / 2;
        ability_modifiers.add_modifier(ModifierSource::Ability(self.ability), base_modifier);
        ability_modifiers
    }

    pub fn total(&self) -> i32 {
        self.base + self.modifiers.total()
    }
}

impl fmt::Display for AbilityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifier = self.ability_modifier().total();
        let sign = if modifier >= 0 { "+" } else { "-" };
        write!(
            f,
            "{}: {} ({}{})",
            self.ability,
            self.total(),
            sign,
            modifier.abs()
        )?;
        if self.modifiers.is_empty() {
            return Ok(());
        }
        write!(f, " ({} {})", self.base, self.modifiers)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AbilityScoreSet {
    pub scores: HashMap<Ability, AbilityScore>,
}

impl AbilityScoreSet {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        for ability in Ability::iter() {
            scores.insert(ability, AbilityScore::default(ability));
        }
        Self { scores }
    }

    pub fn get(&self, ability: Ability) -> &AbilityScore {
        self.scores.get(&ability).unwrap()
    }

    fn get_mut(&mut self, ability: Ability) -> &mut AbilityScore {
        self.scores.get_mut(&ability).unwrap()
    }

    pub fn set(&mut self, ability: Ability, score: AbilityScore) {
        self.scores.insert(ability, score);
    }

    pub fn ability_modifier(&self, ability: Ability) -> ModifierSet {
        self.get(ability).ability_modifier()
    }

    pub fn total(&self, ability: Ability) -> i32 {
        self.get(ability).total()
    }

    pub fn add_modifier(&mut self, ability: Ability, source: ModifierSource, value: i32) {
        self.get_mut(ability).modifiers.add_modifier(source, value);
    }

    pub fn remove_modifier(&mut self, ability: Ability, source: &ModifierSource) {
        self.get_mut(ability).modifiers.remove_modifier(source);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn ability_modifier() {
        let ability_score = AbilityScore::new(Ability::Strength, 16);
        let modifier = ability_score.ability_modifier();
        assert_eq!(modifier.total(), 3); // (16 - 10) / 2 = 3
        println!("{:?}", modifier);
    }

    #[test]
    fn ability_total() {
        let mut ability_score = AbilityScore::new(Ability::Dexterity, 14);
        ability_score
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Dexterity".to_string()), 2);
        assert_eq!(ability_score.total(), 16); // 14 + 2 = 16
        assert_eq!(ability_score.ability_modifier().total(), 3); // (16 - 10) / 2 = 3
    }

    #[test]
    fn ability_score_set() {
        let mut ability_scores = AbilityScoreSet::new();
        ability_scores.set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 15),
        );
        ability_scores.add_modifier(
            Ability::Dexterity,
            ModifierSource::Item("Ring of Dexterity".to_string()),
            2,
        );

        assert_eq!(ability_scores.total(Ability::Dexterity), 17);
        assert_eq!(
            ability_scores.ability_modifier(Ability::Dexterity).total(),
            3
        );
    }

    #[test]
    fn ability_score_set_modifier() {
        let mut ability_scores = AbilityScoreSet::new();
        ability_scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        ability_scores.add_modifier(
            Ability::Strength,
            ModifierSource::Item("Ring of Strength".to_string()),
            2,
        );

        assert_eq!(ability_scores.total(Ability::Strength), 19);
        assert_eq!(
            ability_scores.ability_modifier(Ability::Strength).total(),
            4
        );
    }

    #[test]
    fn ability_score_set_multiple_abilities() {
        let mut ability_scores = AbilityScoreSet::new();
        ability_scores.set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 15),
        );
        ability_scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        ability_scores.set(Ability::Charisma, AbilityScore::new(Ability::Charisma, 8));

        assert_eq!(ability_scores.total(Ability::Dexterity), 15);
        assert_eq!(ability_scores.total(Ability::Strength), 17);
        assert_eq!(ability_scores.total(Ability::Charisma), 8);
    }

    #[test]
    fn ability_score_set_remove_modifier() {
        let mut ability_scores = AbilityScoreSet::new();
        ability_scores.set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 15),
        );
        ability_scores.add_modifier(
            Ability::Dexterity,
            ModifierSource::Item("Ring of Dexterity".to_string()),
            2,
        );

        assert_eq!(ability_scores.total(Ability::Dexterity), 17);
        ability_scores.remove_modifier(
            Ability::Dexterity,
            &ModifierSource::Item("Ring of Dexterity".to_string()),
        );
        assert_eq!(ability_scores.total(Ability::Dexterity), 15);
    }
}
