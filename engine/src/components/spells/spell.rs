use std::{hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    components::{
        ability::Ability,
        actions::action::{Action, ActionKind, TargetingFunction},
        id::{IdProvider, ResourceId, ScriptId, SpellId},
        resource::{ResourceAmount, ResourceAmountMap},
    },
    registry::serialize::spell::SpellDefinition,
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
    school: MagicSchool,
    action: Action,
    base_level: u8,
}

impl Spell {
    pub fn new(
        id: SpellId,
        description: String,
        base_level: u8,
        school: MagicSchool,
        kind: ActionKind,
        resource_cost: ResourceAmountMap,
        targeting: Arc<TargetingFunction>,
        reaction_trigger: Option<ScriptId>,
    ) -> Self {
        let action_id = id.clone().into();
        let mut resource_cost = resource_cost;
        if base_level > 0
            && !resource_cost.contains_key(&ResourceId::new("nat20_rs", "resource.spell_slot"))
        {
            // TODO: Not sure if this is a good crutch to lean on?
            // Ensure the spell has a spell slot cost if it's not a cantrip
            resource_cost.insert(
                ResourceId::new("nat20_rs", "resource.spell_slot"),
                ResourceAmount::Tiered {
                    tier: base_level,
                    amount: 1,
                },
            );
        }

        Self {
            id,
            school,
            base_level,
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
        for (resource, cost) in self.action.resource_cost() {
            if *resource == ResourceId::new("nat20_rs", "resource.spell_slot") {
                match cost {
                    ResourceAmount::Tiered { tier, .. } => {
                        return *tier;
                    }
                    _ => {
                        panic!("Spell slot resource cost must be tiered");
                    }
                }
            }
        }
        // TODO: What to do if no spell slot cost is found?
        0
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
}

impl IdProvider for Spell {
    type Id = SpellId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

pub const SPELL_CASTING_ABILITIES: &[Ability; 3] =
    &[Ability::Intelligence, Ability::Wisdom, Ability::Charisma];
