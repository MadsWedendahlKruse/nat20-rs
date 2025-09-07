use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::{Action, ActionKind},
        ai::PlayerControlledTag,
        faction::Attitude,
        id::AIControllerId,
    },
    engine::{
        encounter::Encounter,
        event::{ActionDecision, ActionPrompt},
    },
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
