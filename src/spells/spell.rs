use std::{collections::HashSet, fmt, hash::Hash, sync::Arc};

use crate::{
    combat::damage::{DamageMitigationResult, DamageRoll, DamageRollResult, DamageSource},
    creature::character::Character,
    dice::dice::{DiceSetRoll, DiceSetRollResult},
    stats::{
        ability::Ability,
        d20_check::D20CheckResult,
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

/// Represents the result of casting a spell on a single target. For spells that affect multiple targets,
/// multiple `SpellResult` instances can be collected.
#[derive(Debug)]
pub struct SpellResult {
    pub target: CharacterId,
    pub saving_throw: Option<D20CheckResult>,
    pub damage_roll: Option<DamageRollResult>,
    pub damage_result: Option<DamageMitigationResult>,
    // pub conditions_applied: Option<String>,
    pub healing: Option<DiceSetRollResult>,
}

impl SpellResult {
    pub fn new(target: CharacterId) -> Self {
        Self {
            target,
            saving_throw: None,
            damage_roll: None,
            damage_result: None,
            // conditions_applied: None,
            healing: None,
        }
    }

    pub fn set_damage(
        &mut self,
        damage_roll: DamageRollResult,
        damage_result: DamageMitigationResult,
    ) {
        self.damage_roll = Some(damage_roll);
        self.damage_result = Some(damage_result);
    }

    pub fn set_healing(&mut self, healing: DiceSetRollResult) {
        self.healing = Some(healing);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellFlag {
    SavingThrowHalfDamage,
}

const BASE_SPELL_SAVE_DC: i32 = 8;

#[derive(Clone)]
pub struct Spell {
    id: SpellId,
    name: String,
    base_level: u8,
    school: MagicSchool,
    damage: Option<Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>>,
    healing: Option<Arc<dyn Fn(&Character, &u8) -> DiceSetRoll + Send + Sync>>,
    targeting: Arc<dyn Fn(&Character, &u8) -> TargetingContext + Send + Sync>,
    // effect: Arc<dyn Fn(&Spell, &Character, &u8, &mut Character) -> SpellResult + Send + Sync>,
    saving_throw_ability: Option<Ability>,
    flags: HashSet<SpellFlag>,
}

impl Spell {
    pub fn new(
        name: String,
        base_level: u8,
        school: MagicSchool,
        damage: Option<Arc<dyn Fn(&Character, &u8) -> DamageRoll + Send + Sync>>,
        healing: Option<Arc<dyn Fn(&Character, &u8) -> DiceSetRoll + Send + Sync>>,
        targeting: Arc<dyn Fn(&Character, &u8) -> TargetingContext + Send + Sync>,
        // effect: Arc<dyn Fn(&Spell, &Character, &u8, &mut Character) -> SpellResult + Send + Sync>,
        flags: HashSet<SpellFlag>,
    ) -> Self {
        Self {
            id: name.to_uppercase().replace(" ", "_").into(),
            name,
            base_level,
            school,
            damage,
            healing,
            targeting,
            // effect,
            saving_throw_ability: None,
            flags,
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

    pub fn school(&self) -> MagicSchool {
        self.school
    }

    fn damage_roll(&self, caster: &Character, level: &u8) -> Option<DamageRoll> {
        self.damage.as_ref().map(|f| f(caster, level))
    }

    fn healing_roll(&self, caster: &Character, level: &u8) -> Option<DiceSetRoll> {
        self.healing.as_ref().map(|f| f(caster, level))
    }

    // pub fn cast(&self, caster: &Character, level: &u8, target: &mut Character) -> SpellResult {
    //     (self.effect)(self, caster, level, target)
    // }

    pub fn saving_throw_ability(&self) -> Option<Ability> {
        self.saving_throw_ability
    }

    /// This should be called when the spell is learned, so it can be set to spell casting ability
    /// of the class which learned the spell.
    pub fn set_saving_throw_ability(&mut self, ability: Ability) {
        self.saving_throw_ability = Some(ability);
    }

    pub fn spell_save_dc(&self, caster: &Character) -> ModifierSet {
        let mut spell_save_dc = ModifierSet::new();
        spell_save_dc.add_modifier(
            ModifierSource::Custom("Base spell save DC".to_string()),
            BASE_SPELL_SAVE_DC,
        );
        let spell_casting_modifier = caster
            .ability_scores()
            .ability_modifier(self.saving_throw_ability.unwrap())
            .total();
        spell_save_dc.add_modifier(
            ModifierSource::Ability(self.saving_throw_ability.unwrap()),
            spell_casting_modifier,
        );
        // TODO: Not sure if Proficiency is the correct modifier source here, since I don't think
        // you can have e.g. Expertise in spell save DCs.
        spell_save_dc.add_modifier(
            ModifierSource::Proficiency(Proficiency::Proficient),
            caster.proficiency_bonus(),
        );
        spell_save_dc
    }

    pub fn has_flag(&self, flag: SpellFlag) -> bool {
        self.flags.contains(&flag)
    }

    pub fn flags(&self) -> &HashSet<SpellFlag> {
        &self.flags
    }

    pub fn snapshot(&self, caster: &Character, level: &u8) -> SpellSnapshot {
        SpellSnapshot {
            id: self.id.clone(),
            name: self.name.clone(),
            base_level: self.base_level,
            school: self.school,
            damage: self.damage_roll(caster, level),
            healing: self.healing_roll(caster, level),
            saving_throw: if self.saving_throw_ability.is_none() {
                None
            } else {
                Some(SavingThrowDC {
                    key: self.saving_throw_ability.unwrap(),
                    dc: self.spell_save_dc(caster).total() as u32,
                })
            },
            targeting_context: (self.targeting)(caster, level),
            flags: self.flags.clone(),
        }
    }
}

impl fmt::Debug for Spell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spell")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_level", &self.base_level)
            .field("school", &self.school)
            .field("flags", &self.flags)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SpellSnapshot {
    pub id: SpellId,
    pub name: String,
    pub base_level: u8,
    pub school: MagicSchool,
    pub damage: Option<DamageRoll>,
    pub healing: Option<DiceSetRoll>,
    pub saving_throw: Option<SavingThrowDC>,
    pub targeting_context: TargetingContext,
    pub flags: HashSet<SpellFlag>,
}

impl SpellSnapshot {
    pub fn cast(&self, target: &mut Character) -> SpellResult {
        let mut spell_result = SpellResult::new(target.id());
        // Apply damage or healing effects
        if let Some(damage) = &self.damage {
            let damage_roll = damage.roll();
            let damage_result = target.take_damage(&damage_roll, &self.damage_source());
            if let Some(damage_result) = damage_result {
                spell_result.set_damage(damage_roll, damage_result);
            }
        }
        if let Some(healing) = &self.healing {
            let healing_roll = healing.roll();
            target.heal(healing_roll.subtotal);
            spell_result.set_healing(healing_roll);
        }
        spell_result
    }

    fn damage_source(&self) -> DamageSource {
        if let Some(saving_throw) = &self.saving_throw {
            return DamageSource::spell_with_saving_throw(self, saving_throw.clone());
        }
        // TODO: Handle attack roll
        DamageSource::spell(self)
    }
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
            None,
            None,
            Arc::new(|_, _| TargetingContext::AreaOfEffect {
                radius: 20,
                centered_on_caster: true,
            }),
            HashSet::new(),
        );

        assert_eq!(spell.name(), "Abrakadabra");
        assert_eq!(spell.base_level(), 1);
        assert_eq!(spell.school(), MagicSchool::Conjuration);
    }

    #[test]
    fn spell_targeting() {
        let spell = fixtures::spells::magic_missile();
        let caster = fixtures::characters::hero_wizard();

        let snapshot = spell.snapshot(&caster, &1);

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
        let spell = fixtures::spells::magic_missile();
        let caster = fixtures::characters::hero_wizard();

        let snapshot = spell.snapshot(&caster, &3); // Upcasting to level 3

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
        let spell = fixtures::spells::fireball();
        let caster = fixtures::characters::hero_wizard();
        let level = 3;

        let snapshot = spell.snapshot(&caster, &level);

        assert!(
            snapshot.damage.is_some(),
            "Expected Fireball to have a damage roll"
        );

        let damage_roll = snapshot.damage.unwrap();
        assert_eq!(
            damage_roll.primary.damage_type,
            DamageType::Fire,
            "Expected Fireball damage roll to be Fire"
        );
        assert_eq!(
            damage_roll.primary.dice_roll.dice.num_dice, 8,
            "Expected Fireball to roll 8d6 at level 3"
        );
        assert_eq!(
            damage_roll.primary.dice_roll.dice.die_size,
            DieSize::D6,
            "Expected Fireball to roll 8d6 at level 3"
        );
    }

    #[test]
    fn spell_damage_upcasting() {
        let spell = fixtures::spells::fireball();
        let caster = fixtures::characters::hero_wizard();
        let level = 5;

        let snapshot = spell.snapshot(&caster, &level);

        assert!(
            snapshot.damage.is_some(),
            "Expected Fireball to have a damage roll"
        );

        let damage_roll = snapshot.damage.unwrap();
        assert_eq!(
            damage_roll.primary.damage_type,
            DamageType::Fire,
            "Expected Fireball damage roll to be Fire"
        );
        assert_eq!(
            damage_roll.primary.dice_roll.dice.num_dice, 10,
            "Expected Fireball to roll 10d6 at level 5"
        );
        assert_eq!(
            damage_roll.primary.dice_roll.dice.die_size,
            DieSize::D6,
            "Expected Fireball to roll 10d6 at level 5"
        );
    }
}
