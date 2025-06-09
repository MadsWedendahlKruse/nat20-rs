use std::{fmt, hash::Hash, sync::Arc};

use crate::{
    combat::damage::{
        AttackRoll, AttackRollResult, DamageEventResult, DamageMitigationResult, DamageRoll,
        DamageRollResult, DamageSource,
    },
    creature::character::Character,
    dice::dice::{DiceSetRoll, DiceSetRollResult},
    effects::effects::Effect,
    stats::{
        ability::Ability,
        d20_check::D20Check,
        modifier::{ModifierSet, ModifierSource},
        proficiency::Proficiency,
        saving_throw::SavingThrowDC,
    },
    utils::id::{CharacterId, SpellId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MagicSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

#[derive(Debug, Clone)]
pub enum TargetingContext {
    Single,
    Multiple(u8), // up to N targets
    // TODO: If it's not centered on the caster, we need to specify the point of origin
    AreaOfEffect {
        radius: u32,
        centered_on_caster: bool,
    },
}

#[derive(Clone)]
pub enum SpellKind {
    /// Spells that deal unconditional damage. Is this only Magic Missile?
    Damage {
        damage: Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>,
    },
    /// Spells that require an attack roll to hit a target, and deal damage on hit.
    /// Some spells may have a damage roll on a failed attack roll (e.g. Acid Arrow)
    AttackRoll {
        damage: Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>,
        damage_on_failure: Option<Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>>,
    },
    /// Spells that require a saving throw to avoid or reduce damage.
    /// Most of the time, these spells will deal damage on a failed save,
    /// and half damage on a successful save.
    SavingThrowDamage {
        saving_throw: Ability,
        half_damage_on_save: bool,
        damage: Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>,
    },
    /// Spells that require a saving throw to avoid or reduce an effect.
    SavingThrowEffect {
        saving_throw: Ability,
        effect: Effect,
    },
    /// Spells that apply a beneficial effect to a target, and therefore do not require
    /// an attack roll or saving throw (e.g. Bless, Shield of Faith).
    BeneficialEffect { effect: Effect },
    /// Spells that heal a target. These spells do not require an attack roll or saving throw.
    /// They simply heal the target for a certain amount of hit points.
    Healing {
        heal: Arc<dyn Fn(&Character, &u8) -> DiceSetRoll + Send + Sync>,
    },
    /// Utility spells that do not deal damage or heal, but have some other effect.
    /// These spells may include buffs, debuffs, or other effects that do not fit into the
    /// other categories (e.g. teleportation, Knock, etc.).
    Utility {
        // E.g. Arcane Lock, Invisibility, etc.
        // Add hooks or custom closures as needed
    },
    /// Custom spells can have any kind of effect, including damage, healing, or utility.
    /// The closure should return a `SpellKindSnapshot` that describes the effect of the spell.
    /// Please note that this should only be used for spells that don't fit into the
    /// standard categories.
    Custom(Arc<dyn Fn(&Character, &u8) -> SpellKindSnapshot + Send + Sync>),
}

impl SpellKind {
    pub fn snapshot(
        &self,
        caster: &Character,
        spell_level: &u8,
        ability: Ability,
    ) -> SpellKindSnapshot {
        match self {
            SpellKind::Damage { damage } => SpellKindSnapshot::Damage {
                damage_roll: damage(caster, spell_level).roll(),
            },

            SpellKind::AttackRoll {
                damage,
                damage_on_failure,
            } => {
                let attack_roll = spell_attack_roll(caster, ability);
                let is_crit = attack_roll.roll_result.is_crit;
                SpellKindSnapshot::AttackRoll {
                    attack_roll,
                    damage_roll: damage(caster, spell_level).roll_crit_damage(is_crit),
                    damage_on_failure: damage_on_failure
                        .as_ref()
                        .map(|f| f(caster, spell_level).roll_crit_damage(is_crit)),
                }
            }

            SpellKind::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage,
            } => SpellKindSnapshot::SavingThrowDamage {
                saving_throw: SavingThrowDC {
                    key: *saving_throw,
                    dc: spell_save_dc(caster, *saving_throw),
                },
                half_damage_on_save: *half_damage_on_save,
                damage_roll: damage(caster, spell_level).roll(),
            },

            SpellKind::SavingThrowEffect {
                saving_throw,
                effect,
            } => SpellKindSnapshot::SavingThrowEffect {
                saving_throw: SavingThrowDC {
                    key: *saving_throw,
                    dc: spell_save_dc(caster, *saving_throw),
                },
                effect: effect.clone(),
            },

            SpellKind::BeneficialEffect { effect } => SpellKindSnapshot::BeneficialEffect {
                effect: effect.clone(),
            },

            SpellKind::Healing { heal } => SpellKindSnapshot::Healing {
                healing: heal(caster, spell_level).roll(),
            },

            SpellKind::Utility {} => SpellKindSnapshot::Utility,

            SpellKind::Custom(effect) => effect(caster, spell_level),
        }
    }
}

impl fmt::Debug for SpellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpellKind::Damage { .. } => write!(f, "Damage"),
            SpellKind::AttackRoll { .. } => write!(f, "AttackRoll"),
            SpellKind::SavingThrowDamage { .. } => write!(f, "SavingThrowDamage"),
            SpellKind::SavingThrowEffect { .. } => write!(f, "SavingThrowEffect"),
            SpellKind::BeneficialEffect { .. } => write!(f, "BeneficialEffect"),
            SpellKind::Healing { .. } => write!(f, "Healing"),
            SpellKind::Utility { .. } => write!(f, "Utility"),
            SpellKind::Custom(_) => write!(f, "Custom"),
        }
    }
}

