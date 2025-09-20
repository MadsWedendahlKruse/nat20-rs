use std::{f32::consts::E, ops::Deref, sync::Arc};

use hecs::{Entity, World};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::{
            action::{ActionContext, ActionKind, ActionKindResult, ActionResult},
            targeting::TargetTypeInstance,
        },
        damage::{
            AttackRollResult, DamageMitigationEffect, DamageMitigationResult, DamageResistances,
            DamageRollResult, MitigationOperation,
        },
        health::{hit_points::HitPoints, life_state::LifeState},
        id::{ActionId, EntityIdentifier},
        level::CharacterLevels,
        modifier::ModifierSource,
        resource::{self, ResourceAmountMap},
        saving_throw::{self, SavingThrowKind, SavingThrowSet},
    },
    engine::{
        event::{
            ActionData, CallbackResult, Event, EventCallback, EventId, EventKind, EventListener,
        },
        game_state::{self, GameState},
    },
    entities::{character::CharacterTag, monster::MonsterTag},
    registry,
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub fn heal(world: &mut World, target: Entity, amount: u32) -> Option<LifeState> {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(target) {
        hit_points.heal(amount);
        if let Ok(mut life_state) = world.get::<&mut LifeState>(target) {
            if hit_points.current() > 0 {
                *life_state = LifeState::Normal;
                return Some(LifeState::Normal);
            }
        }
    }
    None
}

pub fn heal_full(world: &mut World, target: Entity) -> Option<LifeState> {
    // TODO: Bit of a convoluted way to get avoid repeating the life state logic
    let hit_point_max = if let Ok(hit_points) = world.get::<&HitPoints>(target) {
        Some(hit_points.max())
    } else {
        None
    };
    if let Some(max) = hit_point_max {
        return heal(world, target, max);
    }
    None
}

fn damage_internal(
    world: &mut World,
    target: Entity,
    damage_roll_result: &DamageRollResult,
    attack_roll: Option<&AttackRollResult>,
    resistances: &DamageResistances,
) -> (Option<DamageMitigationResult>, Option<LifeState>) {
    // TODO: For something like Shield, which is triggered *IF the target is hit*,
    // we need to simulate the outcome of the action, and then have the reaction
    // be triggered by the outcome of this simulation. And then we need to actually
    // apply the simulated damage after the reaction is resolved.
    // To simulate damage I guess we can just clone the HitPoints and LifeState
    // components, and then apply the damage to the clones?
    // TODO: New event type ActionSimulated?

    let (mitigation_result, killed_by_damage, mut new_life_state) =
        if let Ok((hit_points, life_state)) =
            world.query_one_mut::<(&mut HitPoints, &mut LifeState)>(target)
        {
            // Track any changes to the life state of the target
            let mut new_life_state = None;
            // Check if the target is already at 0 HP
            let hp_before_damage = hit_points.current();
            if hit_points.current() == 0 {
                match life_state {
                    LifeState::Stable => {
                        new_life_state = Some(LifeState::unconscious());
                    }

                    LifeState::Unconscious(death_saving_throws) => {
                        if let Some(attack_roll) = attack_roll {
                            if attack_roll.roll_result.is_crit {
                                death_saving_throws.record_failure(2);
                            } else {
                                death_saving_throws.record_failure(1);
                            }
                        } else {
                            death_saving_throws.record_failure(1);
                        }

                        let next_state = death_saving_throws.next_state();
                        if !matches!(next_state, LifeState::Unconscious(_)) {
                            // If the next state is not still unconscious, we need to update it
                            new_life_state = Some(next_state);
                        }
                    }

                    _ => {
                        // Other valid states where HP would be zero are some form of
                        // dead, so no-op
                        // TODO: Validate that this is the case?
                    }
                }
            }

            let mitigation_result = resistances.apply(damage_roll_result);
            hit_points.damage(mitigation_result.total.max(0) as u32);

            (
                mitigation_result,
                hp_before_damage > 0 && hit_points.current() == 0,
                new_life_state,
            )
        } else {
            return (None, None);
        };

    if killed_by_damage {
        // Monsters and Characters 'die' differently
        if let Ok(_) = world.get::<&MonsterTag>(target) {
            new_life_state = Some(LifeState::Dead);
        }

        if let Ok(_) = world.get::<&CharacterTag>(target) {
            new_life_state = Some(LifeState::unconscious());
        }
    }

    if let Some(new_life_state) = new_life_state {
        if let Ok(mut life_state) = world.get::<&mut LifeState>(target) {
            *life_state = new_life_state;
        }
    }

    (Some(mitigation_result), new_life_state)
}

