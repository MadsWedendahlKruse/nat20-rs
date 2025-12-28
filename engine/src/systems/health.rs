use std::ops::Deref;

use hecs::{Entity, World};
use tracing::debug;

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        damage::{AttackRollResult, DamageMitigationResult, DamageResistances, DamageRollResult},
        health::{hit_points::HitPoints, life_state::LifeState},
        level::CharacterLevels,
        modifier::Modifiable,
    },
    entities::{character::CharacterTag, monster::MonsterTag},
    registry::registry::ClassesRegistry,
    systems::{self},
};

pub fn heal(world: &mut World, target: Entity, amount: u32) -> Option<LifeState> {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(target) {
        let hit_points_before = hit_points.current();
        hit_points.heal(amount);
        if let Ok(mut life_state) = world.get::<&mut LifeState>(target) {
            if hit_points.current() > 0 && hit_points_before == 0 {
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

pub fn damage(
    world: &mut World,
    target: Entity,
    damage_roll_result: &DamageRollResult,
    attack_roll: Option<&AttackRollResult>,
) -> (Option<DamageMitigationResult>, Option<LifeState>) {
    let resistances = if let Ok(resistances) = &world.get::<&mut DamageResistances>(target) {
        resistances.deref().clone()
    } else {
        DamageResistances::new()
    };

    let mut mitigation_result = resistances.apply(damage_roll_result);

    for effect in systems::effects::effects(world, target).iter() {
        (effect.damage_taken)(&world, target, &mut mitigation_result);
    }

    let (killed_by_damage, mut new_life_state) = if let Ok((hit_points, life_state)) =
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

        hit_points.damage(mitigation_result.total.max(0) as u32);
        debug!(
            "Entity {:?} took {} damage (HP: {} -> {})",
            target,
            mitigation_result.total.max(0),
            hp_before_damage,
            hit_points.current()
        );

        (
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

            for (class_id, class_level) in class_levels.all_classes() {
                if let Some(class) = ClassesRegistry::get(class_id) {
                    for level in 1..=class_level.level() {
                        let hp_increase =
                            if class_id == class_levels.first_class().unwrap() && level == 1 {
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
