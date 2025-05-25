use crate::creature::character::Character;
use crate::stats::modifier::{ModifierSet, ModifierSource};
use crate::stats::proficiency::Proficiency;

use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use strum::IntoEnumIterator;

use super::ability::Ability;

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
    modifiers: ModifierSet,
    proficiency: Proficiency,
    advantage_tracker: AdvantageTracker,
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

    pub fn modifiers(&self) -> &ModifierSet {
        &self.modifiers
    }

    pub fn modifiers_mut(&mut self) -> &mut ModifierSet {
        &mut self.modifiers
    }

    pub fn add_modifier(&mut self, source: ModifierSource, value: i32) {
        self.modifiers.add_modifier(source, value);
    }

    pub fn remove_modifier(&mut self, source: &ModifierSource) {
        self.modifiers.remove_modifier(source);
    }

    pub fn proficiency(&self) -> &Proficiency {
        &self.proficiency
    }

    pub fn set_proficiency(&mut self, proficiency: Proficiency) {
        self.proficiency = proficiency;
    }

    pub fn perform(&mut self, proficiency_bonus: i32) -> D20CheckResult {
        self.add_modifier(
            ModifierSource::Proficiency(self.proficiency),
            self.proficiency.bonus(proficiency_bonus),
        );

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
            advantage_tracker: self.advantage_tracker.clone(),
            rolls,
            selected_roll,
            modifier_breakdown: self.modifiers.clone(),
            total_modifier,
            total,
            is_crit: selected_roll == 20,
            is_crit_fail: selected_roll == 1,
            success: None, // Success is determined later based on DC or other conditions
        }
    }
}

impl fmt::Display for D20Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "1d20")?;
        if self.proficiency != Proficiency::None {
            write!(f, " + {}", self.proficiency)?;
        }
        if self.modifiers.is_empty() {
            return Ok(());
        }
        write!(f, " {}", self.modifiers)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct D20CheckResult {
    pub advantage_tracker: AdvantageTracker,
    pub rolls: Vec<u32>,
    pub selected_roll: u32,
    pub modifier_breakdown: ModifierSet,
    pub total_modifier: i32,
    pub total: u32,
    pub is_crit: bool,
    pub is_crit_fail: bool,
    pub success: Option<bool>,
}

impl fmt::Display for D20CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (1d20)", self.selected_roll)?;
        if self.advantage_tracker.roll_mode() != RollMode::Normal {
            write!(
                f,
                " ({}, {}, {:?})",
                self.rolls[0],
                self.rolls[1],
                self.advantage_tracker.roll_mode()
            )?;
        }
        if self.is_crit {
            write!(f, " (Critical Success!)")?;
        }
        if self.is_crit_fail {
            write!(f, " (Critical Failure!)")?;
        }
        if !self.modifier_breakdown.is_empty() {
            write!(f, " {}", self.modifier_breakdown)?;
        }
        write!(f, " = {}", self.total)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct D20CheckSet<K, H>
where
    K: Eq + Hash + IntoEnumIterator + Copy,
{
    checks: HashMap<K, D20Check>,
    get_hooks: fn(K, &Character) -> Vec<&H>,
    apply_check_hook: fn(&H, &Character, &mut D20Check),
    apply_result_hook: fn(&H, &Character, &mut D20CheckResult),
    ability_mapper: fn(K) -> Ability,
}

impl<K, T> D20CheckSet<K, T>
where
    K: Eq + Hash + IntoEnumIterator + Copy,
{
    pub fn new(
        get_hooks: fn(K, &Character) -> Vec<&T>,
        apply_check_hook: fn(&T, &Character, &mut D20Check),
        apply_result_hook: fn(&T, &Character, &mut D20CheckResult),
        ability_mapper: fn(K) -> Ability,
    ) -> Self {
        let checks = K::iter()
            .map(|k| (k, D20Check::new(Proficiency::None)))
            .collect();
        Self {
            checks,
            get_hooks,
            apply_check_hook,
            apply_result_hook,
            ability_mapper,
        }
    }

    fn get(&self, key: K) -> &D20Check {
        self.checks.get(&key).unwrap()
    }

    fn get_mut(&mut self, key: K) -> &mut D20Check {
        self.checks.get_mut(&key).unwrap()
    }

    pub fn set_proficiency(&mut self, key: K, prof: Proficiency) {
        self.get_mut(key).set_proficiency(prof);
    }

    pub fn add_modifier(&mut self, key: K, source: ModifierSource, value: i32) {
        self.get_mut(key).add_modifier(source, value);
    }

    pub fn remove_modifier(&mut self, key: K, source: &ModifierSource) {
        self.get_mut(key).remove_modifier(source);
    }

    pub fn check(&self, key: K, character: &Character) -> D20CheckResult {
        let mut d20 = self.get(key).clone();
        let ability = (self.ability_mapper)(key);
        let ability_scores = character.ability_scores();
        d20.add_modifier(
            ModifierSource::Ability(ability),
            ability_scores.ability_modifier(ability).total(),
        );

        execute_d20_check(
            d20,
            character,
            &(self.get_hooks)(key, character),
            |hook, character, check| (self.apply_check_hook)(*hook, character, check),
            |hook, character, result| (self.apply_result_hook)(*hook, character, result),
        )
    }

    pub fn check_dc(&self, dc: &D20CheckDC<K>, character: &Character) -> D20CheckResult {
        let mut result = self.check(dc.key, character);
        result.success = Some(result.total >= dc.dc);

        result
    }
}

