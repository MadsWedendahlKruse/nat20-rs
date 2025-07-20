use std::collections::HashMap;
use std::fmt;

use crate::components::id::SpellId;

use super::{ability::Ability, proficiency::Proficiency};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum ModifierSource {
    Spell(SpellId),       // e.g. "Bless"
    Item(String),         // e.g. "Belt of Strength"
    Condition(String),    // e.g. "Poisoned"
    ClassFeature(String), // e.g. "Rage"
    EffectId(u32),        // optional: unique ID for internal tracking
    Custom(String),       // fallback for ad-hoc things
    Ability(Ability),     // e.g. "Strength"
    Proficiency(Proficiency),
}

impl fmt::Display for ModifierSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModifierSource::Spell(id) => write!(f, "Spell: {}", id),
            ModifierSource::Item(name) => write!(f, "Item: {}", name),
            ModifierSource::Condition(name) => write!(f, "Condition: {}", name),
            ModifierSource::ClassFeature(name) => write!(f, "Class Feature: {}", name),
            ModifierSource::EffectId(id) => write!(f, "Effect ID: {}", id),
            ModifierSource::Custom(name) => write!(f, "Custom: {}", name),
            ModifierSource::Ability(ability) => write!(f, "{:?} Modifier", ability),
            ModifierSource::Proficiency(proficiency) => write!(f, "Proficiency: {:?}", proficiency),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierSet {
    modifiers: HashMap<ModifierSource, i32>,
}

impl ModifierSet {
    pub fn new() -> Self {
        Self {
            modifiers: HashMap::new(),
        }
    }

    pub fn add_modifier(&mut self, source: ModifierSource, value: i32) {
        self.modifiers.insert(source.clone(), value);
    }

    pub fn remove_modifier(&mut self, source: &ModifierSource) {
        self.modifiers.remove(source);
    }

    pub fn add_modifier_set(&mut self, other: &ModifierSet) {
        for (source, value) in &other.modifiers {
            let entry = self.modifiers.entry(source.clone()).or_insert(0);
            *entry += value;
        }
    }

    pub fn get(&self, source: &ModifierSource) -> Option<i32> {
        self.modifiers.get(source).cloned()
    }

    // Only used for ability modifiers
    pub fn scale_modifiers(&mut self, scale: f32) {
        for m in self.modifiers.values_mut() {
            *m = (m.clone() as f32 * scale).round() as i32;
        }
    }

    pub fn total(&self) -> i32 {
        self.modifiers.values().map(|m| m).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }
}

impl fmt::Display for ModifierSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for i in 0..self.modifiers.len() {
            let (source, value) = self.modifiers.iter().nth(i).unwrap();
            if value == &0 {
                continue;
            }
            if i != 0 {
                s += " ";
            }
            let sign = if *value >= 0 { "+" } else { "-" };
            s += &format!("{} {} ({})", sign, value.abs(), source);
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Item("Belt of Strength".to_string()), 4);
        modifiers.add_modifier(ModifierSource::Spell(SpellId::from_str("BLESS")), 1);

        assert_eq!(modifiers.modifiers.len(), 2);
        assert_eq!(modifiers.total(), 5);

        modifiers.remove_modifier(&ModifierSource::Spell(SpellId::from_str("BLESS")));
        assert_eq!(modifiers.modifiers.len(), 1);
        assert_eq!(modifiers.total(), 4);
        println!("Modifiers breakdown: {}", modifiers);
    }
}
