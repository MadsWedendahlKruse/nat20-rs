use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{Action, ActionContext, ActionKind},
            targeting::{TargetType, TargetingContext, TargetingKind},
        },
        class::ClassName,
        damage::{AttackRoll, DamageRoll},
        dice::{DiceSet, DiceSetRoll, DieSize},
        faction::Attitude,
        id::{ActionId, ResourceId},
        items::equipment::loadout::Loadout,
        level::CharacterLevels,
        modifier::{ModifierSet, ModifierSource},
        resource::RechargeRule,
    },
    registry, systems,
};

pub static ACTION_REGISTRY: LazyLock<HashMap<ActionId, (Action, Option<ActionContext>)>> =
    LazyLock::new(|| {
        HashMap::from([
            (ACTION_SURGE_ID.clone(), ACTION_SURGE.to_owned()),
            (SECOND_WIND_ID.clone(), SECOND_WIND.to_owned()),
            (WEAPON_ATTACK_ID.clone(), (WEAPON_ATTACK.to_owned(), None)),
        ])
    });

pub static ACTION_SURGE_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.action_surge"));

static ACTION_SURGE: LazyLock<(Action, Option<ActionContext>)> = LazyLock::new(|| {
    (
        Action {
            id: ACTION_SURGE_ID.clone(),
            kind: ActionKind::BeneficialEffect {
                effect: registry::effects::ACTION_SURGE_ID.clone(),
            },
            targeting: Arc::new(|_, _, _| TargetingContext::self_target()),
            resource_cost: HashMap::from([(registry::resources::ACTION_SURGE.clone(), 1)]),
            cooldown: Some(RechargeRule::OnTurn),
            reaction_trigger: None,
        },
        Some(ActionContext::Other),
    )
});

pub static SECOND_WIND_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.second_wind"));

static SECOND_WIND: LazyLock<(Action, Option<ActionContext>)> = LazyLock::new(|| {
    (
        Action {
            id: SECOND_WIND_ID.clone(),
            kind: ActionKind::Healing {
                heal: Arc::new(|world, entity, _| {
                    let mut modifiers = ModifierSet::new();
                    modifiers.add_modifier(
                        ModifierSource::ClassFeature("Fighter level".to_string()),
                        systems::helpers::get_component::<CharacterLevels>(world, entity)
                            .class_level(&ClassName::Fighter)
                            .unwrap()
                            .level() as i32,
                    );
                    DiceSetRoll::new(
                        DiceSet {
                            num_dice: 1,
                            die_size: DieSize::D10,
                        },
                        modifiers,
                        SECOND_WIND_ID.to_string(),
                    )
                }),
            },
            targeting: Arc::new(|_, _, _| TargetingContext::self_target()),
            resource_cost: HashMap::from([(registry::resources::BONUS_ACTION.clone(), 1)]),
            cooldown: None,
            reaction_trigger: None,
        },
        Some(ActionContext::Other),
    )
});

pub static WEAPON_ATTACK_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.weapon.attack"));

static WEAPON_ATTACK: LazyLock<Action> = LazyLock::new(|| Action {
    id: registry::actions::WEAPON_ATTACK_ID.clone(),
    kind: ActionKind::AttackRollDamage {
        attack_roll: WEAPON_ATTACK_ROLL.clone(),
        damage: WEAPON_DAMAGE_ROLL.clone(),
        damage_on_failure: None,
    },
    targeting: WEAPON_TARGETING.clone(),
    resource_cost: DEFAULT_RESOURCE_COST.clone(),
    cooldown: None,
    reaction_trigger: None,
});

// TODO: Some of this seems a bit circular?
static WEAPON_ATTACK_ROLL: LazyLock<
    Arc<dyn Fn(&World, Entity, &ActionContext) -> AttackRoll + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(
        |world: &World, entity: Entity, action_context: &ActionContext| {
            if let ActionContext::Weapon { slot } = action_context {
                return systems::combat::attack_roll(world, entity, slot);
            }
            panic!("Action context must be Weapon");
        },
    )
});

static WEAPON_DAMAGE_ROLL: LazyLock<
    Arc<dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(
        |world: &World, entity: Entity, action_context: &ActionContext| {
            if let ActionContext::Weapon { slot } = action_context {
                return systems::combat::damage_roll(world, entity, slot);
            }
            panic!("Action context must be Weapon");
        },
    )
});

static WEAPON_TARGETING: LazyLock<
    Arc<dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync>,
> = LazyLock::new(|| {
    Arc::new(
        |world: &World, entity: Entity, action_context: &ActionContext| {
            if let ActionContext::Weapon { slot } = action_context {
                let (normal_range, max_range) =
                    systems::helpers::get_component::<Loadout>(world, entity)
                        .weapon_in_hand(slot)
                        .unwrap()
                        .range();
                TargetingContext {
                    kind: TargetingKind::Single,
                    normal_range,
                    max_range,
                    valid_target_types: vec![TargetType::entity_not_dead()],
                }
            } else {
                panic!("Action context must be Weapon");
            }
        },
    )
});

static DEFAULT_RESOURCE_COST: LazyLock<HashMap<ResourceId, u8>> =
    LazyLock::new(|| HashMap::from([(registry::resources::ACTION.clone(), 1)]));
