use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::{
            action::{self, ActionContext, ActionKind, ReactionResult},
            targeting::{AreaShape, TargetType, TargetingContext, TargetingKind},
        },
        d20::{D20Check, D20CheckDC},
        damage::{AttackRoll, DamageRoll, DamageSource, DamageType},
        dice::DieSize,
        faction::Attitude,
        id::SpellId,
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
        resource::ResourceCostMap,
        saving_throw::SavingThrowKind,
        spells::{
            spell::{MagicSchool, Spell},
            spellbook::Spellbook,
        },
    },
    engine::event::{ActionData, Event, EventKind, EventListener, EventOrListener, ReactionData},
    math::point::Point,
    registry,
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub static SPELL_REGISTRY: LazyLock<HashMap<SpellId, Spell>> = LazyLock::new(|| {
    HashMap::from([
        (COUNTERSPELL_ID.clone(), COUNTERSPELL.to_owned()),
        (ELDRITCH_BLAST_ID.clone(), ELDRITCH_BLAST.to_owned()),
        (FIREBALL_ID.clone(), FIREBALL.to_owned()),
        (MAGIC_MISSILE_ID.clone(), MAGIC_MISSILE.to_owned()),
    ])
});

pub static COUNTERSPELL_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.counterspell"));

static COUNTERSPELL: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        COUNTERSPELL_ID.clone(),
        3, // Level 3 spell
        MagicSchool::Abjuration,
        ActionKind::Reaction {
            reaction: Arc::new(|game_state, reactor, trigger_event, action_context| {
                let action = match &trigger_event.kind {
                    EventKind::ActionRequested { action } => action,
                    _ => panic!("Invalid event kind for Counterspell reaction"),
                };

                let spell_save_dc = spell_save_dc(&game_state.world, reactor, &COUNTERSPELL_ID);

                let saving_throw_event_id = systems::d20::check(
                    game_state,
                    reactor,
                    &D20CheckDCKind::SavingThrow(spell_save_dc.clone()),
                );

                // Wait for the actor to perform a CON save
                game_state.add_event_listener(EventListener {
                    trigger_id: saving_throw_event_id,

                    // Once the save is resolved, continue processing the Counterspell
                    callback: Arc::new({
                        let trigger_event = trigger_event.clone();
                        let action_context = action_context.clone();
                        move |game_state, event| match &event.kind {
                            EventKind::D20CheckResolved(actor, result_kind, dc_kind) => {
                                match result_kind {
                                    D20ResultKind::SavingThrow { kind, result } => {
                                        let result = if result.success {
                                            // Successful save, Counterspell fails
                                            ReactionResult::NoEffect
                                        } else {
                                            // Failed save, Counterspell succeeds
                                            ReactionResult::CancelEvent {
                                                event_id: trigger_event.id.clone(),
                                            }
                                        };

                                        EventOrListener::Event(Event::new(
                                            EventKind::ReactionPerformed {
                                                reactor: reactor,
                                                reaction: ReactionData {
                                                    reaction_id: COUNTERSPELL_ID
                                                        .clone()
                                                        .to_action_id(),
                                                    context: action_context.clone(),
                                                    // resource_cost: todo!(),
                                                    kind: result,
                                                }
                                                .into(),
                                                event: trigger_event.clone().into(),
                                            },
                                        ))
                                    }
                                    _ => panic!("Invalid result kind in Counterspell callback"),
                                }
                            }
                            _ => panic!("Invalid event kind in Counterspell callback"),
                        }
                    }),
                });
            }),
        },
        ResourceCostMap::from([(registry::resources::REACTION.clone(), 1)]),
        Arc::new(|_, _, _| TargetingContext {
            kind: TargetingKind::Single,
            normal_range: 60,
            max_range: 60,
            valid_target_types: vec![TargetType::entity_not_dead()],
        }),
        Some(Arc::new(|reactor, trigger_event, reaction_context| {
            match &trigger_event.kind {
                EventKind::ActionRequested { action } => {
                    if reactor == action.actor {
                        // Cannot counterspell yourself
                        return None;
                    }
                    // TODO: Can we just counterspell spells of any level?
                    match action.context {
                        ActionContext::Spell { level } => {
                            return Some(ReactionResult::NewEvent {
                                event: EventKind::ActionRequested {
                                    action: ActionData {
                                        actor: reactor,
                                        action_id: COUNTERSPELL_ID.clone().to_action_id(),
                                        context: reaction_context.clone(),
                                        targets: vec![action.actor],
                                    },
                                },
                            });
                        }
                        _ => return None,
                    }
                }
                _ => None,
            }
        })),
    )
});

