use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionKind,
        ai::{AIDecision, PlayerControlledTag},
        faction::Attitude,
        id::AIControllerId,
    },
    engine::{event::ActionPrompt, game_state::GameState},
    registry, systems,
};

pub fn is_player_controlled(world: &World, entity: Entity) -> bool {
    world.get::<&PlayerControlledTag>(entity).is_ok()
}

pub fn decide_action(
    game_state: &mut GameState,
    prompt: &ActionPrompt,
    actor: Entity,
) -> AIDecision {
    let controller_id =
        systems::helpers::get_component_clone::<AIControllerId>(&game_state.world, actor);

    registry::ai::AI_CONTROLLER_REGISTRY
        .get(&controller_id)
        .unwrap()
        .decide(game_state, prompt, actor)
}

pub fn recommeneded_target_attitude(
    world: &World,
    actor: Entity,
    action_kind: &ActionKind,
) -> Attitude {
    match action_kind {
        ActionKind::AttackRollDamage { .. }
            | ActionKind::UnconditionalDamage { .. }
            | ActionKind::SavingThrowDamage { .. }
            | ActionKind::SavingThrowEffect { .. }
            // TODO: What to do with UnconditionalEffect?
            | ActionKind::UnconditionalEffect { .. } => Attitude::Hostile,

        ActionKind::Healing { .. } | ActionKind::BeneficialEffect { .. } => Attitude::Friendly,

        ActionKind::Composite { actions } => {
                // TODO: Hopefully there's never a mix of friendly and hostile sub-actions?
                // If any sub-action is hostile, be hostile; else friendly
                let mut best = Attitude::Friendly;
                for sub_action in actions {
                    let attitude = recommeneded_target_attitude(world, actor, sub_action);
                    best = best.max(attitude);
                    if best == Attitude::Hostile {
                        break;
                    }
                }
                best
            }

        ActionKind::Utility {  } => todo!(),

        ActionKind::Custom(_) => todo!(),

        ActionKind::Reaction { .. } => todo!(),
    }
}
