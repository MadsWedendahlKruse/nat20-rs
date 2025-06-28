use std::{collections::HashMap, fmt, hash::Hash, sync::Arc};

use crate::{
    actions::{
        action::{Action, ActionContext, ActionKind, ActionKindSnapshot},
        targeting::TargetingContext,
    },
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

#[derive(Debug, Clone)]
pub struct Spell {
    id: SpellId,
    base_level: u8,
    school: MagicSchool,
    action: Action,
}

impl Spell {
    pub fn new(
        id: SpellId,
        base_level: u8,
        school: MagicSchool,
        kind: ActionKind,
        resource_cost: HashMap<ResourceId, u8>,
        targeting: Arc<dyn Fn(&Character, &ActionContext) -> TargetingContext + Send + Sync>,
    ) -> Self {
        let action_id = ActionId::from_str(&id.to_string());
        Self {
            id,
            base_level,
            school,
            action: Action {
                id: action_id,
                kind,
                resource_cost,
                targeting,
                cooldown: None,
            },
        }
    }

    pub fn id(&self) -> &SpellId {
        &self.id
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
        // TODO: Something like BG3 Lightning Charges with Magic Missile would not work
        // with this snapshotting, since each damage instance would add an effect to the
        // caster, which would not be reflected in the snapshot.
        // ---
        // Might not be an issue anymore???
        Ok(self.action.kind().snapshot(
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
        let spellcasting_modifier = caster.ability_scores().ability_modifier(ability).total();
        spell_save_dc.add_modifier(ModifierSource::Ability(ability), spellcasting_modifier);
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

    pub fn spell_attack_roll(caster: &Character, spellcasting_ability: Ability) -> AttackRoll {
        let mut roll = D20Check::new(Proficiency::Proficient);
        let spellcasting_modifier = caster
            .ability_scores()
            .ability_modifier(spellcasting_ability)
            .total();
        roll.add_modifier(
            ModifierSource::Ability(spellcasting_ability),
            spellcasting_modifier,
        );

        AttackRoll::new(roll, DamageSource::Spell)
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
}
