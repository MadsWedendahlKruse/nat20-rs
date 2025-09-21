use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{
                Action, ActionContext, ActionKind, ActionKindResult, ActionResult, ReactionResult,
            },
            targeting::{TargetType, TargetTypeInstance, TargetingContext, TargetingKind},
        },
        class::ClassName,
        damage::{AttackRoll, DamageRoll},
        dice::{DiceSet, DiceSetRoll, DieSize},
        id::{ActionId, EntityIdentifier, ResourceId},
        items::equipment::loadout::Loadout,
        level::CharacterLevels,
        modifier::{ModifierSet, ModifierSource},
        resource::{RechargeRule, ResourceAmount},
        saving_throw::SavingThrowSet,
    },
    engine::event::{ActionData, Event, EventKind},
    registry,
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub static ACTION_REGISTRY: LazyLock<HashMap<ActionId, (Action, Option<ActionContext>)>> =
    LazyLock::new(|| {
        HashMap::from([
            (ACTION_SURGE_ID.clone(), ACTION_SURGE.to_owned()),
            (INDOMITABLE_ID.clone(), INDOMITABLE.to_owned()),
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
            resource_cost: HashMap::from([(
                registry::resources::ACTION_SURGE_ID.clone(),
                registry::resources::ACTION_SURGE.build_amount(1),
            )]),
            cooldown: Some(RechargeRule::Turn),
            reaction_trigger: None,
        },
        Some(ActionContext::Other),
    )
});

pub static INDOMITABLE_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.indomitable"));

pub static INDOMITABLE: LazyLock<(Action, Option<ActionContext>)> = LazyLock::new(|| {
    (
        Action {
            id: INDOMITABLE_ID.clone(),
            kind: ActionKind::Reaction {
                reaction: Arc::new(|game_state, reactor, trigger_event, reaction_context| {
                    let dc = match &trigger_event.kind {
                        EventKind::D20CheckPerformed(_, _, dc_kind) => {
                            match dc_kind.as_ref().unwrap() {
                                D20CheckDCKind::SavingThrow(dc) => dc,
                                _ => panic!("Indomitable can only be triggered by a saving throw"),
                            }
                        }
                        _ => panic!("Invalid trigger event for Indomitable"),
                    };

                    let mut new_roll = systems::helpers::get_component::<SavingThrowSet>(
                        &game_state.world,
                        reactor,
                    )
                    .check_dc(dc, &game_state.world, reactor);

                    new_roll.modifier_breakdown.add_modifier(
                        ModifierSource::ClassFeature("fighter.indomitable".to_string()),
                        systems::class::class_level(
                            &game_state.world,
                            reactor,
                            &ClassName::Fighter,
                        ),
                    );

                    let _ = game_state.process_event(Event::new(EventKind::ActionPerformed {
                        action: ActionData {
                            actor: reactor,
                            action_id: INDOMITABLE_ID.clone(),
                            context: reaction_context.clone(),
                            resource_cost: HashMap::from([(
                                registry::resources::INDOMITABLE_ID.clone(),
                                registry::resources::INDOMITABLE.build_amount(1),
                            )]),
                            targets: vec![reactor],
                        },
                        results: vec![ActionResult {
                            performer: EntityIdentifier::from_world(&game_state.world, reactor),
                            target: TargetTypeInstance::Entity(EntityIdentifier::from_world(
                                &game_state.world,
                                reactor,
                            )),
                            kind: ActionKindResult::Reaction {
                                result: ReactionResult::ModifyEvent {
                                    modification: Arc::new({
                                        move |event: &mut Event| {
                                            if let EventKind::D20CheckPerformed(
                                                _,
                                                ref mut existing_result,
                                                _,
                                            ) = event.kind
                                            {
                                                match existing_result {
                                                    D20ResultKind::SavingThrow { result, .. } => {
                                                        *result = new_roll.clone();
                                                    }
                                                    _ => panic!("Indomitable modification applied to wrong result type"),
                                                    
                                                }
                                            } else {
                                                panic!("Indomitable modification applied to wrong event type");
                                            }
                                        }
                                    }),
                                },
                            },
                        }],
                    }));
                }),
            },
            targeting: Arc::new(|_, _, _| TargetingContext::self_target()),
            resource_cost: HashMap::from([
                (
                    registry::resources::INDOMITABLE_ID.clone(),
                    registry::resources::INDOMITABLE.build_amount(1),
                ),
                (
                    registry::resources::REACTION_ID.clone(),
                    registry::resources::REACTION.build_amount(1),
                )
            ]),
            cooldown: Some(RechargeRule::LongRest),
            reaction_trigger: Some(Arc::new(|reactor, event| match &event.kind {
                EventKind::D20CheckPerformed(performer, result, dc_kind) => {
                    performer == &reactor
                        && !result.is_success(dc_kind.as_ref().unwrap())
                        && matches!(result, D20ResultKind::SavingThrow { .. })
                }
                _ => false,
            })),
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
            resource_cost: HashMap::from([(
                registry::resources::BONUS_ACTION_ID.clone(),
                registry::resources::BONUS_ACTION.build_amount(1),
            )]),
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
        damage_on_miss: None,
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

pub static DEFAULT_RESOURCE_COST: LazyLock<HashMap<ResourceId, ResourceAmount>> =
    LazyLock::new(|| {
        HashMap::from([(
            registry::resources::ACTION_ID.clone(),
            registry::resources::ACTION.build_amount(1),
        )])
    });
