use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    actions::{
        action::{ActionContext, ActionKind},
        targeting::{AreaShape, TargetType, TargetingContext, TargetingKind},
    },
    combat::damage::{DamageRoll, DamageSource, DamageType},
    creature::character::Character,
    dice::dice::DieSize,
    math::point::Point,
    registry,
    spells::spell::{MagicSchool, Spell},
    stats::{ability::Ability, modifier::ModifierSource},
    utils::id::SpellId,
};

pub static SPELL_REGISTRY: LazyLock<HashMap<SpellId, Spell>> = LazyLock::new(|| {
    HashMap::from([
        (ELDRITCH_BLAST_ID.clone(), ELDRITCH_BLAST.to_owned()),
        (FIREBALL_ID.clone(), FIREBALL.to_owned()),
        (MAGIC_MISSILE_ID.clone(), MAGIC_MISSILE.to_owned()),
    ])
});

pub static ELDRITCH_BLAST_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.eldritch_blast"));

static ELDRITCH_BLAST: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        ELDRITCH_BLAST_ID.clone(),
        0, // Cantrip
        MagicSchool::Evocation,
        ActionKind::AttackRollDamage {
            attack_roll: Arc::new(|caster, _| {
                // TODO: Macro?
                Spell::spell_attack_roll(caster, spellcasting_ability(caster, &ELDRITCH_BLAST_ID))
            }),
            damage: Arc::new(|_, _| {
                DamageRoll::new(
                    1,
                    DieSize::D10,
                    DamageType::Force,
                    DamageSource::Spell,
                    "Eldritch Blast".to_string(),
                )
            }),
            damage_on_failure: None,
        },
        HashMap::from([(registry::resources::ACTION.clone(), 1)]),
        Arc::new(|caster, _| {
            let caster_level = caster.total_level();
            TargetingContext {
                kind: TargetingKind::Multiple {
                    max_targets: match caster_level {
                        1..=4 => 1,
                        5..=10 => 2,
                        11..=16 => 3,
                        _ => 4, // Level 17+ can hit up to 4 targets
                    },
                },
                normal_range: 120,
                max_range: 120,
                valid_target_types: vec![TargetType::Character],
            }
        }),
    )
});

pub static FIREBALL_ID: LazyLock<SpellId> = LazyLock::new(|| SpellId::from_str("spell.fireball"));

static FIREBALL: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        FIREBALL_ID.clone(),
        3,
        MagicSchool::Evocation,
        ActionKind::SavingThrowDamage {
            saving_throw: Arc::new(|caster, _| {
                Spell::spell_save_dc(caster, spellcasting_ability(caster, &FIREBALL_ID))
            }),
            half_damage_on_save: true,
            damage: Arc::new(|_, action_context| {
                let spell_level = match action_context {
                    ActionContext::Spell { level } => *level,
                    _ => panic!("Invalid action context"),
                };
                DamageRoll::new(
                    8 + (spell_level as u32 - 3),
                    DieSize::D6,
                    DamageType::Fire,
                    DamageSource::Spell,
                    "Fireball".to_string(),
                )
            }),
        },
        HashMap::from([(registry::resources::ACTION.clone(), 1)]),
        Arc::new(|_, _| TargetingContext {
            kind: TargetingKind::Area {
                shape: AreaShape::Sphere { radius: 20 },
                // TODO: What do we do here?
                origin: Point {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            normal_range: 150,
            max_range: 150,
            // TODO: Can also hit objects
            valid_target_types: vec![TargetType::Character],
        }),
    )
});

pub static MAGIC_MISSILE_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.magic_missile"));

static MAGIC_MISSILE: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        MAGIC_MISSILE_ID.clone(),
        1,
        MagicSchool::Evocation,
        ActionKind::UnconditionalDamage {
            damage: Arc::new(|_, _| {
                // TODO: Damage roll hooks? e.g. Empowered Evocation
                let mut damage_roll = DamageRoll::new(
                    1,
                    DieSize::D4,
                    DamageType::Force,
                    DamageSource::Spell,
                    "Magic Missile".to_string(),
                );
                damage_roll
                    .primary
                    .dice_roll
                    .modifiers
                    .add_modifier(ModifierSource::Spell(SpellId::from_str("MAGIC_MISSILE")), 1);

                damage_roll
            }),
        },
        HashMap::from([(registry::resources::ACTION.clone(), 1)]),
        Arc::new(|_, action_context| {
            let spell_level = match action_context {
                ActionContext::Spell { level } => *level,
                // TODO: Better error message? Replace other places too
                _ => panic!("Invalid action context"),
            };
            TargetingContext {
                kind: TargetingKind::Multiple {
                    max_targets: 3 + (spell_level - 1),
                },
                normal_range: 120,
                max_range: 120,
                valid_target_types: vec![TargetType::Character],
            }
        }),
    )
});

fn spellcasting_ability(caster: &Character, spell_id: &SpellId) -> Ability {
    *caster.spellbook().spellcasting_ability(spell_id).unwrap()
}
