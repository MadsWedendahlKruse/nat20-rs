use std::{hash::Hash, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        ability::Ability,
        actions::action::{Action, ActionKind, TargetingFunction},
        id::{EffectId, IdProvider, ScriptId, SpellId},
        resource::ResourceAmountMap,
    },
    registry::serialize::spell::SpellDefinition,
    systems,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "SpellDefinition")]
pub struct Spell {
    id: SpellId,
    base_level: u8,
    school: MagicSchool,
    concentration: bool,
    action: Action,
}

impl Spell {
    pub fn new(
        id: SpellId,
        description: String,
        base_level: u8,
        school: MagicSchool,
        concentration: bool,
        kind: ActionKind,
        resource_cost: ResourceAmountMap,
        targeting: Arc<TargetingFunction>,
        reaction_trigger: Option<ScriptId>,
    ) -> Self {
        let action_id = id.clone().into();

        Self {
            id,
            school,
            base_level,
            concentration,
            action: Action {
                id: action_id,
                description,
                kind,
                resource_cost,
                targeting,
                cooldown: None,
                reaction_trigger,
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
        self.base_level() == 0
    }

    pub fn school(&self) -> MagicSchool {
        self.school
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn requires_concentration(&self) -> bool {
        self.concentration
    }
}

impl IdProvider for Spell {
    type Id = SpellId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

pub const SPELL_CASTING_ABILITIES: &[Ability; 3] =
    &[Ability::Intelligence, Ability::Wisdom, Ability::Charisma];

pub const CONCENTRATION_SAVING_THROW_DC_DEFAULT: i32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcentrationInstance {
    Effect { entity: Entity, effect: EffectId },
    // TODO: Environmental effects (e.g. web)
}

impl ConcentrationInstance {
    pub fn break_concentration(&self, world: &mut World) {
        match self {
            ConcentrationInstance::Effect { entity, effect } => {
                systems::effects::remove_effect(world, *entity, effect);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationTracker {
    pub instances: Vec<ConcentrationInstance>,
}

impl ConcentrationTracker {
    pub fn add_instance(&mut self, instance: ConcentrationInstance) {
        self.instances.push(instance);
    }

    pub fn is_concentrating(&self) -> bool {
        !self.instances.is_empty()
    }
}

impl Default for ConcentrationTracker {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
        }
    }
}
