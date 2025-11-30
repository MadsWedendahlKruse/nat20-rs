use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{Action, ActionContext, ActionKind, ActionKindResult, ReactionResult},
            targeting::{EntityFilter, TargetingContext, TargetingKind},
        },
        damage::{AttackRoll, DamageRoll},
        dice::{DiceSet, DiceSetRoll, DieSize},
        id::{ActionId, ClassId, ResourceId},
        items::equipment::loadout::Loadout,
        level::CharacterLevels,
        modifier::{ModifierSet, ModifierSource},
        resource::{RechargeRule, ResourceAmount, ResourceAmountMap},
        saving_throw::SavingThrowSet,
    },
    engine::event::{Event, EventKind},
    registry,
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub static ACTION_REGISTRY: LazyLock<HashMap<ActionId, (Action, Option<ActionContext>)>> =
    LazyLock::new(|| {
        HashMap::from([
            (INDOMITABLE_ID.clone(), INDOMITABLE.to_owned()),
            (TACTICAL_MIND_ID.clone(), TACTICAL_MIND.to_owned()),
        ])
    });

pub static INDOMITABLE_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.indomitable"));

pub static INDOMITABLE: LazyLock<(Action, Option<ActionContext>)> = LazyLock::new(|| {
    (
        Action {
            id: INDOMITABLE_ID.clone(),
            description: "If you fail a saving throw, you can reroll it with a \
                bonus equal to your Fighter level. You must use the new roll."
                .to_string(),
            kind: ActionKind::Reaction {
                reaction: Arc::new(|game_state, reaction_data| {
                    let reactor = reaction_data.reactor;
                    let trigger_event = &reaction_data.event;
                    let reaction_context = &reaction_data.context;

                    let dc = match &trigger_event.kind {
                        EventKind::D20CheckPerformed(_, _, dc_kind) => match dc_kind {
                            D20CheckDCKind::SavingThrow(dc) => dc,
                            _ => panic!("Indomitable can only be triggered by a saving throw"),
                        },
                        _ => panic!("Invalid trigger event for Indomitable"),
                    };

                    let mut new_roll = systems::helpers::get_component::<SavingThrowSet>(
                        &game_state.world,
                        reactor,
                    )
                    .check_dc(dc, &game_state.world, reactor);

                    new_roll.modifier_breakdown.add_modifier(
                        ModifierSource::Action(INDOMITABLE_ID.clone()),
                        systems::class::class_level(
                            &game_state.world,
                            reactor,
                            &ClassId::from_str("class.fighter"),
                        ),
                    );

                    let _ = game_state.process_event(
                        Event::action_performed_event(
                            &game_state,
                            reactor,
                            &INDOMITABLE_ID.clone().into(),
                            &reaction_context,
                            &ResourceAmountMap::from([
                                (
                                    ResourceId::from_str("resource.fighter.indomitable"),
                                    ResourceAmount::Flat(1),
                                ),
                                (
                                    ResourceId::from_str("resource.reaction"),
                                    ResourceAmount::Flat(1),
                                ),
                            ]),
                            reactor,
                            ActionKindResult::Reaction {
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
                            }
                        ),
                    );
                }),
            },
            targeting: Arc::new(|_, _, _| TargetingContext::self_target()),
            resource_cost: ResourceAmountMap::from([
                (
                    ResourceId::from_str("resource.fighter.indomitable"),
                    ResourceAmount::Flat(1),
                ),
                (
                    ResourceId::from_str("resource.reaction"),
                    ResourceAmount::Flat(1),
                ),
            ]),
            cooldown: Some(RechargeRule::LongRest),
            reaction_trigger: Some(Arc::new(|reactor, event| match &event.kind {
                EventKind::D20CheckPerformed(performer, result, dc_kind) => {
                    performer == &reactor
                        && !result.is_success(dc_kind)
                        && matches!(result, D20ResultKind::SavingThrow { .. })
                }
                _ => false,
            })),
        },
        Some(ActionContext::Other),
    )
});

pub static TACTICAL_MIND_ID: LazyLock<ActionId> =
    LazyLock::new(|| ActionId::from_str("action.fighter.tactical_mind"));

static TACTICAL_MIND: LazyLock<(Action, Option<ActionContext>)> = LazyLock::new(|| {
    (
        Action {
            id: TACTICAL_MIND_ID.clone(),
            description: "You have a mind for tactics on and off the battlefield. \
                When you fail an ability check, you can expend \
                a use of your Second Wind to push yourself toward \
                success. Rather than regaining Hit Points, you roll \
                1d10 and add the number rolled to the ability check, \
                potentially turning it into a success. If the check still \
                fails, this use of Second Wind isn’t expended"
                .to_string(),
            kind: ActionKind::Reaction {
                reaction: Arc::new(|game_state, reaction_data| {
                    let _ = game_state.process_event(
                        Event::action_performed_event(
                            &game_state,
                            reaction_data.reactor,
                            &TACTICAL_MIND_ID.clone().into(),
                            &reaction_data.context,
                            &ResourceAmountMap::from([(
                                ResourceId::from_str("resource.fighter.second_wind"),
                                ResourceAmount::Flat(1),
                            )]),
                            reaction_data.reactor,
                            ActionKindResult::Reaction {
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
                                                    D20ResultKind::Skill { result, .. } => {
                                                        result.add_bonus(ModifierSource::Action(TACTICAL_MIND_ID.clone()), DiceSetRoll::from_str("1d10").unwrap().roll().subtotal);
                                                    }
                                                    _ => panic!("Tactical Mind modification applied to wrong result type"),

                                                }
                                            } else {
                                                panic!("Tactical Mind modification applied to wrong event type");
                                            }
                                        }
                                    }),
                                },
                            }
                        ),
                    );
                }),
            },
            targeting: Arc::new(|_, _, _| TargetingContext::self_target()),
            resource_cost: HashMap::from([(
                ResourceId::from_str("resource.fighter.second_wind"),
                ResourceAmount::Flat(1),
            )]),
            cooldown: None,
            reaction_trigger: Some(Arc::new(|reactor, event| match &event.kind {
                EventKind::D20CheckPerformed(performer, result, dc_kind) => {
                    performer == &reactor
                        && !result.is_success(dc_kind)
                        && matches!(result, D20ResultKind::Skill { .. })
                }
                _ => false,
            })),
        },
        Some(ActionContext::Other),
    )
});

pub static DEFAULT_RESOURCE_COST: LazyLock<HashMap<ResourceId, ResourceAmount>> =
    LazyLock::new(|| {
        HashMap::from([(
            ResourceId::from_str("resource.action").clone(),
            ResourceAmount::Flat(1),
        )])
    });
