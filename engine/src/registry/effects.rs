use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    components::{
        ability::Ability,
        actions::action::ActionContext,
        d20_check::AdvantageType,
        damage::{DamageSource, DamageType},
        effects::{
            effects::{Effect, EffectDuration},
            hooks::D20CheckHooks,
        },
        id::{EffectId, ItemId},
        items::{
            equipment::{armor::ArmorType, loadout, weapon::WeaponKind},
            item::Item,
        },
        modifier::ModifierSource,
        resource::{RechargeRule, Resource, ResourceMap},
        skill::{Skill, SkillSet},
    },
    registry, systems,
};

pub static EFFECT_REGISTRY: LazyLock<HashMap<EffectId, Effect>> = LazyLock::new(|| {
    HashMap::from([
        (ACTION_SURGE_ID.clone(), ACTION_SURGE.to_owned()),
        (
            ARMOR_OF_CONSTITUTION_SAVING_THROWS_ID.clone(),
            ARMOR_OF_CONSTITUTION_SAVING_THROWS.to_owned(),
        ),
        (ARMOR_OF_SNEAKING_ID.clone(), ARMOR_OF_SNEAKING.to_owned()),
        (
            ARMOR_STEALTH_DISADVANTAGE_ID.clone(),
            ARMOR_STEALTH_DISADVANTAGE.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_BLACK_ID.clone(),
            DRACONIC_ANCESTRY_BLACK.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_BLUE_ID.clone(),
            DRACONIC_ANCESTRY_BLUE.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_BRASS_ID.clone(),
            DRACONIC_ANCESTRY_BRASS.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_BRONZE_ID.clone(),
            DRACONIC_ANCESTRY_BRONZE.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_COPPER_ID.clone(),
            DRACONIC_ANCESTRY_COPPER.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_GOLD_ID.clone(),
            DRACONIC_ANCESTRY_GOLD.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_GREEN_ID.clone(),
            DRACONIC_ANCESTRY_GREEN.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_RED_ID.clone(),
            DRACONIC_ANCESTRY_RED.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_SILVER_ID.clone(),
            DRACONIC_ANCESTRY_SILVER.to_owned(),
        ),
        (
            DRACONIC_ANCESTRY_WHITE_ID.clone(),
            DRACONIC_ANCESTRY_WHITE.to_owned(),
        ),
        (EXTRA_ATTACK_ID.clone(), EXTRA_ATTACK.to_owned()),
        (
            FIGHTING_STYLE_ARCHERY_ID.clone(),
            FIGHTING_STYLE_ARCHERY.to_owned(),
        ),
        (
            FIGHTING_STYLE_DEFENSE_ID.clone(),
            FIGHTING_STYLE_DEFENSE.to_owned(),
        ),
        (
            FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
            FIGHTING_STYLE_GREAT_WEAPON_FIGHTING.to_owned(),
        ),
        (IMPROVED_CRITICAL_ID.clone(), IMPROVED_CRITICAL.to_owned()),
        (REMARKABLE_ATHLETE_ID.clone(), REMARKABLE_ATHLETE.to_owned()),
        (RING_OF_ATTACKING_ID.clone(), RING_OF_ATTACKING.to_owned()),
        (SUPERIOR_CRITICAL_ID.clone(), SUPERIOR_CRITICAL.to_owned()),
        (
            THREE_EXTRA_ATTACKS_ID.clone(),
            THREE_EXTRA_ATTACKS.to_owned(),
        ),
        (TWO_EXTRA_ATTACKS_ID.clone(), TWO_EXTRA_ATTACKS.to_owned()),
    ])
});

pub static ACTION_SURGE_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.action_surge"));

static ACTION_SURGE: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        ACTION_SURGE_ID.clone(),
        ModifierSource::ClassFeature(ACTION_SURGE_ID.to_string()),
        EffectDuration::temporary(1),
    );
    effect.on_apply = Arc::new(|world, entity| {
        let _ = systems::helpers::get_component_mut::<ResourceMap>(world, entity)
            .get_mut(&registry::resources::ACTION)
            .unwrap()
            .add_use();
    });
    effect.on_unapply = Arc::new(|world, entity| {
        let _ = systems::helpers::get_component_mut::<ResourceMap>(world, entity)
            .get_mut(&registry::resources::ACTION)
            .unwrap()
            .remove_use();
    });
    effect
});

pub static ARMOR_OF_CONSTITUTION_SAVING_THROWS_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.item.armor_of_constitution_saving_throws"));

