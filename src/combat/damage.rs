use crate::dice::dice::*;
use crate::stats::modifier::{ModifierSet, ModifierSource};

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

#[derive(Debug, Clone, PartialEq)]
pub struct DamageComponent {
    pub dice_roll: DiceSetRoll,
    pub damage_type: DamageType,
}

impl DamageComponent {
    pub fn new(num_dice: u32, die_size: DieSize, damage_type: DamageType, label: String) -> Self {
        Self {
            dice_roll: DiceSetRoll::new(DiceSet { num_dice, die_size }, ModifierSet::new(), label),
            damage_type,
        }
    }
}

#[derive(Debug)]
pub struct DamageComponentResult {
    pub result: DiceSetRollResult,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageRoll {
    /// Separate the primary so we know where to apply e.g. ability modifiers
    pub primary: DamageComponent,
    pub bonus: Vec<DamageComponent>,
    pub label: String,
}

impl DamageRoll {
    // TODO: There's too many labels everywhere
    pub fn new(num_dice: u32, die_size: DieSize, damage_type: DamageType, label: String) -> Self {
        Self {
            label: label.clone(),
            primary: DamageComponent::new(num_dice, die_size, damage_type, label),
            bonus: Vec::new(),
        }
    }

    pub fn roll(&self) -> DamageRollResult {
        let mut components = Vec::new();
        let mut total = 0;

        let primary_result = self.primary.dice_roll.roll();
        total += primary_result.subtotal;
        components.push(DamageComponentResult {
            damage_type: self.primary.damage_type,
            result: primary_result,
        });

        for component in &self.bonus {
            let result = component.dice_roll.roll();
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
pub enum MitigationOperation {
    Resistance,         // divide by 2
    Vulnerability,      // multiply by 2
    Immunity,           // set to 0
    FlatReduction(i32), // subtract N
}

impl MitigationOperation {
    fn apply(&self, value: i32) -> i32 {
        match self {
            MitigationOperation::Resistance => value / 2,
            MitigationOperation::Vulnerability => value * 2,
            MitigationOperation::Immunity => 0,
            MitigationOperation::FlatReduction(amount) => (value - amount).max(0),
        }
    }

    fn priority(&self) -> u8 {
        match self {
            MitigationOperation::Immunity => 0,
            MitigationOperation::FlatReduction(_) => 1,
            MitigationOperation::Resistance => 2,
            MitigationOperation::Vulnerability => 3,
        }
    }
}

impl fmt::Display for MitigationOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MitigationOperation::Resistance => write!(f, "/ 2"),
            MitigationOperation::Vulnerability => write!(f, "* 2"),
            MitigationOperation::Immunity => write!(f, "* 0"),
            MitigationOperation::FlatReduction(amount) => write!(f, "-{}", amount),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamageMitigationEffect {
    pub source: ModifierSource,
    pub operation: MitigationOperation,
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
                // Sort by priority
                let mut sorted_effects = effects.clone();
                sorted_effects.sort_by_key(|e| e.operation.priority());

                for effect in sorted_effects {
                    value = effect.operation.apply(value);
                    applied_mods.push(effect);
                    if value <= 0 {
                        break;
                    }
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
            write!(
                f,
                " ({} {}) {:?}",
                self.original, modif.operation, modif.source
            )?;
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
    fn damage_roll() {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Ability(Ability::Strength), 2);
        let damage_roll = DamageRoll {
            label: "Sword of Flame".to_string(),
            primary: DamageComponent {
                dice_roll: DiceSetRoll::new(
                    DiceSet {
                        num_dice: 2,
                        die_size: DieSize::D6,
                    },
                    modifiers,
                    "Base damage".to_string(),
                ),
                damage_type: DamageType::Slashing,
            },
            bonus: vec![DamageComponent {
                dice_roll: DiceSetRoll::new(
                    DiceSet {
                        num_dice: 1,
                        die_size: DieSize::D4,
                    },
                    ModifierSet::new(),
                    "Fire Enchant".to_string(),
                ),
                damage_type: DamageType::Fire,
            }],
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
    fn damage_mitigation_resistance() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
                operation: MitigationOperation::Resistance,
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // 7 / 2 + 2 = 3.5 + 2 = 5.5 -> round down to 5
        assert_eq!(mitigation_result.total, 5);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_immunity() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
            DamageType::Fire,
            vec![DamageMitigationEffect {
                source: ModifierSource::Item("Ring of Fire Immunity".to_string()),
                operation: MitigationOperation::Immunity,
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // 7 + 0 = 7
        assert_eq!(mitigation_result.total, 7);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_vulnerability() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
                source: ModifierSource::Item("Shield of Vulnerability".to_string()),
                operation: MitigationOperation::Vulnerability,
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // 7 * 2 + 2 = 16
        assert_eq!(mitigation_result.total, 16);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_flat_reduction() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
                source: ModifierSource::Item("Shield of Flat Reduction".to_string()),
                operation: MitigationOperation::FlatReduction(3),
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // 7 - 3 + 2 = 6
        assert_eq!(mitigation_result.total, 6);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_multiple_effects() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
            vec![
                DamageMitigationEffect {
                    source: ModifierSource::Item("Shield of Resistance".to_string()),
                    operation: MitigationOperation::Resistance,
                },
                DamageMitigationEffect {
                    source: ModifierSource::Item("Shield of Flat Reduction".to_string()),
                    operation: MitigationOperation::FlatReduction(3),
                },
            ],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // Slashing: 7 - 3 / 2 = 2
        // 2 Slashing + 2 Fire = 4
        assert_eq!(mitigation_result.total, 4);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_multiple_types() {
        let roll_result = DamageRollResult {
            label: "Sword of Flame".to_string(),
            components: vec![
                DamageComponentResult {
                    damage_type: DamageType::Slashing,
                    result: DiceSetRollResult {
                        label: "Base damage".to_string(),
                        rolls: vec![3, 4],
                        die_size: DieSize::D6,
                        modifiers: ModifierSet::new(),
                        subtotal: 7,
                    },
                },
                DamageComponentResult {
                    damage_type: DamageType::Fire,
                    result: DiceSetRollResult {
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
                operation: MitigationOperation::Resistance,
            }],
        );
        resistances.effects.insert(
            DamageType::Fire,
            vec![DamageMitigationEffect {
                source: ModifierSource::Item("Ring of Fire Immunity".to_string()),
                operation: MitigationOperation::Immunity,
            }],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // Slashing: 7 / 2 = 3.5 -> round down to 3
        // Fire: 2 * 0 = 0
        assert_eq!(mitigation_result.total, 3);
        println!("{:?}", mitigation_result);
    }

    #[test]
    fn damage_mitigation_immunity_priority() {
        let roll_result = DamageRollResult {
            label: "Sword".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Slashing,
                result: DiceSetRollResult {
                    label: "Base damage".to_string(),
                    rolls: vec![3, 4],
                    die_size: DieSize::D6,
                    modifiers: ModifierSet::new(),
                    subtotal: 7,
                },
            }],
            total: 7,
        };

        let mut resistances = DamageResistances {
            effects: HashMap::new(),
        };
        resistances.effects.insert(
            DamageType::Slashing,
            vec![
                DamageMitigationEffect {
                    source: ModifierSource::Item("Shield of Resistance".to_string()),
                    operation: MitigationOperation::Resistance,
                },
                DamageMitigationEffect {
                    source: ModifierSource::Spell("Curse of Slashing".to_string()),
                    operation: MitigationOperation::Vulnerability,
                },
                DamageMitigationEffect {
                    source: ModifierSource::Item("Shield of Slashing Immunity".to_string()),
                    operation: MitigationOperation::Immunity,
                },
            ],
        );

        let mitigation_result = resistances.apply(&roll_result);
        // Slashing immunity takes priority
        println!("{:?}", mitigation_result);
        assert_eq!(mitigation_result.total, 0);
        assert_eq!(mitigation_result.components.len(), 1);
    }
}