const BASE_SPELL_SAVE_DC: i32 = 8;

fn spell_save_dc(caster: &Character, ability: Ability) -> ModifierSet {
    let mut spell_save_dc = ModifierSet::new();
    spell_save_dc.add_modifier(
        ModifierSource::Custom("Base spell save DC".to_string()),
        BASE_SPELL_SAVE_DC,
    );
    let spell_casting_modifier = caster.ability_scores().ability_modifier(ability).total();
    spell_save_dc.add_modifier(ModifierSource::Ability(ability), spell_casting_modifier);
    // TODO: Not sure if Proficiency is the correct modifier source here, since I don't think
    // you can have e.g. Expertise in spell save DCs.
    spell_save_dc.add_modifier(
        ModifierSource::Proficiency(Proficiency::Proficient),
        caster.proficiency_bonus() as i32,
    );
    spell_save_dc
}

fn spell_attack_roll(caster: &Character, ability: Ability) -> AttackRollResult {
    let mut roll = D20Check::new(Proficiency::Proficient);
    let spell_casting_modifier = caster.ability_scores().ability_modifier(ability).total();
    roll.add_modifier(ModifierSource::Ability(ability), spell_casting_modifier);

    let attack_roll = AttackRoll::new(roll, DamageSource::Spell);

    attack_roll.roll(caster)
}

/// To avoid the issue of not being able to borrow the caster immutably and the
/// target mutably at the same time, we need to create a snapshot of the spell that
/// has already taken into account the caster's stats and abilities. So all the rolls
/// and checks should be precomputed in the snapshot (as results of the rolls),
/// and the spell should be able to apply those results to the target when cast.
#[derive(Debug, Clone)]
pub enum SpellKindSnapshot {
    Damage {
        damage_roll: DamageRollResult,
    },
    AttackRoll {
        attack_roll: AttackRollResult,
        damage_roll: DamageRollResult,
        damage_on_failure: Option<DamageRollResult>,
    },
    SavingThrowDamage {
        saving_throw: SavingThrowDC,
        half_damage_on_save: bool,
        damage_roll: DamageRollResult,
    },
    SavingThrowEffect {
        saving_throw: SavingThrowDC,
        effect: Effect,
    },
    BeneficialEffect {
        effect: Effect,
    },
    Healing {
        healing: DiceSetRollResult,
    },
    Utility,
    Custom {
        damage_roll: DamageRollResult,
        // TODO: Add more fields as needed for custom spells
    },
}

