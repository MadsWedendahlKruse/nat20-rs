use crate::dice::dice::*;
use crate::spells::spell::{SpellFlag, SpellSnapshot};
use crate::stats::d20_check::D20CheckResult;
use crate::stats::modifier::{ModifierSet, ModifierSource};
use crate::stats::saving_throw::SavingThrowDC;
use crate::utils::id::SpellId;

use std::collections::{HashMap, HashSet};
use std::fmt::{self};

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

impl fmt::Display for DamageComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.dice_roll, self.damage_type)
    }
}

#[derive(Debug, Clone)]
pub struct DamageComponentResult {
    pub result: DiceSetRollResult,
    pub damage_type: DamageType,
}

impl fmt::Display for DamageComponentResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} ", self.result.subtotal, self.damage_type)?;
        write!(
            f,
            "({} ({}d{})",
            self.result.rolls.iter().sum::<u32>(),
            self.result.rolls.len(),
            self.result.die_size as u32,
        )?;
        if !self.result.modifiers.is_empty() {
            write!(f, " {}", self.result.modifiers)?;
        }
        write!(f, ")")
    }
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
        self.roll_internal(1)
    }

    pub fn roll_crit(&self, crit: bool) -> DamageRollResult {
        if crit {
            self.roll_internal(2)
        } else {
            self.roll_internal(1)
        }
    }

    fn roll_internal(&self, repeat: u32) -> DamageRollResult {
        let mut results = Vec::new();
        let mut total = 0;

        let mut damage_components = self.bonus.clone();
        damage_components.push(self.primary.clone());

        for component in damage_components {
            let mut component_dice_roll = component.dice_roll.clone();
            component_dice_roll.dice.num_dice *= repeat;
            let result = component_dice_roll.roll();
            total += result.subtotal;
            results.push(DamageComponentResult {
                damage_type: component.damage_type,
                result,
            });
        }

        DamageRollResult {
            label: self.label.clone(),
            components: results,
            total,
        }
    }
}

impl fmt::Display for DamageRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.primary)?;
        for comp in &self.bonus {
            write!(f, " + ({})", comp)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DamageRollResult {
    pub label: String,
    pub components: Vec<DamageComponentResult>,
    pub total: i32,
}

impl fmt::Display for DamageRollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.components[0])?;
        for comp in &self.components[1..] {
            write!(f, " + {}", comp)?;
        }
        write!(f, " = {}", self.total)
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
            MitigationOperation::FlatReduction(amount) => write!(f, "- {}", amount),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamageMitigationEffect {
    pub source: ModifierSource,
    pub operation: MitigationOperation,
}

#[derive(Debug, Clone)]
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
                original: comp.result.clone(),
                after_mods: value,
                modifiers: applied_mods,
            });
        }

        DamageMitigationResult { components, total }
    }
}

#[derive(Debug, Clone)]
pub struct DamageComponentMitigation {
    pub damage_type: DamageType,
    pub original: DiceSetRollResult,
    pub after_mods: i32,
    /// Sorted by priority
    pub modifiers: Vec<DamageMitigationEffect>,
}

impl fmt::Display for DamageComponentMitigation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.original.subtotal == self.after_mods {
            return write!(f, "{} {}", self.original.subtotal, self.damage_type);
        }
        if self.after_mods == 0 {
            return write!(
                f,
                "0 {} ({:?})",
                self.damage_type,
                MitigationOperation::Immunity
            );
        }
        write!(f, "{} {} ", self.after_mods, self.damage_type)?;
        let mut amount = self.original.subtotal.to_string();
        for modifier in &self.modifiers {
            let explanation = match modifier.operation {
                MitigationOperation::FlatReduction(_) => format!("{}", modifier.source),
                _ => format!("{:?}", modifier.operation),
            };
            amount = format!("({} {} ({}))", amount, modifier.operation, explanation);
        }
        write!(f, "{}", amount)
    }
}

#[derive(Debug, Clone)]
pub struct DamageMitigationResult {
    pub components: Vec<DamageComponentMitigation>,
    pub total: i32,
}

impl fmt::Display for DamageMitigationResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.components[0])?;
        for comp in &self.components[1..] {
            write!(f, " + {}", comp)?;
        }
        write!(f, " = {}", self.total)
    }
}

#[derive(Debug, Clone)]
pub enum DamageSource {
    Attack {
        attack_roll_result: D20CheckResult,
    },
    Spell {
        spell_id: SpellId,
        spell_flags: HashSet<SpellFlag>,
        attack_roll_result: Option<D20CheckResult>,
        saving_throw_dc: Option<SavingThrowDC>,
    },
}

