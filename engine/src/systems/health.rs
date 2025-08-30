use hecs::{Entity, World};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::action::{ActionKindResult, ActionKindSnapshot},
        damage::{
            AttackRollResult, DamageMitigationEffect, DamageMitigationResult, DamageResistances,
            DamageRollResult, MitigationOperation,
        },
        health::{hit_points::HitPoints, life_state::LifeState},
        level::CharacterLevels,
        modifier::ModifierSource,
        saving_throw::{SavingThrowKind, SavingThrowSet},
    },
    entities::{character::CharacterTag, monster::MonsterTag},
    registry, systems,
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
            new_life_state = Some(LifeState::Defeated);
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

// TODO: This should return some more information, like for an attack roll
// what was the armor class it rolled against, or for a saving throw,
// what did the target roll, etc.
pub fn damage(
    world: &mut World,
    target: Entity,
    // TODO: Attacker?
    damage_source: &ActionKindSnapshot,
) -> ActionKindResult {
    let mut resistances = if let Ok(resistances) = world.get::<&mut DamageResistances>(target) {
        resistances.clone()
    } else {
        DamageResistances::new()
    };

    match damage_source {
        ActionKindSnapshot::UnconditionalDamage { damage_roll } => {
            let (damage_taken, new_life_state) =
                damage_internal(world, target, damage_roll, None, &resistances);
            ActionKindResult::UnconditionalDamage {
                damage_roll: damage_roll.clone(),
                damage_taken: damage_taken,
                new_life_state,
            }
        }

        ActionKindSnapshot::AttackRollDamage {
            attack_roll,
            damage_roll,
            damage_on_failure,
        } => {
            let (damage_taken, new_life_state) =
                if !systems::combat::attack_hits(world, target, attack_roll) {
                    if let Some(damage_on_failure) = damage_on_failure {
                        damage_internal(
                            world,
                            target,
                            damage_on_failure,
                            Some(attack_roll),
                            &resistances,
                        )
                    } else {
                        (None, None)
                    }
                } else {
                    damage_internal(world, target, damage_roll, Some(attack_roll), &resistances)
                };

            ActionKindResult::AttackRollDamage {
                armor_class: systems::loadout::armor_class(world, target),
                attack_roll: attack_roll.clone(),
                damage_roll: damage_roll.clone(),
                damage_taken,
                new_life_state,
            }
        }

        ActionKindSnapshot::SavingThrowDamage {
            saving_throw_dc,
            half_damage_on_save,
            damage_roll,
        } => {
            let check_result = {
                let saving_throws =
                    systems::helpers::get_component::<&SavingThrowSet>(world, target);
                saving_throws.check_dc(&saving_throw_dc, world, target)
            };

            let (mut damage_taken, mut new_life_state) = (None, None);
            if check_result.success {
                if *half_damage_on_save {
                    // Apply half damage on successful save
                    // TODO: This is definitely not the nicest way to do this
                    let ability = match saving_throw_dc.key {
                        SavingThrowKind::Ability(ability) => ability,
                        SavingThrowKind::Death => Ability::Constitution,
                    };
                    for component in damage_roll.components.iter() {
                        resistances.add_effect(
                            component.damage_type,
                            DamageMitigationEffect {
                                // TODO: Not sure if this is the best source
                                source: ModifierSource::Ability(ability),
                                operation: MitigationOperation::Resistance,
                            },
                        );
                    }
                    (damage_taken, new_life_state) =
                        damage_internal(world, target, &damage_roll, None, &resistances);
                }
            } else {
                (damage_taken, new_life_state) =
                    damage_internal(world, target, damage_roll, None, &resistances);
            }

            ActionKindResult::SavingThrowDamage {
                saving_throw_dc: saving_throw_dc.clone(),
                saving_throw_result: check_result,
                half_damage_on_save: *half_damage_on_save,
                damage_roll: damage_roll.clone(),
                damage_taken,
                new_life_state,
            }
        }

        // TODO: Not sure how to handle composite actions yet
        // ActionKindSnapshot::Composite { actions } => {
        //     for action in actions {
        //         self.take_damage(action);
        //     }
        // }
        _ => {
            panic!(
                "Character::take_damage called with unsupported damage source (action snapshot): {:?}",
                damage_source
            );
        }
    }
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
