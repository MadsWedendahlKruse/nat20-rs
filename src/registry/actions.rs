use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    actions::{
        action::{Action, ActionContext, ActionKind},
        targeting::TargetingContext,
    },
    registry,
    resources::resources::RechargeRule,
    utils::id::ActionId,
};

pub static ACTION_REGISTRY: LazyLock<HashMap<ActionId, (Action, ActionContext)>> =
    LazyLock::new(|| HashMap::from([(ACTION_SURGE_ID.clone(), ACTION_SURGE.clone())]));

pub static ACTION_SURGE_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.action_surge"));

pub static ACTION_SURGE: LazyLock<(Action, ActionContext)> = LazyLock::new(|| {
    let action = Action {
        id: ACTION_SURGE_ID.clone(),
        kind: ActionKind::BeneficialEffect {
            effect: registry::effects::ACTION_SURGE_ID.clone(),
        },
        targeting: Arc::new(|_, _| TargetingContext::self_target()),
        resource_cost: HashMap::from([(registry::resources::ACTION_SURGE.clone(), 1)]),
        cooldown: Some(RechargeRule::OnTurn),
    };
    let context = ActionContext::Other;
    (action, context)
});

pub static WEAPON_MELEE_ATTACK_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.weapon.melee_attack"));
