use std::{hash::Hash, sync::Arc};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{Action, ActionContext, ActionKind},
            targeting::TargetingContext,
        },
        id::{ResourceId, SpellId},
        resource::{ResourceAmount, ResourceAmountMap},
    },
    engine::event::Event,
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
    school: MagicSchool,
    action: Action,
}

impl Spell {
    pub fn new(
        id: SpellId,
        description: String,
        base_level: u8,
        school: MagicSchool,
        kind: ActionKind,
        resource_cost: ResourceAmountMap,
        targeting: Arc<dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync>,
        reaction_trigger: Option<Arc<dyn Fn(Entity, &Event) -> bool + Send + Sync>>,
    ) -> Self {
        let action_id = id.clone().into();
        let mut resource_cost = resource_cost;
        if base_level > 0
            && !resource_cost.contains_key(&ResourceId::from_str("resource.spell_slot"))
        {
            // Ensure the spell has a spell slot cost if it's not a cantrip
            resource_cost.insert(
                ResourceId::from_str("resource.spell_slot"),
                ResourceAmount::Tiered {
                    tier: base_level,
                    amount: 1,
                },
            );
        }

        Self {
            id,
            school,
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
            if *resource == ResourceId::from_str("resource.spell_slot") {
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
