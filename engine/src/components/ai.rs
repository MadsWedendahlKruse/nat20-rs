use hecs::{Entity, World};

use crate::engine::{
    encounter::Encounter,
    event::{ActionDecision, ActionPrompt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerControlledTag;

pub trait AIController: Send + Sync + 'static {
    fn decide(
        &self,
        world: &World,
        encounter: &Encounter,
        prompt: &ActionPrompt,
        actor: Entity,
    ) -> Option<ActionDecision>;
}
