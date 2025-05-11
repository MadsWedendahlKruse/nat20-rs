use crate::dice::dice::*;
use crate::stats::modifier::ModifierSource;

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

impl fmt::Display for DamageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// --- DAMAGE APPLICATION ---

#[derive(Debug)]
pub struct DamageComponent {
    pub dice: DiceGroup,
    pub damage_type: DamageType,
}

#[derive(Debug)]
pub struct DamageRoll {
    pub components: Vec<DamageComponent>,
    pub label: String,
}

impl DamageRoll {
    fn roll(&self) -> DamageRollResult {
        let mut components = Vec::new();
        let mut total = 0;

        for component in &self.components {
            let result = component.dice.roll();
            total += result.subtotal;
            components.push(DamageComponentResult {
                damage_type: component.damage_type,
                result,
            });
        }

        DamageRollResult {
            label: self.label.clone(),
            components,
            total,
        }
    }
}

#[derive(Debug)]
pub struct DamageComponentResult {
    pub damage_type: DamageType,
    pub result: DiceGroupRollResult,
}

#[derive(Debug)]
pub struct DamageRollResult {
    pub label: String,
    pub components: Vec<DamageComponentResult>,
    pub total: i32,
}

impl DamageRollResult {
    fn display(&self) {
        println!("== {} ==", self.label);
        for comp in &self.components {
            println!(
                "{}: {} ({}d{}) + {:?} = {} {}",
                comp.result.label,
                comp.result.rolls.iter().sum::<u32>(),
                comp.result.rolls.len(),
                comp.result.die_size as u32,
                comp.result.modifiers,
                comp.result.subtotal,
                comp.damage_type.to_string()
            );
        }
        println!("Total Damage: {}", self.total);
    }
}

/// --- DAMAGE MITIGATION ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MitigationOp {
    Resistance,         // divide by 2
    Vulnerability,      // multiply by 2
    Immunity,           // set to 0
    FlatReduction(i32), // subtract N
}

impl fmt::Display for MitigationOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MitigationOp::Resistance => write!(f, "/ 2"),
            MitigationOp::Vulnerability => write!(f, "* 2"),
            MitigationOp::Immunity => write!(f, "* 0"),
            MitigationOp::FlatReduction(amount) => write!(f, "-{}", amount),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamageMitigationEffect {
    pub source: ModifierSource,
    pub op: MitigationOp,
}

#[derive(Debug)]
pub struct DamageResistances {
    pub effects: HashMap<DamageType, Vec<DamageMitigationEffect>>,
}

impl DamageResistances {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }

    pub fn add_effect(&mut self, dtype: DamageType, effect: DamageMitigationEffect) {
        self.effects
            .entry(dtype)
            .or_insert_with(Vec::new)
            .push(effect);
    }

    pub fn remove_effect(&mut self, dtype: DamageType, effect: &DamageMitigationEffect) {
        if let Some(effects) = self.effects.get_mut(&dtype) {
            effects.retain(|e| e != effect);
            if effects.is_empty() {
                self.effects.remove(&dtype);
            }
        }
    }

    pub fn apply(&self, roll: &DamageRollResult) -> DamageMitigationResult {
        let mut components = Vec::new();
        let mut total = 0;

        for comp in &roll.components {
            let dtype = comp.damage_type;
            let mut value = comp.result.subtotal;
            let mut applied_mods = Vec::new();

            if let Some(effects) = self.effects.get(&dtype) {
                for effect in effects {
                    match effect.op {
                        MitigationOp::Resistance => {
                            value /= 2;
                        }
                        MitigationOp::Vulnerability => {
                            value *= 2;
                        }
                        MitigationOp::Immunity => {
                            value = 0;
                        }
                        MitigationOp::FlatReduction(amount) => {
                            value = (value - amount).max(0);
                        }
                    }
                    applied_mods.push(effect.clone());
                }
            }

            total += value;
            components.push(DamageComponentMitigation {
                damage_type: dtype,
                original: comp.result.subtotal,
                after_mods: value,
                modifiers: applied_mods,
            });
        }

        DamageMitigationResult { components, total }
    }
}

#[derive(Debug)]
pub struct DamageComponentMitigation {
    pub damage_type: DamageType,
    pub original: i32,
    pub after_mods: i32,
    pub modifiers: Vec<DamageMitigationEffect>,
}

impl fmt::Display for DamageComponentMitigation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.original == self.after_mods {
            return write!(f, "{}", self.original);
        }
        write!(f, "{}", self.after_mods)?;
        for modif in &self.modifiers {
            write!(f, " ({} {}) {:?}", self.original, modif.op, modif.source)?;
        }
        return Ok(());
    }
}

#[derive(Debug)]
pub struct DamageMitigationResult {
    pub components: Vec<DamageComponentMitigation>,
    pub total: i32,
}

impl fmt::Display for DamageMitigationResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Damage Mitigation Result:\n")?;
        for comp in &self.components {
            write!(f, "{}", comp)?;
        }
        write!(f, "Total Damage Mitigation: {}", self.total)
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::{ability::Ability, modifier::ModifierSet, modifier::ModifierSource};

    use super::*;

    #[test]
    fn test_damage_roll() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Ability(Ability::Strength), 2);
        let damage_roll = DamageRoll {
            components: vec![
                DamageComponent {
                    dice: DiceGroup::new(2, DieSize::D6, modifiers, "Base damage".to_string()),
                    damage_type: DamageType::Slashing,
                },
                DamageComponent {
                    dice: DiceGroup::new(1, DieSize::D4, ModifierSet::new(), "Enchant".to_string()),
                    damage_type: DamageType::Fire,
                },
            ],
            label: "Everburn Blade".to_string(),
        };

        let result = damage_roll.roll();
        assert_eq!(result.components.len(), 2);
        // 2d6 + 1d4 + 2 (str mod)
        // Min roll: 2 + 1 + 2 = 5
        // Max roll: 12 + 4 + 2 = 18
        assert!(result.total >= 5 && result.total <= 18);
        result.display();
    }

    #[test]
    fn test_damage_mitigation() {
        let roll_result = DamageRollResult {
            label: "Everburn Blade".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceGroupRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceGroupRollResult {
                        label: "Enchant".to_string(),
                        rolls: vec![2],
                        die_size: DieSize::D4,
                        modifiers: ModifierSet::new(),
                        subtotal: 2,
                    },
                },
            ],
            total: 9,
        };

        let mut resistances = DamageResistances {
            effects: HashMap::new(),
        };
        resistances.effects.insert(
            DamageType::Slashing,
            vec![DamageMitigationEffect {
                source: ModifierSource::Item("Shield of Resistance".to_string()),
                op: MitigationOp::Resistance,
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // 7 / 2 + 2 = 3.5 + 2 = 5.5 -> round down to 5
        assert_eq!(mitigation_result.total, 5);
        println!("{:?}", mitigation_result);
    }
}