pub fn execute_d20_check<T>(
    mut check: D20Check,
    character: &Character,
    hooks: &[T],
    pre: impl Fn(&T, &Character, &mut D20Check),
    post: impl Fn(&T, &Character, &mut D20CheckResult),
) -> D20CheckResult {
    for hook in hooks {
        pre(hook, character, &mut check);
    }

    let mut result = check.perform(character.proficiency_bonus());

    for hook in hooks {
        post(hook, character, &mut result);
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct D20CheckDC<T>
where
    T: IntoEnumIterator + Copy + Eq + Hash,
{
    pub key: T,
    pub dc: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::modifier::ModifierSource;

    #[test]
    fn d20_check() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        println!("Check: {}", check);
        let result = check.perform(2);

        // 1d20 + 2 + 2
        // Min: 1 + 2 + 2 = 5
        // Max: 20 + 2 + 2 = 24
        assert!(result.total >= 5 && result.total <= 24);
        assert_eq!(result.rolls.len(), 1);
        assert_eq!(result.advantage_tracker.roll_mode(), RollMode::Normal);
        println!("Result: {}", result);
    }

    #[test]
    fn d20_check_with_advantage() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        check.advantage_tracker.add(
            AdvantageType::Advantage,
            ModifierSource::Item("Lucky Charm".to_string()),
        );
        let result = check.perform(0);

        // 1d20 + 2
        // Min: 1 + 2 = 3
        // Max: 20 + 2 = 22
        assert!(result.total >= 3 && result.total <= 22);
        assert_eq!(result.rolls.len(), 2);
        assert_eq!(result.advantage_tracker.roll_mode(), RollMode::Advantage);
        // Check if the selected roll is the maximum
        assert_eq!(
            result.selected_roll,
            result.rolls.iter().max().unwrap().clone()
        );
        println!("Result: {}", result);
    }

    #[test]
    fn d20_check_with_disadvantage() {
        let mut check = D20Check::new(Proficiency::Expertise);
        check.advantage_tracker.add(
            AdvantageType::Disadvantage,
            ModifierSource::Item("Cursed Ring".to_string()),
        );
        let result = check.perform(4);

        // 1d20
        // Min: 1 + 8 = 9
        // Max: 20 + 8 = 28
        assert!(result.total >= 9 && result.total <= 28);
        assert_eq!(result.rolls.len(), 2);
        assert_eq!(result.advantage_tracker.roll_mode(), RollMode::Disadvantage);
        // Check if the selected roll is the minimum
        assert_eq!(
            result.selected_roll,
            result.rolls.iter().min().unwrap().clone()
        );
        println!("Result: {}", result);
    }

    #[test]
    fn d20_check_with_advantage_and_disadvantage() {
        let mut check = D20Check::new(Proficiency::Expertise);
        check.advantage_tracker.add(
            AdvantageType::Advantage,
            ModifierSource::Item("Lucky Charm".to_string()),
        );
        check.advantage_tracker.add(
            AdvantageType::Disadvantage,
            ModifierSource::Item("Cursed Ring".to_string()),
        );
        let result = check.perform(4);

        // 1d20
        // Min: 1 + 8 = 9
        // Max: 20 + 8 = 28
        assert!(result.total >= 9 && result.total <= 28);
        assert_eq!(result.rolls.len(), 1);
        assert_eq!(result.advantage_tracker.roll_mode(), RollMode::Normal);
        println!("Result: {}", result);
    }

    #[test]
    fn d20_check_critical_success() {
        let mut check = D20Check::new(Proficiency::Proficient);
        check
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Rolling".to_string()), 2);
        let mut result = check.perform(0);
        while result.selected_roll != 20 {
            // Simulate rolling again until we get a critical success
            result = check.perform(0);
        }

        // Simulate a critical success by setting the selected roll to 20
        assert!(result.is_crit);
        println!("Result: {}", result);
    }
}