static ARMOR_OF_CONSTITUTION_SAVING_THROWS: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        ARMOR_OF_CONSTITUTION_SAVING_THROWS_ID.clone(),
        ModifierSource::Item(ItemId::from_str("item.armor_of_constitution_saving_throws")),
        EffectDuration::Permanent,
    );

    effect.on_saving_throw = HashMap::from([(
        Ability::Constitution,
        D20CheckHooks::with_check_hook(|_, _, d20_check| {
            d20_check.advantage_tracker_mut().add(
                AdvantageType::Advantage,
                ModifierSource::Item(ItemId::from_str("item.armor_of_constitution_saving_throws")),
            );
        }),
    )]);

    effect
});

pub static ARMOR_OF_SNEAKING_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.item.armor_of_sneaking"));

static ARMOR_OF_SNEAKING: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        ARMOR_OF_SNEAKING_ID.clone(),
        ModifierSource::Item(ItemId::from_str("item.armor_of_sneaking")),
        EffectDuration::Permanent,
    );

    effect.on_apply = Arc::new({
        move |world, entity| {
            let mut skills = systems::helpers::get_component_mut::<SkillSet>(world, entity);
            skills.add_modifier(
                Skill::Stealth,
                ModifierSource::Item(ItemId::from_str("item.armor_of_sneaking")),
                2,
            );
        }
    });

    effect.on_unapply = Arc::new({
        move |world, entity| {
            let mut skills = systems::helpers::get_component_mut::<SkillSet>(world, entity);
            skills.remove_modifier(
                Skill::Stealth,
                &ModifierSource::Item(ItemId::from_str("item.armor_of_sneaking")),
            );
        }
    });

    effect
});

pub static ARMOR_STEALTH_DISADVANTAGE_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.armor.stealth_disadvantage"));

pub static ARMOR_STEALTH_DISADVANTAGE: LazyLock<Effect> = LazyLock::new(|| {
    let modifier_source: ModifierSource =
        ModifierSource::Item(ItemId::from_str("item.plate_armor"));

    let mut stealth_disadvantage_effect = Effect::new(
        EffectId::from_str("effect.armor.stealth_disadvantage"),
        modifier_source.clone(),
        EffectDuration::Permanent,
    );

    let mut stealth_hook = D20CheckHooks::new();
    stealth_hook.check_hook = Arc::new(move |_, _, d20_check| {
        d20_check
            .advantage_tracker_mut()
            .add(AdvantageType::Disadvantage, modifier_source.clone());
    });
    stealth_disadvantage_effect
        .on_skill_check
        .insert(Skill::Stealth, stealth_hook);

    stealth_disadvantage_effect
});

macro_rules! draconic_ancestry {
    ($( $Name:ident => $slug:literal => $dtype:path ),+ $(,)?) => {
        use paste::paste;
        paste! {
            $(
                pub static [<DRACONIC_ANCESTRY_ $Name _ID>]: LazyLock<EffectId> =
                    LazyLock::new(|| EffectId::from_str(concat!("effect.race.draconic_ancestry.", $slug)));

                pub static [<DRACONIC_ANCESTRY_ $Name>]: LazyLock<Effect> =
                    LazyLock::new(|| {
                        helpers::damage_resistance_effect(
                            [<DRACONIC_ANCESTRY_ $Name _ID>].clone(),
                            $dtype
                        )
                    });
            )+
        }
    };
}

draconic_ancestry!(
    BLACK  => "black"  => DamageType::Acid,
    BLUE   => "blue"   => DamageType::Lightning,
    BRASS  => "brass"  => DamageType::Fire,
    BRONZE => "bronze" => DamageType::Lightning,
    COPPER => "copper" => DamageType::Acid,
    GOLD   => "gold"   => DamageType::Fire,
    GREEN  => "green"  => DamageType::Poison,
    RED    => "red"    => DamageType::Fire,
    SILVER => "silver" => DamageType::Cold,
    WHITE  => "white"  => DamageType::Cold,
);

pub static EXTRA_ATTACK_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.extra_attack"));

static EXTRA_ATTACK: LazyLock<Effect> =
    LazyLock::new(|| helpers::extra_attack_effect(EXTRA_ATTACK_ID.clone(), 2));

// TODO: In the SRD fighting styles are a specific type of Feat, but I don't think that's necessary

pub static FIGHTING_STYLE_ARCHERY_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighting_style.archery"));

static FIGHTING_STYLE_ARCHERY: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        FIGHTING_STYLE_ARCHERY_ID.clone(),
        ModifierSource::ClassFeature("Fighting Style: Archery".to_string()),
        EffectDuration::Permanent,
    );
    effect.pre_attack_roll = Arc::new(|_, _, attack_roll| {
        if match &attack_roll.source {
            DamageSource::Weapon(weapon_type) => *weapon_type == WeaponKind::Ranged,
            _ => false,
        } {
            attack_roll.d20_check.add_modifier(
                ModifierSource::ClassFeature("Fighting Style: Archery".to_string()),
                2,
            );
        }
    });
    effect
});