#[derive(Debug)]
pub enum SpellKindResult {
    Damage {
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
    AttackRoll {
        attack_roll: AttackRollResult,
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
    SavingThrowDamage {
        saving_throw: SavingThrowDC,
        half_damage_on_save: bool,
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
    SavingThrowEffect {
        saving_throw: SavingThrowDC,
        effect: Effect,
        applied: bool,
    },
    BeneficialEffect {
        effect: Effect,
        applied: bool,
    },
    Healing {
        healing: DiceSetRollResult,
    },
    Utility,
    Custom {
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
}

/// Represents the result of casting a spell on a single target. For spells that affect multiple targets,
/// multiple `SpellResult` instances can be collected.
#[derive(Debug)]
pub struct SpellResult {
    // TODO: What if the target isn't a Character, but e.g. an object? Like if you cast
    // Knock on a door?
    pub target: CharacterId,
    pub result: SpellKindResult,
}

impl fmt::Display for SpellResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Target: {}\n", self.target)?;
        match &self.result {
            SpellKindResult::Damage {
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage_taken) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage_taken)?;
                }
            }

            SpellKindResult::AttackRoll {
                attack_roll,
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tAttack Roll: {}\n", attack_roll)?;
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage_taken) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage_taken)?;
                }
            }

            SpellKindResult::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tSaving Throw: {}\n", saving_throw)?;
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                // TODO: How do we know if the saving throw was successful?
                if let Some(damage_taken) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage_taken)?;
                }
            }

            SpellKindResult::Healing { healing } => {
                write!(f, "\tHealing: {}\n", healing)?;
            }

            SpellKindResult::BeneficialEffect { effect, applied } => {
                // write!(f, "\tEffect: {}\n", effect)?;
                write!(f, "\tApplied: {}\n", applied)?;
            }

            SpellKindResult::SavingThrowEffect {
                saving_throw,
                effect,
                applied,
            } => {
                write!(f, "\tSaving Throw: {}\n", saving_throw)?;
                // write!(f, "\tEffect: {}\n", effect)?;
                write!(f, "\tApplied: {}\n", applied)?;
            }

            SpellKindResult::Utility {} => {
                write!(f, "\tUtility spell with no specific result.\n")?;
            }

            SpellKindResult::Custom {
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage_taken) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage_taken)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Spell {
    id: SpellId,
    name: String,
    base_level: u8,
    school: MagicSchool,
    kind: SpellKind,
    targeting: Arc<dyn Fn(&Character, &u8) -> TargetingContext + Send + Sync>,
    spellcasting_ability: Option<Ability>,
}

impl Spell {
    pub fn new(
        name: String,
        base_level: u8,
        school: MagicSchool,
        kind: SpellKind,
        targeting: Arc<dyn Fn(&Character, &u8) -> TargetingContext + Send + Sync>,
    ) -> Self {
        Self {
            id: name.to_uppercase().replace(" ", "_").into(),
            name,
            base_level,
            school,
            targeting,
            kind,
            // effect,
            spellcasting_ability: None,
        }
    }

    pub fn id(&self) -> &SpellId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base_level(&self) -> u8 {
        self.base_level
    }

    pub fn is_cantrip(&self) -> bool {
        self.base_level == 0
    }

    pub fn school(&self) -> MagicSchool {
        self.school
    }

    // pub fn cast(&self, caster: &Character, level: &u8, target: &mut Character) -> SpellResult {
    //     (self.effect)(self, caster, level, target)
    // }

    pub fn spellcasting_ability(&self) -> Option<Ability> {
        self.spellcasting_ability
    }

    /// This should be called when the spell is learned, so it can be set to spell casting ability
    /// of the class which learned the spell.
    pub fn set_spellcasting_ability(&mut self, ability: Ability) {
        self.spellcasting_ability = Some(ability);
    }

    pub fn snapshot(
        &self,
        caster: &Character,
        spell_level: &u8,
    ) -> Result<SpellSnapshot, SnapshotError> {
        if spell_level < &self.base_level {
            return Err(SnapshotError::DowncastingNotAllowed(
                self.base_level,
                *spell_level,
            ));
        }
        if self.is_cantrip() && spell_level > &self.base_level {
            return Err(SnapshotError::UpcastingCantripNotAllowed);
        }
        if self.spellcasting_ability.is_none() {
            return Err(SnapshotError::SpellcastingAbilityNotSet);
        }
        // TODO: Something like BG3 Lightning Charges with Magic Missile would not work
        // with this snapshotting, since each damage instance would add an effect to the
        // caster, which would not be reflected in the snapshot.
        // ---
        // Might not be an issue anymore???
        Ok(SpellSnapshot {
            id: self.id.clone(),
            name: self.name.clone(),
            base_level: self.base_level,
            school: self.school,
            kind: self
                .kind
                .snapshot(caster, spell_level, self.spellcasting_ability.unwrap()),
            targeting_context: (self.targeting)(caster, spell_level),
        })
    }
}

impl fmt::Debug for Spell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spell")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_level", &self.base_level)
            .field("school", &self.school)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SpellSnapshot {
    pub id: SpellId,
    pub name: String,
    pub base_level: u8,
    pub school: MagicSchool,
    pub kind: SpellKindSnapshot,
    pub targeting_context: TargetingContext,
}

