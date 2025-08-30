use hecs::{Entity, World};

use crate::engine::encounter::{ActionDecision, ActionPrompt, Encounter};

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