pub static FIGHTING_STYLE_DEFENSE_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighting_style.defense"));

static FIGHTING_STYLE_DEFENSE: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        FIGHTING_STYLE_DEFENSE_ID.clone(),
        ModifierSource::ClassFeature("Fighting Style: Defense".to_string()),
        EffectDuration::Permanent,
    );
    effect.on_armor_class = Arc::new(|world, entity, armor_class| {
        let loadout = systems::helpers::get_component::<loadout::Loadout>(world, entity);
        // If the character is not wearing armor, we don't apply this effect
        if let Some(armor) = &loadout.armor() {
            if armor.armor_type == ArmorType::Clothing {
                return;
            }
        } else {
            return;
        }
        armor_class.add_modifier(
            ModifierSource::ClassFeature("Fighting Style: Defense".to_string()),
            1,
        );
    });
    effect
});

pub static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighting_style.great_weapon_fighting"));

static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
        ModifierSource::ClassFeature("Fighting Style: Great Weapon Fighting".to_string()),
        EffectDuration::Permanent,
    );
    effect.post_damage_roll = Arc::new(|world, entity, damage_roll_result| {
        // Great weapon fighting only applies to melee attacks (with both hands)
        if match &damage_roll_result.source {
            DamageSource::Weapon(weapon_type) => *weapon_type != WeaponKind::Melee,
            _ => false,
        } {
            return;
        }

        let loadout = systems::helpers::get_component::<loadout::Loadout>(world, entity);
        if !loadout.is_wielding_weapon_with_both_hands(&WeaponKind::Melee) {
            return;
        }

        // TODO: When does this ever happen?
        if damage_roll_result.components.is_empty() {
            return;
        }

        // First component is the primary damage component
        let primary_damage_rolls = &mut damage_roll_result.components[0].result.rolls;
        for i in 0..primary_damage_rolls.len() {
            // Any roll that is less than 3 is rerolled to 3
            if primary_damage_rolls[i] < 3 {
                primary_damage_rolls[i] = 3
            }
        }
    });
    effect
});

pub static IMPROVED_CRITICAL_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.champion.improved_critical"));

static IMPROVED_CRITICAL: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        IMPROVED_CRITICAL_ID.clone(),
        ModifierSource::ClassFeature("Improved Critical".to_string()),
        EffectDuration::Permanent,
    );
    effect.pre_attack_roll = Arc::new(|_, _, attack_roll| {
        attack_roll.reduce_crit_threshold(1);
    });
    effect
});

pub static REMARKABLE_ATHLETE_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.champion.remarkable_athlete"));

static REMARKABLE_ATHLETE: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        REMARKABLE_ATHLETE_ID.clone(),
        ModifierSource::ClassFeature("Remarkable Athlete".to_string()),
        EffectDuration::Permanent,
    );

    [Skill::Athletics, Skill::Initiative]
        .iter()
        .for_each(|skill| {
            effect.on_skill_check.insert(
                *skill,
                D20CheckHooks::with_check_hook(|_, _, d20_check| {
                    d20_check.advantage_tracker_mut().add(
                        AdvantageType::Advantage,
                        ModifierSource::ClassFeature("Remarkable Athlete".to_string()),
                    );
                }),
            );
        });
    effect
});

pub static RING_OF_ATTACKING_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.item.ring_of_attacking"));

static RING_OF_ATTACKING: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        RING_OF_ATTACKING_ID.clone(),
        ModifierSource::Item(ItemId::from_str("item.ring_of_attacking")),
        EffectDuration::Permanent,
    );
    effect.pre_attack_roll = Arc::new(|_, _, attack_roll| {
        attack_roll.d20_check.advantage_tracker_mut().add(
            AdvantageType::Advantage,
            ModifierSource::Item(ItemId::from_str("item.ring_of_attacking")),
        );
    });
    effect
});

pub static SUPERIOR_CRITICAL_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.champion.superior_critical"));

static SUPERIOR_CRITICAL: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        SUPERIOR_CRITICAL_ID.clone(),
        ModifierSource::ClassFeature("Superior Critical".to_string()),
        EffectDuration::Permanent,
    );
    effect.pre_attack_roll = Arc::new(|_, _, attack_roll| {
        attack_roll.reduce_crit_threshold(2);
    });
    effect.replaces = Some(IMPROVED_CRITICAL_ID.clone());
    effect
});

pub static THREE_EXTRA_ATTACKS_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.three_extra_attacks"));

