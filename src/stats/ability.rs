use super::modifier::{ModifierSet, ModifierSource};

use strum::EnumIter;

#[derive(EnumIter, Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_modifier() {
        let ability_score = AbilityScore::new(Ability::Strength, 16);
        let modifier = ability_score.ability_modifier();
        assert_eq!(modifier.total(), 3); // (16 - 10) / 2 = 3
        println!("{:?}", modifier);
    }

    #[test]
    fn test_ability_total() {
        let mut ability_score = AbilityScore::new(Ability::Dexterity, 14);
        ability_score
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Dexterity".to_string()), 2);
        assert_eq!(ability_score.total(), 16); // 14 + 2 = 16
        assert_eq!(ability_score.ability_modifier().total(), 3); // (16 - 10) / 2 = 3
    }
}
