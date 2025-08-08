use hecs::{Entity, World};

use crate::{
    components::{
        ability::{Ability, AbilityScoreSet},
        actions::action::{ActionKindResult, ActionKindSnapshot},
        damage::{
            DamageMitigationEffect, DamageMitigationResult, DamageResistances, DamageRollResult,
            MitigationOperation,
        },
        hit_points::HitPoints,
        level::CharacterLevels,
        modifier::ModifierSource,
        saving_throw::SavingThrowSet,
    },
    registry, systems,
};

pub fn heal(world: &mut World, target: Entity, amount: u32) {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(target) {
        hit_points.heal(amount);
    }
}

pub fn heal_full(world: &mut World, target: Entity) {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(target) {
        hit_points.heal_full();
    }
}

fn damage_internal(
    world: &mut World,
    target: Entity,
    damage_roll_result: &DamageRollResult,
    resistances: &DamageResistances,
) -> Option<DamageMitigationResult> {
    if let Ok(mut hit_points) = world.get::<&mut HitPoints>(target) {
        let mitigation_result = resistances.apply(damage_roll_result);
        hit_points.damage(mitigation_result.total.max(0) as u32);
        Some(mitigation_result)
    } else {
        None
    }
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
            ActionKindResult::UnconditionalDamage {
                damage_roll: damage_roll.clone(),
                damage_taken: damage_internal(world, target, damage_roll, &resistances),
            }
        }

        ActionKindSnapshot::AttackRollDamage {
            attack_roll,
            damage_roll,
            damage_on_failure,
        } => {
            let damage_taken = if !systems::combat::attack_hits(world, target, attack_roll) {
                if let Some(damage_on_failure) = damage_on_failure {
                    damage_internal(world, target, damage_on_failure, &resistances)
                } else {
                    None
                }
            } else {
                damage_internal(world, target, damage_roll, &resistances)
            };

            ActionKindResult::AttackRollDamage {
                armor_class: systems::loadout::armor_class(world, target),
                attack_roll: attack_roll.clone(),
                damage_roll: damage_roll.clone(),
                damage_taken,
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

            let mut damage_taken = None;
            if check_result.success {
                if *half_damage_on_save {
                    // Apply half damage on successful save
                    for component in damage_roll.components.iter() {
                        resistances.add_effect(
                            component.damage_type,
                            DamageMitigationEffect {
                                // TODO: Not sure if this is the best source
                                source: ModifierSource::Ability(saving_throw_dc.key),
                                operation: MitigationOperation::Resistance,
                            },
                        );
                    }
                    damage_taken = damage_internal(world, target, &damage_roll, &resistances);
                }
            } else {
                damage_taken = damage_internal(world, target, damage_roll, &resistances);
            }

            ActionKindResult::SavingThrowDamage {
                saving_throw_dc: saving_throw_dc.clone(),
                saving_throw_result: check_result,
                half_damage_on_save: *half_damage_on_save,
                damage_roll: damage_roll.clone(),
                damage_taken,
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
                systems::helpers::get_component::<AbilityScoreSet>(world, entity)
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