pub fn damage(
    game_state: &mut GameState,
    performer: Entity,
    target: Entity,
    action_id: &ActionId,
    action_kind: &ActionKind,
    context: &ActionContext,
    resource_cost: ResourceAmountMap,
) {
    let resistances =
        if let Ok(resistances) = &game_state.world.get::<&mut DamageResistances>(target) {
            resistances.deref().clone()
        } else {
            DamageResistances::new()
        };

    let (event, callback) = match action_kind {
        ActionKind::UnconditionalDamage { damage } => {
            // Create the damage roll event
            let damage_roll = damage(&game_state.world, performer, context).roll();
            let event = Event::new(EventKind::DamageRollPerformed(
                performer,
                damage_roll.clone(),
            ));

            // Create a callback to handle the result of the damage roll
            let callback: EventCallback = Arc::new({
                let action_id = action_id.clone();
                let context = context.clone();
                let resource_cost = resource_cost.clone();
                let target = target;
                let resistances = resistances.clone();
                move |game_state, event| match &event.kind {
                    EventKind::DamageRollResolved(performer, damage_roll_result) => {
                        let (damage_taken, new_life_state) = damage_internal(
                            &mut game_state.world,
                            target,
                            damage_roll_result,
                            None,
                            &resistances,
                        );

                        CallbackResult::Event(Event::new(EventKind::ActionPerformed {
                            action: ActionData {
                                actor: *performer,
                                action_id: action_id.clone(),
                                context: context.clone(),
                                resource_cost: resource_cost.clone(),
                                targets: vec![target],
                            },
                            results: vec![ActionResult {
                                performer: EntityIdentifier::from_world(
                                    &game_state.world,
                                    *performer,
                                ),
                                target: TargetTypeInstance::Entity(EntityIdentifier::from_world(
                                    &game_state.world,
                                    target,
                                )),
                                kind: ActionKindResult::UnconditionalDamage {
                                    damage_roll: damage_roll.clone(),
                                    damage_taken,
                                    new_life_state,
                                },
                            }],
                        }))
                    }
                    _ => {
                        panic!("Unexpected event kind in damage roll callback: {:?}", event);
                    }
                }
            });

            (event, callback)
        }

        ActionKind::AttackRollDamage {
            attack_roll,
            damage,
            damage_on_miss,
        } => {
            // TODO: The attack roll obtained here doesn't actually correctly reflect
            // whether the attack hits or not, because it doesn't take into account
            // the target's AC
            let attack_roll = attack_roll(&game_state.world, performer, context)
                .roll(&game_state.world, performer);
            let armor_class = systems::loadout::armor_class(&game_state.world, target);
            let ActionContext::Weapon { slot } = context else {
                panic!("AttackRollDamage action must be used with a weapon");
            };

            // Create an event to represent the attack roll being made
            let event = Event::new(EventKind::D20CheckPerformed(
                performer,
                D20ResultKind::AttackRoll {
                    result: attack_roll.clone(),
                },
                Some(D20CheckDCKind::AttackRoll(
                    *slot,
                    target,
                    armor_class.clone(),
                )),
            ));

            // Create a callback to handle the result of the attack roll
            // This callback will handle applying damage based on whether the attack
            // hits or misses, and will also handle critical hits
            let callback: EventCallback = Arc::new({
                let action_id = action_id.clone();
                let context = context.clone();
                let resource_cost = resource_cost.clone();
                let resistances = resistances.clone();

                let damage = damage.clone();
                let damage_on_miss = damage_on_miss.clone();

                move |game_state, event| match &event.kind {
                    EventKind::D20CheckResolved(performer, result, dc) => {
                        // Determine the damage to apply based on whether the attack hits or misses
                        let (damage_roll, is_crit) = if result.is_success(dc.as_ref().unwrap()) {
                            (
                                damage(&game_state.world, *performer, &context),
                                result.d20_result().is_crit,
                            )
                        } else if let Some(damage_on_miss) = &damage_on_miss {
                            (
                                damage_on_miss(&game_state.world, *performer, &context),
                                false,
                            )
                        } else {
                            // If the attack misses and there's no damage on miss, no damage is dealt
                            return CallbackResult::Event(Event::new(EventKind::ActionPerformed {
                                action: ActionData {
                                    actor: *performer,
                                    action_id: action_id.clone(),
                                    context: context.clone(),
                                    resource_cost: resource_cost.clone(),
                                    targets: vec![target],
                                },
                                results: vec![ActionResult {
                                    performer: EntityIdentifier::from_world(
                                        &game_state.world,
                                        *performer,
                                    ),
                                    target: TargetTypeInstance::Entity(
                                        EntityIdentifier::from_world(&game_state.world, target),
                                    ),
                                    kind: ActionKindResult::AttackRollDamage {
                                        attack_roll: attack_roll.clone(),
                                        armor_class: armor_class.clone(),
                                        damage_roll: None,
                                        damage_taken: None,
                                        new_life_state: None,
                                    },
                                }],
                            }));
                        };

                        // Create the damage roll event
                        let damage_roll = damage_roll.roll_crit_damage(is_crit);
                        let event = Event::new(EventKind::DamageRollPerformed(
                            *performer,
                            damage_roll.clone(),
                        ));

                        // Create an event listener to handle the result of the damage roll
                        return CallbackResult::EventWithCallback(
                            event,
                            Arc::new({
                                let action_id = action_id.clone();
                                let context = context.clone();
                                let resource_cost = resource_cost.clone();
                                let target = target;
                                let resistances = resistances.clone();
                                let attack_roll = attack_roll.clone();
                                let armor_class = armor_class.clone();
                                move |game_state, event| match &event.kind {
                                    EventKind::DamageRollResolved(
                                        performer,
                                        damage_roll_result,
                                    ) => {
                                        let (damage_taken, new_life_state) = damage_internal(
                                            &mut game_state.world,
                                            target,
                                            damage_roll_result,
                                            Some(&attack_roll),
                                            &resistances,
                                        );

                                        return CallbackResult::Event(Event::new(
                                            EventKind::ActionPerformed {
                                                action: ActionData {
                                                    actor: *performer,
                                                    action_id: action_id.clone(),
                                                    context: context.clone(),
                                                    resource_cost: resource_cost.clone(),
                                                    targets: vec![target],
                                                },
                                                results: vec![ActionResult {
                                                    performer: EntityIdentifier::from_world(
                                                        &game_state.world,
                                                        *performer,
                                                    ),
                                                    target: TargetTypeInstance::Entity(
                                                        EntityIdentifier::from_world(
                                                            &game_state.world,
                                                            target,
                                                        ),
                                                    ),
                                                    kind: ActionKindResult::AttackRollDamage {
                                                        attack_roll: attack_roll.clone(),
                                                        armor_class: armor_class.clone(),
                                                        damage_roll: Some(
                                                            damage_roll_result.clone(),
                                                        ),
                                                        damage_taken,
                                                        new_life_state,
                                                    },
                                                }],
                                            },
                                        ));
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected event kind in damage roll callback: {:?}",
                                            event
                                        );
                                    }
                                }
                            }),
                        );
                    }

                    _ => {
                        panic!("Unexpected event kind in attack roll callback: {:?}", event);
                    }
                }
            });

            (event, callback)
        }

        ActionKind::SavingThrowDamage {
            saving_throw,
            half_damage_on_save,
            damage,
        } => {
            let saving_throw_dc = saving_throw(&game_state.world, performer, context);

            let damage_roll = damage(&game_state.world, performer, context).roll();
            let event = Event::new(EventKind::DamageRollPerformed(
                performer,
                damage_roll.clone(),
            ));

            // Create a callback listener to handle the result of the damage roll
            let callback: EventCallback = Arc::new({
                let action_id = action_id.clone();
                let context = context.clone();
                let target = target;
                let resistances = resistances.clone();
                let saving_throw_dc = saving_throw_dc.clone();
                let half_damage_on_save = *half_damage_on_save;
                move |game_state, event| match &event.kind {
                    EventKind::DamageRollResolved(performer, damage_roll_result) => {
                        // Create an event to represent the saving throw being made
                        let saving_throw_event = systems::d20::check(
                            game_state,
                            *performer,
                            &D20CheckDCKind::SavingThrow(saving_throw_dc.clone()),
                        );

                        // Create an event listener to handle the result of the saving throw
                        return CallbackResult::EventWithCallback(
                            saving_throw_event,
                            Arc::new({
                                let action_id = action_id.clone();
                                let context = context.clone();
                                let resource_cost = resource_cost.clone();
                                let target = target;
                                let resistances = resistances.clone();
                                let saving_throw_dc = saving_throw_dc.clone();
                                let damage_roll_result = damage_roll_result.clone();
                                let half_damage_on_save = half_damage_on_save;
                                move |game_state, event| match &event.kind {
                                    EventKind::D20CheckResolved(performer, result, dc) => {
                                        let mut resistances = resistances.clone();
                                        if result.is_success(dc.as_ref().unwrap())
                                            && half_damage_on_save
                                        {
                                            // Apply half damage on successful save

                                            let ability = match saving_throw_dc.key {
                                                SavingThrowKind::Ability(ability) => ability,
                                                SavingThrowKind::Death => Ability::Constitution,
                                            };
                                            for component in damage_roll_result.components.iter() {
                                                resistances.add_effect(
                                                    component.damage_type,
                                                    DamageMitigationEffect {
                                                        // TODO: Not sure if this is the best source
                                                        source: ModifierSource::Ability(ability),
                                                        operation: MitigationOperation::Resistance,
                                                    },
                                                );
                                            }
                                        }

                                        let (damage_taken, new_life_state) = damage_internal(
                                            &mut game_state.world,
                                            target,
                                            &damage_roll_result,
                                            None,
                                            &resistances,
                                        );

                                        return CallbackResult::Event(Event::new(
                                            EventKind::ActionPerformed {
                                                action: ActionData {
                                                    actor: *performer,
                                                    action_id: action_id.clone(),
                                                    context: context.clone(),
                                                    resource_cost: resource_cost.clone(),
                                                    targets: vec![target],
                                                },
                                                results: vec![ActionResult {
                                                    performer: EntityIdentifier::from_world(
                                                        &game_state.world,
                                                        *performer,
                                                    ),
                                                    target: TargetTypeInstance::Entity(
                                                        EntityIdentifier::from_world(
                                                            &game_state.world,
                                                            target,
                                                        ),
                                                    ),
                                                    kind: ActionKindResult::SavingThrowDamage {
                                                        saving_throw_dc: saving_throw_dc.clone(),
                                                        saving_throw_result: result
                                                            .d20_result()
                                                            .clone(),
                                                        half_damage_on_save,
                                                        damage_roll: damage_roll_result.clone(),
                                                        damage_taken,
                                                        new_life_state,
                                                    },
                                                }],
                                            },
                                        ));
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected event kind in saving throw callback: {:?}",
                                            event
                                        );
                                    }
                                }
                            }),
                        );
                    }
                    _ => {
                        panic!("Unexpected event kind in damage roll callback: {:?}", event);
                    }
                }
            });

            (event, callback)
        }

        _ => {
            panic!(
                "systems::health::damage called with unsupported action kind: {:?}",
                action_kind
            );
        }
    };

    game_state.process_event_with_callback(event, callback);
}