pub static ELDRITCH_BLAST_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.eldritch_blast"));

static ELDRITCH_BLAST: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        ELDRITCH_BLAST_ID.clone(),
        0, // Cantrip
        MagicSchool::Evocation,
        ActionKind::AttackRollDamage {
            attack_roll: Arc::new(|world, caster, _| {
                spell_attack_roll(world, caster, &ELDRITCH_BLAST_ID)
            }),
            damage: Arc::new(|_, _, _| {
                DamageRoll::new(
                    1,
                    DieSize::D10,
                    DamageType::Force,
                    DamageSource::Spell,
                    "Eldritch Blast".to_string(),
                )
            }),
            damage_on_miss: None,
        },
        HashMap::from([(registry::resources::ACTION.clone(), 1)]),
        Arc::new(|world, entity, _| {
            let caster_level = systems::helpers::level(world, entity)
                .unwrap()
                .total_level();
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
                valid_target_types: vec![TargetType::entity_not_dead()],
            }
        }),
        None,
    )
});

pub static FIREBALL_ID: LazyLock<SpellId> = LazyLock::new(|| SpellId::from_str("spell.fireball"));

static FIREBALL: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        FIREBALL_ID.clone(),
        3,
        MagicSchool::Evocation,
        ActionKind::SavingThrowDamage {
            saving_throw: Arc::new(|world, caster, _| spell_save_dc(world, caster, &FIREBALL_ID)),
            half_damage_on_save: true,
            damage: Arc::new(|_, _, action_context| {
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
        Arc::new(|_, _, _| TargetingContext {
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
            valid_target_types: vec![TargetType::entity_not_dead()],
        }),
        None,
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
            damage: Arc::new(|_, _, _| {
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
        Arc::new(|_, _, action_context| {
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
                valid_target_types: vec![TargetType::entity_not_dead()],
            }
        }),
        None,
    )
});

const BASE_SPELL_SAVE_DC: i32 = 8;

fn spell_save_dc(world: &World, caster: Entity, spell_id: &SpellId) -> D20CheckDC<SavingThrowKind> {
    let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(world, caster);
    let spellcasting_ability = systems::helpers::get_component::<Spellbook>(world, caster)
        .spellcasting_ability(spell_id)
        .unwrap()
        .clone();
    let proficiency_bonus = systems::helpers::level(world, caster)
        .unwrap()
        .proficiency_bonus();

    let mut spell_save_dc = ModifierSet::new();
    spell_save_dc.add_modifier(
        ModifierSource::Custom("Base spell save DC".to_string()),
        BASE_SPELL_SAVE_DC,
    );
    let spellcasting_modifier = ability_scores
        .ability_modifier(spellcasting_ability)
        .total();
    spell_save_dc.add_modifier(
        ModifierSource::Ability(spellcasting_ability),
        spellcasting_modifier,
    );
    // TODO: Not sure if Proficiency is the correct modifier source here, since I don't think
    // you can have e.g. Expertise in spell save DCs.
    spell_save_dc.add_modifier(
        ModifierSource::Proficiency(ProficiencyLevel::Proficient),
        proficiency_bonus as i32,
    );

    D20CheckDC {
        key: SavingThrowKind::Ability(spellcasting_ability),
        dc: spell_save_dc,
    }
}

fn spell_attack_roll(world: &World, caster: Entity, spell_id: &SpellId) -> AttackRoll {
    let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(world, caster);
    let spellcasting_ability = systems::helpers::get_component::<Spellbook>(world, caster)
        .spellcasting_ability(spell_id)
        .unwrap()
        .clone();
    let proficiency_bonus = systems::helpers::level(world, caster)
        .unwrap()
        .proficiency_bonus();

    let mut roll = D20Check::new(Proficiency::new(
        ProficiencyLevel::Proficient,
        ModifierSource::None,
    ));
    let spellcasting_modifier = ability_scores
        .ability_modifier(spellcasting_ability)
        .total();
    roll.add_modifier(
        ModifierSource::Ability(spellcasting_ability),
        spellcasting_modifier,
    );
    roll.add_modifier(
        ModifierSource::Proficiency(ProficiencyLevel::Proficient),
        proficiency_bonus as i32,
    );

    AttackRoll::new(roll, DamageSource::Spell)
}
