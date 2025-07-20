use std::fmt;

use rand::Rng;

use crate::components::modifier::ModifierSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DieSize {
    D4 = 4,
    D6 = 6,
    D8 = 8,
    D10 = 10,
    D12 = 12,
    D20 = 20,
    D100 = 100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiceSet {
    pub num_dice: u32,
    pub die_size: DieSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceSetRoll {
    pub dice: DiceSet,
    pub modifiers: ModifierSet,
    pub label: String,
}

impl DiceSetRoll {
    // TODO: Redundant new?
    pub fn new(dice_set: DiceSet, modifier: ModifierSet, label: String) -> Self {
        Self {
            dice: dice_set,
            modifiers: modifier,
            label,
        }
    }

    pub fn roll(&self) -> DiceSetRollResult {
        let mut rng = rand::rng();
        let rolls: Vec<u32> = (0..self.dice.num_dice)
            .map(|_| rng.random_range(1..=self.dice.die_size as u32))
            .collect();
        let subtotal = rolls.iter().sum::<u32>() as i32 + self.modifiers.total();

        DiceSetRollResult {
            label: self.label.clone(),
            die_size: self.dice.die_size,
            rolls,
            modifiers: self.modifiers.clone(),
            subtotal,
        }
    }

    pub fn min_roll(&self) -> i32 {
        (self.dice.num_dice as i32) + self.modifiers.total()
    }

    pub fn max_roll(&self) -> i32 {
        (self.dice.num_dice as i32 * self.dice.die_size as i32) + self.modifiers.total()
    }
}

impl fmt::Display for DiceSetRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            return write!(f, "{}d{}", self.dice.num_dice, self.dice.die_size as u32);
        }
        write!(
            f,
            "{}d{} {}",
            self.dice.num_dice, self.dice.die_size as u32, self.modifiers
        )
    }
}

#[derive(Debug, Clone)]
pub struct DiceSetRollResult {
    pub label: String,
    pub die_size: DieSize,
    pub rolls: Vec<u32>,
    pub modifiers: ModifierSet,
    pub subtotal: i32,
}

impl fmt::Display for DiceSetRollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}d{})",
            self.rolls.iter().sum::<u32>(),
            self.rolls.len(),
            self.die_size as u32,
        )?;
        if self.modifiers.is_empty() {
            write!(f, " = {}", self.subtotal)
        } else {
            write!(f, " {} = {}", self.modifiers, self.subtotal)
        }
    }
}

#[derive(Debug)]
pub struct CompositeRoll {
    pub groups: Vec<DiceSetRoll>,
    pub label: String, // optional general label for the whole roll
}

impl CompositeRoll {
    pub fn roll(&self) -> CompositeRollResult {
        let mut total = 0;
        let mut components = Vec::new();

        for group in &self.groups {
            let result = group.roll();
            total += result.subtotal;
            components.push(result);
        }

        CompositeRollResult {
            label: self.label.clone(),
            components,
            total,
        }
    }

    pub fn min_roll(&self) -> i32 {
        self.groups.iter().map(|g| g.min_roll()).sum()
    }

    pub fn max_roll(&self) -> i32 {
        self.groups.iter().map(|g| g.max_roll()).sum()
    }
}

#[derive(Debug)]
pub struct CompositeRollResult {
    pub label: String,
    pub components: Vec<DiceSetRollResult>,
    pub total: i32,
}

impl fmt::Display for CompositeRollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.label)?;
        for comp in &self.components {
            write!(f, "{} ", comp)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{ability::Ability, modifier::ModifierSource};

    use super::*;

    #[test]
    fn dice_roll() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Ability(Ability::Charisma), 3);
        let dice = DiceSetRoll {
            dice: DiceSet {
                num_dice: 2,
                die_size: DieSize::D6,
            },
            modifiers,
            label: "Test Dice".to_string(),
        };
        println!("Rolling:\n{}", dice);
        let result = dice.roll();

        let expected_min = dice.min_roll();
        let expected_max = dice.max_roll();
        assert_eq!(5, expected_min);
        assert_eq!(15, expected_max);

        assert_eq!(result.rolls.len(), 2);
        for roll in &result.rolls {
            assert!(*roll >= 1 && *roll <= 6, "Roll out of bounds: {}", roll);
        }
        assert!(result.subtotal >= 5 && result.subtotal <= 15);
        println!("Dice Roll Result:\n{}", result);
    }

    #[test]
    fn composite_roll() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        let group1 = DiceSetRoll {
            dice: DiceSet {
                num_dice: 2,
                die_size: DieSize::D6,
            },
            modifiers: modifiers,
            label: "Group 1".to_string(),
        };
        let group2 = DiceSetRoll {
            dice: DiceSet {
                num_dice: 3,
                die_size: DieSize::D4,
            },
            modifiers: ModifierSet::new(),
            label: "Group 2".to_string(),
        };
        let composite = CompositeRoll {
            groups: vec![group1, group2],
            label: "Composite Roll".to_string(),
        };
        let result = composite.roll();
        assert_eq!(result.components.len(), 2);

        // Min rolls for group1: 2 (1+1) + 2 = 4
        // Max rolls for group1: 12 (6+6) + 2 = 14
        // Min rolls for group2: 3 (1+1+1) + 0 = 3
        // Max rolls for group2: 12 (4+4+4) + 0 = 12
        // Total min: 4 + 3 = 7
        // Total max: 14 + 12 = 26
        let min_roll = composite.min_roll();
        let max_roll = composite.max_roll();
        assert_eq!(min_roll, 7);
        assert_eq!(max_roll, 26);

        assert!(result.total >= 10 && result.total <= 31);
        println!("{}", result);
    }
}