pub fn is_alive(world: &World, entity: Entity) -> bool {
    if let Ok(hit_points) = world.get::<&HitPoints>(entity) {
        hit_points.current() > 0
    } else {
        false
    }
}

pub fn update_hit_points(world: &mut World, entity: Entity) {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(entity) {
        if let Ok(class_levels) = world.get::<&CharacterLevels>(entity) {
            let mut new_hp = 0;

            let total_level = class_levels.total_level();

            if total_level == 0 {
                // TODO: Not sure if this ever happens
                return;
            }

            // Calculate the hit points based on the class levels and the
            // Constitution modifier. Hit points are calculated on a per-level
            // basis. Hit points are calculated as follows:
            // - For level 1, the hit points are the hit die of the first class
            //   + Constitution modifier
            // - For subsequent levels, the hit points are increased by a fixed
            //   amount based on the class + Constitution modifier.

            let constitution_modifier =
                systems::helpers::get_component::<AbilityScoreMap>(world, entity)
                    .get(Ability::Constitution)
                    .ability_modifier()
                    .total();

            for (class_name, class_level) in class_levels.all_classes() {
                if let Some(class) = registry::classes::CLASS_REGISTRY.get(class_name) {
                    for level in 1..=class_level.level() {
                        let hp_increase =
                            if class_name == class_levels.first_class().unwrap() && level == 1 {
                                class.hit_die as u32
                            } else {
                                class.hp_per_level as u32
                            };

                        new_hp += (hp_increase + (constitution_modifier as u32)).max(1);
                    }
                }
            }

            hit_points.update_max(new_hp);
        }
    }
}