impl DamageSource {
    pub fn spell(spell: &SpellSnapshot) -> Self {
        Self::Spell {
            spell_id: spell.id.clone(),
            spell_flags: spell.flags.clone(),
            attack_roll_result: None,
            saving_throw_dc: None,
        }
    }

    pub fn spell_with_attack_roll(
        spell: &SpellSnapshot,
        attack_roll_result: D20CheckResult,
    ) -> Self {
        Self::Spell {
            spell_id: spell.id.clone(),
            spell_flags: spell.flags.clone(),
            attack_roll_result: Some(attack_roll_result),
            saving_throw_dc: None,
        }
    }

    pub fn spell_with_saving_throw(spell: &SpellSnapshot, saving_throw: SavingThrowDC) -> Self {
        Self::Spell {
            spell_id: spell.id.clone(),
            spell_flags: spell.flags.clone(),
            attack_roll_result: None,
            saving_throw_dc: Some(saving_throw),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use crate::stats::{ability::Ability, modifier::ModifierSet, modifier::ModifierSource};

    use super::*;

    #[rstest]
    fn damage_roll_values(damage_roll: DamageRoll) {
        println!("Roll: {}", damage_roll);
        let result = damage_roll.roll();
        assert_eq!(result.components.len(), 2);
        // 2d6 + 1d4 + 2 (str mod)
        // Min roll: 2 + 1 + 2 = 5
        // Max roll: 12 + 4 + 2 = 18
        assert!(result.total >= 5 && result.total <= 18);
        println!("Roll result:{}", result);
    }

    #[rstest]
    fn damage_roll_crit(damage_roll: DamageRoll) {
        println!("Roll: {}", damage_roll);
        let result = damage_roll.roll_crit(true);
        assert_eq!(result.components.len(), 2);
        // 4d6 (2 * 2d6) + 2d4 (2 * 1d4) + 2 (str mod)
        // Min roll: 4 + 2 + 2 = 8
        // Max roll: 24 + 8 + 2 = 34
        assert!(result.total >= 8 && result.total <= 34);
        println!("Roll result: {}", result);
    }

    #[rstest]
    fn damage_mitigation_resistance(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // 7 / 2 + 2 = 3.5 + 2 = 5.5 -> round down to 5
        assert_eq!(mitigation_result.total, 5);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_immunity(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // 7 + 0 = 7
        assert_eq!(mitigation_result.total, 7);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_vulnerability(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // 7 * 2 + 2 = 16
        assert_eq!(mitigation_result.total, 16);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_flat_reduction(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // 7 - 3 + 2 = 6
        assert_eq!(mitigation_result.total, 6);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_multiple_effects(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // Slashing: 7 - 3 / 2 = 2
        // 2 Slashing + 2 Fire = 4
        assert_eq!(mitigation_result.total, 4);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_multiple_types(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // Slashing: 7 / 2 = 3.5 -> round down to 3
        // Fire: 2 * 0 = 0
        assert_eq!(mitigation_result.total, 3);
        println!("{}", mitigation_result);
    }

    #[rstest]
    fn damage_mitigation_immunity_priority(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // Slashing immunity takes priority
        println!("{}", mitigation_result);
        // 7 * 0 = 0 slashing
        // 0 slashing + 2 fire = 2
        assert_eq!(mitigation_result.total, 2);
        assert_eq!(mitigation_result.components.len(), 2);
    }

    #[rstest]
    fn damage_mitigation_flat_reduction_and_resistance(damage_roll_result: DamageRollResult) {
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

        let mitigation_result = resistances.apply(&damage_roll_result);
        // Slashing: (7 - 3) / 2 = 2
        // 2 Slashing + 2 Fire = 4
        assert_eq!(mitigation_result.total, 4);
        println!("{}", mitigation_result);
    }

    #[fixture]
    fn damage_roll() -> DamageRoll {
        let mut modifiers = ModifierSet::new();
        modifiers.add_modifier(ModifierSource::Ability(Ability::Strength), 2);
        DamageRoll {
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
        }
    }

    #[fixture]
    fn damage_roll_result() -> DamageRollResult {
        DamageRollResult {
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
                        label: "Fire Enchant".to_string(),
                        rolls: vec![2],
                        die_size: DieSize::D4,
                        modifiers: ModifierSet::new(),
                        subtotal: 2,
                    },
                },
            ],
            total: 9,
        }
    }
}
