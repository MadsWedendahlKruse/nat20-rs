use std::collections::HashMap;

use super::{ability::Ability, proficiency::Proficiency};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum ModifierSource {
    Spell(String),        // e.g. "Bless"
    Item(String),         // e.g. "Belt of Strength"
    Condition(String),    // e.g. "Poisoned"
    ClassFeature(String), // e.g. "Rage"
    EffectId(u32),        // optional: unique ID for internal tracking
    Custom(String),       // fallback for ad-hoc things
    Ability(Ability),     // e.g. "Strength"
    Proficiency(Proficiency),
}

#[derive(Debug, Clone)]
pub struct ModifierSet {
    pub modifiers: HashMap<ModifierSource, i32>,
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

    // Only used for ability modifiers
    pub fn scale_modifiers(&mut self, scale: f32) {
        for m in self.modifiers.values_mut() {
            *m = (m.clone() as f32 * scale).round() as i32;
        }
    }

    pub fn total(&self) -> i32 {
        self.modifiers.values().map(|m| m).sum()
    }

    pub fn breakdown(&self) -> String {
        let mut s = String::new();
        for (source, value) in &self.modifiers {
            let sign = if *value >= 0 { "+" } else { "" };
            s += &format!(", {:?}: {}{}", source, sign, value);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Item("Belt of Strength".to_string()), 4);
        modifiers.add_modifier(ModifierSource::Spell("Bless".to_string()), 1);

        assert_eq!(modifiers.modifiers.len(), 2);
        assert_eq!(modifiers.total(), 5);

        modifiers.remove_modifier(&ModifierSource::Spell("Bless".to_string()));
        assert_eq!(modifiers.modifiers.len(), 1);
        assert_eq!(modifiers.total(), 4);
        println!("Modifiers breakdown: {}", modifiers.breakdown());
    }
}