impl SpellSnapshot {
    pub fn cast(&self, target: &mut Character) -> SpellResult {
        let spell_kind_result = match &self.kind {
            SpellKindSnapshot::Damage { damage_roll } => {
                let damage_taken = target.take_damage(&self.damage_source());
                SpellKindResult::Damage {
                    damage_roll: damage_roll.clone(),
                    damage_taken,
                }
            }

            SpellKindSnapshot::AttackRoll {
                attack_roll,
                damage_roll,
                damage_on_failure,
            } => {
                let damage_taken = target.take_damage(&self.damage_source());

                // TODO: How do we know if the attack roll was successful? i.e. what damage roll to use?

                SpellKindResult::AttackRoll {
                    attack_roll: attack_roll.clone(),
                    damage_roll: damage_roll.clone(),
                    damage_taken: damage_taken,
                }
            }

            SpellKindSnapshot::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage_roll,
            } => {
                todo!()
            }

            SpellKindSnapshot::SavingThrowEffect {
                saving_throw,
                effect,
            } => {
                todo!()
            }

            SpellKindSnapshot::BeneficialEffect { effect } => {
                // TODO: Isn't it just always going to be applied?
                let applied = target.add_effect(effect.clone());
                SpellKindResult::BeneficialEffect {
                    effect: effect.clone(),
                    applied: true,
                }
            }

            SpellKindSnapshot::Healing { healing: heal } => {
                todo!()
            }

            SpellKindSnapshot::Utility => {
                todo!()
            }

            SpellKindSnapshot::Custom {
                damage_roll: damage,
            } => {
                let damage_taken = target.take_damage(&self.damage_source());
                SpellKindResult::Custom {
                    damage_roll: damage.clone(),
                    damage_taken,
                }
            }
        };

