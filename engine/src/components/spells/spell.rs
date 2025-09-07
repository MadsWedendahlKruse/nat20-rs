use std::{collections::HashMap, hash::Hash, sync::Arc};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{Action, ActionContext, ActionKind, ReactionResult},
            targeting::TargetingContext,
        },
        id::{ActionId, ResourceId, SpellId},
    },
    engine::{
        event::{Event, EventKind},
        game_state::GameState,
    },
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
        targeting: Arc<dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync>,
        reaction_trigger: Option<
            Arc<dyn Fn(Entity, &Event, &ActionContext) -> Option<ReactionResult> + Send + Sync>,
        >,
    ) -> Self {
        let action_id = id.to_action_id();
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
        self.base_level == 0
    }

    pub fn school(&self) -> MagicSchool {
        self.school
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    // TODO: Apparently not used anywhere?
    // pub fn snapshot(
    //     &self,
    //     world: &World,
    //     caster: Entity,
    //     spell_level: &u8,
    // ) -> Result<ActionKindSnapshot, SnapshotError> {
    //     if spell_level < &self.base_level {
    //         return Err(SnapshotError::DowncastingNotAllowed(
    //             self.base_level,
    //             *spell_level,
    //         ));
    //     }
    //     if self.is_cantrip() && spell_level > &self.base_level {
    //         return Err(SnapshotError::UpcastingCantripNotAllowed);
    //     }
    //     // TODO: Something like BG3 Lightning Charges with Magic Missile would not work
    //     // with this snapshotting, since each damage instance would add an effect to the
    //     // caster, which would not be reflected in the snapshot.
    //     // ---
    //     // Might not be an issue anymore???
    //     Ok(self.action.kind().snapshot(
    //         world,
    //         caster,
    //         &ActionContext::Spell {
    //             level: *spell_level,
    //         },
    //     ))
    // }
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