static THREE_EXTRA_ATTACKS: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = helpers::extra_attack_effect(THREE_EXTRA_ATTACKS_ID.clone(), 3);
    effect.replaces = Some(TWO_EXTRA_ATTACKS_ID.clone());
    effect
});

pub static TWO_EXTRA_ATTACKS_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighter.two_extra_attacks"));

static TWO_EXTRA_ATTACKS: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = helpers::extra_attack_effect(TWO_EXTRA_ATTACKS_ID.clone(), 3);
    effect.replaces = Some(EXTRA_ATTACK_ID.clone());
    effect
});

#[macro_use]
mod helpers {
    use crate::components::damage::{
        DamageMitigationEffect, DamageResistances, DamageType, MitigationOperation,
    };

    use super::*;

    pub fn extra_attack_effect(effect_id: EffectId, charges: u8) -> Effect {
        let mut effect = Effect::new(
            effect_id.clone(),
            ModifierSource::ClassFeature(effect_id.to_string()),
            EffectDuration::Permanent,
        );

        effect.on_action = Arc::new({
            // This closure captures the `charges` variable, so we can use it in the
            // closure without having to pass it as an argument.
            let charges = charges;
            move |world, performer, action, context| {
                // Check that this is only applied for weapon attacks
                // TODO: Is this logic sufficient? (And is it the nicest way to do this?)
                if !matches!(context, ActionContext::Weapon { .. }) {
                    return;
                }

                // If the Action doesn't cost an "Action" this effect is not relevant
                if !action
                    .resource_cost()
                    .contains_key(&registry::resources::ACTION)
                {
                    return;
                }

                // The first time the character triggers Extra Attack their action is
                // consumed and they're given a number of charges of an "Extra Attack"
                // resource.
                // Check if the character has any of those charges (i.e. they've already
                // triggered Extra Attack). Otherwise, use an action and give them the
                // "Extra Attack" charges.
                let mut resources =
                    systems::helpers::get_component_mut::<ResourceMap>(world, performer);
                if let Some(extra_attack) = resources.get(&registry::resources::EXTRA_ATTACK) {
                    if extra_attack.current_uses() > 0 {
                        return;
                    }
                }

                // Consume the action and give the "Extra Attack" charges
                // TODO: Assume the action has been validated?
                let _ = resources
                    .get_mut(&registry::resources::ACTION)
                    .unwrap()
                    .spend(1);
                resources.add(
                    Resource::new(
                        registry::resources::EXTRA_ATTACK.clone(),
                        charges,
                        RechargeRule::Never,
                    )
                    .unwrap(),
                    true, // Set current uses to max uses
                );
            }
        });

        effect.on_resource_cost = Arc::new(|world, performer, context, resource_cost| {
            // Check that this is only applied for weapon attacks
            if !matches!(context, ActionContext::Weapon { .. }) {
                return;
            }

            // If the Action doesn't cost an "Action" this effect is not relevant
            if !resource_cost.contains_key(&registry::resources::ACTION) {
                return;
            }

            // If the character has any "Extra Attack" charges, we use those instead
            // of the "Action" resource.
            let resources = systems::helpers::get_component::<ResourceMap>(world, performer);
            if let Some(extra_attack) = resources.get(&registry::resources::EXTRA_ATTACK) {
                if extra_attack.current_uses() > 0 {
                    resource_cost.remove(&registry::resources::ACTION);
                    resource_cost.insert(registry::resources::EXTRA_ATTACK.clone(), 1);
                }
            }
        });

        effect
    }

    pub fn damage_resistance_effect(effect_id: EffectId, damage_type: DamageType) -> Effect {
        let mut effect = Effect::new(
            effect_id.clone(),
            ModifierSource::ClassFeature(effect_id.to_string()),
            EffectDuration::Permanent,
        );

        effect.on_apply = Arc::new({
            move |world, entity| {
                let mut damage_resistances =
                    systems::helpers::get_component_mut::<DamageResistances>(world, entity);
                damage_resistances.add_effect(
                    damage_type,
                    DamageMitigationEffect {
                        source: ModifierSource::Effect(effect_id.clone()),
                        operation: MitigationOperation::Resistance,
                    },
                );
            }
        });

        // TODO: Implement!
        // effect.on_unapply = Arc::new({
        //     move |world, entity| {
        //         let mut damage_resistances =
        //             systems::helpers::get_component_mut::<DamageResistances>(world, entity);
        //         damage_resistances.remove_effect(
        //             damage_type,
        //             &DamageMitigationEffect {
        //                 source: ModifierSource::Effect(effect_id.clone()),
        //                 operation: MitigationOperation::Resistance,
        //             },
        //         );
        //     }
        // });

        effect
    }
}
