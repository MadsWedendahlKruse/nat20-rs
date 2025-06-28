use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    combat::damage::DamageSource,
    effects::effects::{Effect, EffectDuration},
    items::equipment::{armor::ArmorType, weapon::WeaponType},
    registry,
    stats::modifier::ModifierSource,
    utils::id::EffectId,
};

pub static EFFECT_REGISTRY: LazyLock<HashMap<EffectId, Effect>> = LazyLock::new(|| {
    HashMap::from([
        (ACTION_SURGE_ID.clone(), ACTION_SURGE.to_owned()),
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
    effect.on_apply = Arc::new(|character| {
        let _ = character
            .resource_mut(&registry::resources::ACTION)
            .unwrap()
            .add_use();
    });
    effect.on_unapply = Arc::new(|character| {
        let _ = character
            .resource_mut(&registry::resources::ACTION)
            .unwrap()
            .remove_use();
    });
    effect
});

// TODO: In the SRD fighting styles are a specific type of Feat, but I don't think that's necessary

pub static FIGHTING_STYLE_ARCHERY_ID: LazyLock<EffectId> =
    LazyLock::new(|| EffectId::from_str("effect.fighting_style.archery"));

static FIGHTING_STYLE_ARCHERY: LazyLock<Effect> = LazyLock::new(|| {
    let mut effect = Effect::new(
        FIGHTING_STYLE_ARCHERY_ID.clone(),
        ModifierSource::ClassFeature("Fighting Style: Archery".to_string()),
        EffectDuration::Persistent,
    );
    effect.pre_attack_roll = Arc::new(|_, attack_roll| {
        if match &attack_roll.source {
            DamageSource::Weapon(weapon_type, _) => *weapon_type == WeaponType::Ranged,
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
        EffectDuration::Persistent,
    );
    effect.on_armor_class = Arc::new(|character, armor_class| {
        if let Some(armor) = &character.loadout().armor {
            if armor.armor_type == ArmorType::Clothing {
                return;
            }
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
        EffectDuration::Persistent,
    );
    effect.post_damage_roll = Arc::new(|character, damage_roll_result| {
        // Great weapon fighting only applies to melee attacks (with both hands)
        if match &damage_roll_result.source {
            DamageSource::Weapon(weapon_type, _) => *weapon_type != WeaponType::Melee,
            _ => false,
        } {
            return;
        }

        if !character
            .loadout()
            .is_wielding_weapon_with_both_hands(&WeaponType::Melee)
        {
            return;
        }

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
        EffectDuration::Persistent,
    );
    effect.pre_attack_roll = Arc::new(|_, attack_roll| {
        attack_roll.reduce_crit_threshold(1);
    });
    effect
});
