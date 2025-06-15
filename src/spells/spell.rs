use std::{collections::HashMap, fmt, hash::Hash, sync::Arc};

use crate::{
    actions::action::{Action, ActionContext, ActionKind, ActionKindSnapshot, TargetingContext},
    combat::damage::{AttackRoll, DamageSource},
    creature::character::Character,
    stats::{
        ability::Ability,
        d20_check::{D20Check, D20CheckDC},
        modifier::{ModifierSet, ModifierSource},
        proficiency::Proficiency,
    },
    utils::id::{ActionId, ResourceId, SpellId},
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

// TODO: Not sure where to put these functions

#[derive(Clone)]
pub struct Spell {
    id: SpellId,
    name: String,
    base_level: u8,
    school: MagicSchool,
    action: Action,
    spellcasting_ability: Option<Ability>,
}

impl Spell {
    pub fn new(
        name: String,
        base_level: u8,
        school: MagicSchool,
        kind: ActionKind,
        resource_cost: HashMap<ResourceId, u8>,
        targeting: Arc<dyn Fn(&Character, &ActionContext) -> TargetingContext + Send + Sync>,
    ) -> Self {
        let spell_id = SpellId::from_str(&name.to_uppercase().replace(" ", "_"));
        let action_id = ActionId::from_str(&spell_id.to_string());
        Self {
            id: spell_id,
            name,
            base_level,
            school,
            action: Action {
                id: action_id,
                kind,
                resource_cost,
                targeting,
            },
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

    pub fn spellcasting_ability(&self) -> Option<Ability> {
        self.spellcasting_ability
    }

    /// This should be called when the spell is learned, so it can be set to spell casting ability
    /// of the class which learned the spell.
    pub fn set_spellcasting_ability(&mut self, ability: Ability) {
        self.spellcasting_ability = Some(ability);
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn snapshot(
        &self,
        caster: &Character,
        spell_level: &u8,
    ) -> Result<ActionKindSnapshot, SnapshotError> {
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
        Ok(self.action.kind.snapshot(
            caster,
            &ActionContext::Spell {
                level: *spell_level,
            },
        ))
    }

    const BASE_SPELL_SAVE_DC: i32 = 8;

    pub fn spell_save_dc(caster: &Character, ability: Ability) -> D20CheckDC<Ability> {
        let mut spell_save_dc = ModifierSet::new();
        spell_save_dc.add_modifier(
            ModifierSource::Custom("Base spell save DC".to_string()),
            Spell::BASE_SPELL_SAVE_DC,
        );
        let spell_casting_modifier = caster.ability_scores().ability_modifier(ability).total();
        spell_save_dc.add_modifier(ModifierSource::Ability(ability), spell_casting_modifier);
        // TODO: Not sure if Proficiency is the correct modifier source here, since I don't think
        // you can have e.g. Expertise in spell save DCs.
        spell_save_dc.add_modifier(
            ModifierSource::Proficiency(Proficiency::Proficient),
            caster.proficiency_bonus() as i32,
        );

        D20CheckDC {
            key: ability,
            dc: spell_save_dc,
        }
    }

    pub fn spell_attack_roll(caster: &Character, ability: Ability) -> AttackRoll {
        let mut roll = D20Check::new(Proficiency::Proficient);
        let spell_casting_modifier = caster.ability_scores().ability_modifier(ability).total();
        roll.add_modifier(ModifierSource::Ability(ability), spell_casting_modifier);

        AttackRoll::new(roll, DamageSource::Spell)
    }
}

impl fmt::Debug for Spell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spell")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_level", &self.base_level)
            .field("school", &self.school)
            .field("action", &self.action)
            .field("action_cost", &self.action.resource_cost)
            .finish()
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
