use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    actions::{
        action::{Action, ActionContext, ActionKind},
        targeting::{TargetType, TargetingContext, TargetingKind},
    },
    combat::damage::{AttackRoll, DamageRoll},
    creature::character::Character,
    registry,
    resources::resources::RechargeRule,
    utils::id::{ActionId, ResourceId},
};

pub static ACTION_REGISTRY: LazyLock<HashMap<ActionId, (Action, ActionContext)>> =
    LazyLock::new(|| {
        HashMap::from([
            (ACTION_SURGE_ID.clone(), ACTION_SURGE.clone()),
            (
                WEAPON_MELEE_ATTACK_ID.clone(),
                // TODO: What to do with actions that don't have a fixed context?
                (WEAPON_MELEE_ATTACK.clone(), ActionContext::Other),
            ),
        ])
    });

pub static ACTION_SURGE_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.action_surge"));

static ACTION_SURGE: LazyLock<(Action, ActionContext)> = LazyLock::new(|| {
    (
        Action {
            id: ACTION_SURGE_ID.clone(),
            kind: ActionKind::BeneficialEffect {
                effect: registry::effects::ACTION_SURGE_ID.clone(),
            },
            targeting: Arc::new(|_, _| TargetingContext::self_target()),
            resource_cost: HashMap::from([(registry::resources::ACTION_SURGE.clone(), 1)]),
            cooldown: Some(RechargeRule::OnTurn),
        },
        ActionContext::Other,
    )
});

pub static WEAPON_MELEE_ATTACK_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.weapon.melee_attack"));

static WEAPON_MELEE_ATTACK: LazyLock<Action> = LazyLock::new(|| Action {
    id: registry::actions::WEAPON_MELEE_ATTACK_ID.clone(),
    kind: ActionKind::AttackRollDamage {
        attack_roll: WEAPON_ATTACK_ROLL.clone(),
        damage: WEAPON_DAMAGE_ROLL.clone(),
        damage_on_failure: None,
    },
    targeting: WEAPON_TARGETING.clone(),
    resource_cost: DEFAULT_RESOURCE_COST.clone(),
    cooldown: None,
});

// TODO: Implement
// TODO: What's the actual difference between this and melee attack?
pub static WEAPON_RANGED_ATTACK_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.weapon.ranged_attack"));

static WEAPON_RANGED_ATTACK: LazyLock<Action> = LazyLock::new(|| Action {
    id: registry::actions::WEAPON_RANGED_ATTACK_ID.clone(),
    kind: ActionKind::AttackRollDamage {
        attack_roll: WEAPON_ATTACK_ROLL.clone(),
        damage: WEAPON_DAMAGE_ROLL.clone(),
        damage_on_failure: None,
    },
    targeting: WEAPON_TARGETING.clone(),
    resource_cost: DEFAULT_RESOURCE_COST.clone(),
    cooldown: None,
});

// TODO: Some of this seems a bit circular?
static WEAPON_ATTACK_ROLL: LazyLock<
    Arc<dyn Fn(&Character, &ActionContext) -> AttackRoll + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(|character: &Character, action_context: &ActionContext| {
        let (weapon_type, hand) = match action_context {
            ActionContext::Weapon { weapon_type, hand } => (weapon_type, hand),
            _ => panic!("Action context must be Weapon"),
        };
        character
            .loadout()
            .weapon_in_hand(weapon_type, hand)
            .unwrap()
            .attack_roll(character)
    })
});

static WEAPON_DAMAGE_ROLL: LazyLock<
    Arc<dyn Fn(&Character, &ActionContext) -> DamageRoll + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(|character: &Character, action_context: &ActionContext| {
        let (weapon_type, hand) = match action_context {
            ActionContext::Weapon { weapon_type, hand } => (weapon_type, hand),
            _ => panic!("Action context must be Weapon"),
        };
        character
            .loadout()
            .weapon_in_hand(weapon_type, hand)
            .unwrap()
            .damage_roll(character, hand)
    })
});

static WEAPON_TARGETING: LazyLock<
    Arc<dyn Fn(&Character, &ActionContext) -> TargetingContext + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(|character: &Character, action_context: &ActionContext| {
        let (weapon_type, hand) = match action_context {
            ActionContext::Weapon { weapon_type, hand } => (weapon_type, hand),
            _ => panic!("Action context must be Weapon"),
        };
        let (normal_range, max_range) = character
            .loadout()
            .weapon_in_hand(weapon_type, hand)
            .unwrap()
            .range();

        TargetingContext {
            kind: TargetingKind::Single,
            normal_range,
            max_range,
            valid_target_types: vec![TargetType::Character],
        }
    })
});

static DEFAULT_RESOURCE_COST: LazyLock<HashMap<ResourceId, u8>> =
    LazyLock::new(|| HashMap::from([(registry::resources::ACTION.clone(), 1)]));
