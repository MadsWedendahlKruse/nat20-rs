use hecs::{Entity, World};

use crate::{
    components::{ai::PlayerControlledTag, id::AIControllerId},
    engine::encounter::{ActionDecision, ActionPrompt, Encounter},
    registry, systems,
};

pub fn is_player_controlled(world: &World, entity: Entity) -> bool {
    world.get::<&PlayerControlledTag>(entity).is_ok()
}

pub fn decide_action(
    world: &World,
    encounter: &Encounter,
    prompt: &ActionPrompt,
    actor: Entity,
) -> Option<ActionDecision> {
    let controller_id = systems::helpers::get_component::<AIControllerId>(world, actor);

    registry::ai::AI_CONTROLLER_REGISTRY
        .get(&controller_id)
        .and_then(|controller| controller.decide(world, encounter, prompt, actor))
}
