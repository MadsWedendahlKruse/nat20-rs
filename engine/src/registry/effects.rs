use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    components::{
        actions::action::ActionContext,
        d20_check::AdvantageType,
        damage::DamageSource,
        effects::{
            effects::{Effect, EffectDuration},
            hooks::SkillCheckHook,
        },
        id::EffectId,
        items::equipment::{armor::ArmorType, loadout, weapon::WeaponType},
        modifier::ModifierSource,
        resource::{RechargeRule, Resource, ResourceMap},
        skill::Skill,
    },
    registry, systems,
};

pub static EFFECT_REGISTRY: LazyLock<HashMap<EffectId, Effect>> = LazyLock::new(|| {
    HashMap::from([
        (ACTION_SURGE_ID.clone(), ACTION_SURGE.to_owned()),
        (
            ARMOR_STEALTH_DISADVANTAGE_ID.clone(),
            ARMOR_STEALTH_DISADVANTAGE.to_owned(),
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

pub static ARMOR_STEALTH_DISADVANTAGE_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.armor.stealth_disadvantage"));

pub static ARMOR_STEALTH_DISADVANTAGE: LazyLock<Effect> = LazyLock::new(|| {
    let modifier_source: ModifierSource = ModifierSource::Item("Armor".to_string());

    let mut stealth_disadvantage_effect = Effect::new(
        EffectId::from_str("effect.armor.stealth_disadvantage"),
        modifier_source.clone(),
        EffectDuration::Permanent,
    );

    let mut skill_check_hook = SkillCheckHook::new(Skill::Stealth);
    skill_check_hook.check_hook = Arc::new(move |_, _, d20_check| {
        d20_check
            .advantage_tracker_mut()
            .add(AdvantageType::Disadvantage, modifier_source.clone());
    });
    stealth_disadvantage_effect.on_skill_check = Some(skill_check_hook);

    stealth_disadvantage_effect
});

pub static EXTRA_ATTACK_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.extra_attack"));

static EXTRA_ATTACK: LazyLock<Effect> =
    LazyLock::new(|| extra_attack_effect(EXTRA_ATTACK_ID.clone(), 2));

fn extra_attack_effect(effect_id: EffectId, charges: u8) -> Effect {
    let mut effect = Effect::new(
        effect_id.clone(),
        ModifierSource::ClassFeature(effect_id.to_string()),
        EffectDuration::Permanent,
    );

    // TODO: The logic seems sound, but we need to apply the hook when finding
    // the resource cost of the action, not when it's performed!
    // BUT we also need to give the "Extra Attack" resource when the Action is
    // performed, so we need to do both??

    effect.on_action = Arc::new({
        // This closure captures the `charges` variable, so we can use it in the
        // closure without having to pass it as an argument.
        let charges = charges;
        move |world, performer, action, context| {
            // Check that this is only applied for weapon attacks
            // TODO: Is this logic sufficient? (And is it the nicest way to do this?)
            if !matches!(
                context,
                ActionContext::Weapon {
                    weapon_type: _,
                    hand: _
                }
            ) {
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
        if !matches!(
            context,
            ActionContext::Weapon {
                weapon_type: _,
                hand: _
            }
        ) {
            return;
        }

        // If the Action doesn't cost an "Action" this effect is not relevant
        if !resource_cost.contains_key(&registry::resources::ACTION) {
            return;
        }

        // If the character has any "Extra Attack" charges, we use those instead
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
            DamageSource::Weapon(weapon_type) => *weapon_type == WeaponType::Ranged,
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
            DamageSource::Weapon(weapon_type) => *weapon_type != WeaponType::Melee,
            _ => false,
        } {
            return;
        }

        let loadout = systems::helpers::get_component::<loadout::Loadout>(world, entity);
        if !loadout.is_wielding_weapon_with_both_hands(&WeaponType::Melee) {
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
