use crate::stats::modifier::{ModifierSet, ModifierSource};
use crate::stats::proficiency::Proficiency;

use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RollMode {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone, Copy)]
pub enum AdvantageType {
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone)]
pub struct AdvantageSource {
    pub kind: AdvantageType,
    pub source: ModifierSource,
}

#[derive(Debug, Default, Clone)]
pub struct AdvantageTracker {
    sources: Vec<AdvantageSource>,
}

impl AdvantageTracker {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add(&mut self, kind: AdvantageType, source: ModifierSource) {
        self.sources.push(AdvantageSource { kind, source });
    }

    pub fn remove(&mut self, source: &ModifierSource) {
        self.sources.retain(|s| &s.source != source);
    }

    pub fn roll_mode(&self) -> RollMode {
        match self.sources.iter().fold(0, |acc, s| {
            acc + match s.kind {
                AdvantageType::Advantage => 1,
                AdvantageType::Disadvantage => -1,
            }
        }) {
            n if n > 0 => RollMode::Advantage,
            n if n < 0 => RollMode::Disadvantage,
            _ => RollMode::Normal,
        }
    }

    pub fn summary(&self) -> Vec<(&ModifierSource, AdvantageType)> {
        self.sources.iter().map(|s| (&s.source, s.kind)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct D20Check {
    pub modifiers: ModifierSet,
    pub proficiency: Proficiency,
    pub advantage_tracker: AdvantageTracker,
}

impl D20Check {
    pub fn new(proficiency: Proficiency) -> Self {
        Self {
            modifiers: ModifierSet::new(),
            proficiency,
            advantage_tracker: AdvantageTracker::new(),
        }
    }

    pub fn advantage_tracker(&self) -> &AdvantageTracker {
        &self.advantage_tracker
    }

    pub fn advantage_tracker_mut(&mut self) -> &mut AdvantageTracker {
        &mut self.advantage_tracker
    }

    pub fn perform(&self) -> D20CheckResult {
        let mut rng = rand::rng();
        // Technically inefficient to always roll two dice, but it's probably not a big deal
        let roll1 = rng.random_range(1..=20);
        let roll2 = rng.random_range(1..=20);

        let roll_mode = self.advantage_tracker.roll_mode();
        let rolls = match roll_mode {
            RollMode::Normal => vec![roll1],
            _ => vec![roll1 as u32, roll2 as u32],
        };
        let selected_roll = match roll_mode {
            RollMode::Normal => roll1,
            RollMode::Advantage => roll1.max(roll2),
            RollMode::Disadvantage => roll1.min(roll2),
        } as u32;

        let total_modifier = self.modifiers.total();
        let total = selected_roll + total_modifier.max(0) as u32;

        D20CheckResult {
            roll_mode,
            rolls,
            selected_roll,
            modifier_breakdown: self.modifiers.clone(),
            total_modifier,
            total,
            is_crit: selected_roll == 20,
            is_crit_fail: selected_roll == 1,
        }
    }
}

#[derive(Debug)]
pub struct D20CheckResult {
    pub roll_mode: RollMode,
    pub rolls: Vec<u32>,
    pub selected_roll: u32,
    pub modifier_breakdown: ModifierSet,
    pub total_modifier: i32,
    pub total: u32,
    pub is_crit: bool,
    pub is_crit_fail: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::modifier::ModifierSource;

    #[test]
    fn test_d20_check() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        check
            .modifiers
            .add_modifier(ModifierSource::Proficiency(Proficiency::Proficient), 2);
        let result = check.perform();

        // 1d20 + 2 + 2
        // Min: 1 + 2 + 2 = 5
        // Max: 20 + 2 + 2 = 24
        assert!(result.total >= 5 && result.total <= 24);
        assert_eq!(result.rolls.len(), 1);
        assert_eq!(result.roll_mode, RollMode::Normal);
        println!("{:?}", result);
    }

    #[test]
    fn test_d20_check_with_advantage() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        check.advantage_tracker.add(
            AdvantageType::Advantage,
            ModifierSource::Item("Lucky Charm".to_string()),
        );
        let result = check.perform();

        // 1d20 + 2
        // Min: 1 + 2 = 3
        // Max: 20 + 2 = 22
        assert!(result.total >= 3 && result.total <= 22);
        assert_eq!(result.rolls.len(), 2);
        assert_eq!(result.roll_mode, RollMode::Advantage);
        // Check if the selected roll is the maximum
        assert_eq!(
            result.selected_roll,
            result.rolls.iter().max().unwrap().clone()
        );
        println!("{:?}", result);
    }

    #[test]
    fn test_d20_check_with_disadvantage() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check.advantage_tracker.add(
            AdvantageType::Disadvantage,
            ModifierSource::Item("Cursed Ring".to_string()),
        );
        let result = check.perform();

        // 1d20
        // Min: 1 = 1
        // Max: 20 = 20
        assert!(result.total >= 1 && result.total <= 20);
        assert_eq!(result.rolls.len(), 2);
        assert_eq!(result.roll_mode, RollMode::Disadvantage);
        // Check if the selected roll is the minimum
        assert_eq!(
            result.selected_roll,
            result.rolls.iter().min().unwrap().clone()
        );
        println!("{:?}", result);
    }
}
