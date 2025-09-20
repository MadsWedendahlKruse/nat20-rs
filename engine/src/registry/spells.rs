use std::{
    collections::HashMap,
    f32::consts::E,
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::{
            action::{
                self, ActionContext, ActionKind, ActionKindResult, ActionResult, ReactionResult,
            },
            targeting::{
                AreaShape, TargetType, TargetTypeInstance, TargetingContext, TargetingKind,
            },
        },
        d20::{D20Check, D20CheckDC},
        damage::{AttackRoll, DamageRoll, DamageSource, DamageType},
        dice::DieSize,
        faction::Attitude,
        id::{EntityIdentifier, SpellId},
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
        resource::ResourceAmountMap,
        saving_throw::{self, SavingThrowKind},
        spells::{
            spell::{MagicSchool, Spell},
            spellbook::Spellbook,
        },
    },
    engine::event::{ActionData, CallbackResult, Event, EventKind, EventListener, ReactionData},
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
        3,
        MagicSchool::Abjuration,
        ActionKind::Reaction {
            reaction: Arc::new(|game_state, reactor, trigger_event, reaction_context| {
                let trigger_action = match &trigger_event.kind {
                    EventKind::ActionRequested { action } => action,
                    _ => panic!("Invalid event kind for Counterspell reaction"),
                };

                let spell_save_dc = spell_save_dc(
                    &game_state.world,
                    reactor,
                    &COUNTERSPELL_ID,
                    Ability::Constitution,
                );

                let saving_throw_event = systems::d20::check(
                    game_state,
                    trigger_action.actor,
                    &D20CheckDCKind::SavingThrow(spell_save_dc.clone()),
                );
                // Wait for the actor to perform a CON save
                let _ = game_state.process_event_with_callback(
                    saving_throw_event,
                    // Once the save is resolved, continue processing the Counterspell
                    Arc::new({
                        let trigger_event = trigger_event.clone();
                        let trigger_action = trigger_action.clone();
                        let reaction_context = reaction_context.clone();
                        move |game_state, event| match &event.kind {
                            EventKind::D20CheckResolved(actor, result_kind, dc_kind) => {
                                match result_kind {
                                    D20ResultKind::SavingThrow { kind, result } => {
                                        let result = if result.success {
                                            // Successful save, Counterspell fails
                                            ReactionResult::NoEffect
                                        } else {
                                            // Spell slots are not consumed by Counterspell
                                            let mut resources_refunded = ResourceAmountMap::new();
                                            resources_refunded.insert(
                                                registry::resources::SPELL_SLOT_ID.clone(),
                                                trigger_action
                                                    .resource_cost
                                                    .get(&registry::resources::SPELL_SLOT_ID)
                                                    .cloned()
                                                    .unwrap(),
                                            );
                                            // Failed save, Counterspell succeeds
                                            ReactionResult::CancelEvent {
                                                event: trigger_event.clone().into(),
                                                resources_refunded,
                                            }
                                        };

                                        CallbackResult::Event(Event::new(
                                            EventKind::ActionPerformed {
                                                action: ActionData {
                                                    actor: reactor,
                                                    action_id: COUNTERSPELL_ID
                                                        .clone()
                                                        .to_action_id(),
                                                    context: reaction_context.clone(),
                                                    resource_cost: ResourceAmountMap::from([(
                                                        registry::resources::REACTION_ID.clone(),
                                                        registry::resources::REACTION
                                                            .build_amount(1),
                                                    )]),
                                                    targets: vec![*actor],
                                                },
                                                results: vec![ActionResult {
                                                    performer: EntityIdentifier::from_world(
                                                        &game_state.world,
                                                        reactor,
                                                    ),
                                                    target: TargetTypeInstance::Entity(
                                                        EntityIdentifier::from_world(
                                                            &game_state.world,
                                                            *actor,
                                                        ),
                                                    ),
                                                    kind: ActionKindResult::Reaction { result },
                                                }],
                                            },
                                        ))
                                    }
                                    _ => panic!("Invalid result kind in Counterspell callback"),
                                }
                            }
                            _ => panic!("Invalid event kind in Counterspell callback"),
                        }
                    }),
                );
            }),
        },
        ResourceAmountMap::from([(
            registry::resources::REACTION_ID.clone(),
            registry::resources::REACTION.build_amount(1),
        )]),
        Arc::new(|_, _, _| TargetingContext {
            kind: TargetingKind::Single,
            normal_range: 60,
            max_range: 60,
            valid_target_types: vec![TargetType::entity_not_dead()],
        }),
        Some(Arc::new(|reactor, trigger_event| {
            match &trigger_event.kind {
                EventKind::ActionRequested { action } => {
                    if reactor == action.actor {
                        // Cannot counterspell yourself
                        return false;
                    }
                    // TODO: Can we just counterspell spells of any level?
                    // Find a way to get rid of ActionContext::Reaction
                    match &action.context {
                        ActionContext::Spell { level } => true,
                        // TODO: *NOT* a fan of having to check both of these
                        ActionContext::Reaction { context, .. } => match context.as_ref() {
                            ActionContext::Spell { level } => true,
                            _ => false,
                        },
                        _ => false,
                    }
                }
                _ => false,
            }
        })),
    )
});

pub static ELDRITCH_BLAST_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.eldritch_blast"));

static ELDRITCH_BLAST: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        ELDRITCH_BLAST_ID.clone(),
        0,
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
        registry::actions::DEFAULT_RESOURCE_COST.clone(),
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
            saving_throw: Arc::new(|world, caster, _| {
                spell_save_dc(world, caster, &FIREBALL_ID, Ability::Dexterity)
            }),
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
        registry::actions::DEFAULT_RESOURCE_COST.clone(),
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
        registry::actions::DEFAULT_RESOURCE_COST.clone(),
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

fn spell_save_dc(
    world: &World,
    caster: Entity,
    spell_id: &SpellId,
    saving_throw_ability: Ability,
) -> D20CheckDC<SavingThrowKind> {
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
        key: SavingThrowKind::Ability(saving_throw_ability),
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