        SpellResult {
            target: target.id(),
            result: spell_kind_result,
        }
    }

    fn damage_source(&self) -> DamageEventResult {
        DamageEventResult::Spell(self.kind.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// Downcasting a spell to a lower level is not allowed, e.g. Fireball is a 3rd level spell
    /// and cannot be downcast to a 1st or 2nd level spell.
    /// (base_level, requested_level)
    DowncastingNotAllowed(u8, u8),
    /// Cantrips cannot be upcast, so this error is returned when trying to upcast a cantrip.
    /// This is not supposed to be allowed, so the option should not be presented to the player.
    UpcastingCantripNotAllowed,
    /// The spellcasting ability has not been set for this spell. This usually means it hasn't
    /// been set by the class that learned the spell.
    /// This is a programming error, so it should not happen in normal gameplay.
    SpellcastingAbilityNotSet,
}

#[cfg(test)]
mod tests {
    use crate::{combat::damage::DamageType, dice::dice::DieSize, test_utils::fixtures};

    use super::*;

    #[test]
    fn spell_creation() {
        let spell = Spell::new(
            "Abrakadabra".to_string(),
            1,
            MagicSchool::Conjuration,
            SpellKind::Utility {},
            Arc::new(|_, _| TargetingContext::AreaOfEffect {
                radius: 20,
                centered_on_caster: true,
            }),
        );

        assert_eq!(spell.name(), "Abrakadabra");
        assert_eq!(spell.base_level(), 1);
        assert_eq!(spell.school(), MagicSchool::Conjuration);
    }

    #[test]
    fn spell_targeting() {
        let caster = fixtures::creatures::heroes::wizard();
        let spell_id = fixtures::spells::magic_missile().id().clone();
        let snapshot = caster.spell_snapshot(&spell_id, 1).unwrap().unwrap();

        match snapshot.targeting_context {
            TargetingContext::Multiple(count) => {
                assert_eq!(
                    count, 3,
                    "Expected {} targets for level {} Magic Missile",
                    3, 1
                );
            }
            _ => panic!("Expected Multiple targeting context"),
        }
    }

    #[test]
    fn spell_targeting_upcasting() {
        let caster = fixtures::creatures::heroes::wizard();
        let spell_id = fixtures::spells::magic_missile().id().clone();
        // Upcasting Magic Missile to level 3
        // should increase the number of targets to 5.
        let snapshot = caster.spell_snapshot(&spell_id, 3).unwrap().unwrap();

        match snapshot.targeting_context {
            TargetingContext::Multiple(count) => {
                assert_eq!(
                    count, 5,
                    "Expected {} targets for level {} Magic Missile",
                    5, 3
                );
            }
            _ => panic!("Expected Multiple targeting context"),
        }
    }

    #[test]
    fn spell_damage() {
        let caster = fixtures::creatures::heroes::wizard();
        let spell_id = fixtures::spells::fireball().id().clone();
        let snapshot = caster.spell_snapshot(&spell_id, 3).unwrap().unwrap();

        let damage_roll: &DamageRollResult = match &snapshot.kind {
            SpellKindSnapshot::SavingThrowDamage { damage_roll, .. } => damage_roll,
            _ => panic!("Expected Fireball to be a SavingThrowDamage spell"),
        };

        assert_eq!(
            damage_roll.components[0].damage_type,
            DamageType::Fire,
            "Expected Fireball damage roll to be Fire"
        );
        assert_eq!(
            damage_roll.components[0].result.rolls.len(),
            8,
            "Expected Fireball to roll 8d6 at level 3"
        );
        assert_eq!(
            damage_roll.components[0].result.die_size,
            DieSize::D6,
            "Expected Fireball to roll 8d6 at level 3"
        );
    }

    #[test]
    fn spell_damage_upcasting() {
        let caster = fixtures::creatures::heroes::wizard();
        let spell_id = fixtures::spells::fireball().id().clone();
        let snapshot = caster.spell_snapshot(&spell_id, 5).unwrap().unwrap();

        let damage_roll: &DamageRollResult = match &snapshot.kind {
            SpellKindSnapshot::SavingThrowDamage { damage_roll, .. } => damage_roll,
            _ => panic!("Expected Fireball to be a SavingThrowDamage spell"),
        };

        assert_eq!(
            damage_roll.components[0].damage_type,
            DamageType::Fire,
            "Expected Fireball damage roll to be Fire"
        );
        assert_eq!(
            damage_roll.components[0].result.rolls.len(),
            10,
            "Expected Fireball to roll 10d6 at level 5"
        );
        assert_eq!(
            damage_roll.components[0].result.die_size,
            DieSize::D6,
            "Expected Fireball to roll 10d6 at level 5"
        );
    }

    #[test]
    fn cantrip_upcasting() {
        let caster = fixtures::creatures::heroes::warlock();
        let spell_id = fixtures::spells::eldritch_blast().id().clone();

        let result = caster.spell_snapshot(&spell_id, 3).unwrap();

        assert!(
            matches!(result, Err(SnapshotError::UpcastingCantripNotAllowed)),
            "Expected upcasting a cantrip to return an error"
        );
    }

    #[test]
    fn spell_snapshot_downcasting() {
        let caster = fixtures::creatures::heroes::wizard();
        let spell_id = fixtures::spells::fireball().id().clone();

        let result = caster.spell_snapshot(&spell_id, 2).unwrap();

        assert!(
            matches!(result, Err(SnapshotError::DowncastingNotAllowed(3, 2))),
            "Expected downcasting Fireball to level 2 to return an error"
        );
    }

    #[test]
    fn spell_snapshot_spellcasting_ability_not_set() {
        let caster = fixtures::creatures::heroes::warlock();
        // Warlock doesn't know Magic Missile, so the spellcasting ability is not set
        let spell = fixtures::spells::magic_missile();

        // Create a snapshot without setting the spellcasting ability
        let result = spell.snapshot(&caster, &1);

        assert!(
            matches!(result, Err(SnapshotError::SpellcastingAbilityNotSet)),
            "Expected snapshot without spellcasting ability to return an error"
        );
    }

    #[test]
    fn cantrip_level_scaling() {
        let caster = fixtures::creatures::heroes::warlock();
        let spell_id = fixtures::spells::eldritch_blast().id().clone();
        assert_eq!(caster.total_level(), 5, "Expected Warlock to be level 5");

        // Warlock is level 5, so Eldritch Blast should have two beams at this level
        let snapshot = caster.spell_snapshot(&spell_id, 0).unwrap().unwrap();
        assert_eq!(snapshot.base_level, 0, "Cantrip should have base level 0");

        match snapshot.targeting_context {
            TargetingContext::Multiple(count) => {
                assert_eq!(
                    count, 2,
                    "Expected {} beams for Eldritch Blast at caster level 5, but got {}",
                    2, count
                );
            }
            _ => panic!("Expected Multiple targeting context for cantrip"),
        }
    }
}
