use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use hecs::{Entity, World};
use uom::si::{f32::Length, length::foot};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::{
            action::{ActionContext, ActionKind, ActionKindResult, ReactionResult},
            targeting::{AreaShape, EntityFilter, TargetingContext, TargetingKind, TargetingRange},
        },
        d20::{D20Check, D20CheckDC},
        damage::{AttackRoll, DamageRoll, DamageSource, DamageType},
        dice::{DiceSet, DieSize},
        id::{ResourceId, SpellId},
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
        resource::{ResourceAmount, ResourceAmountMap},
        saving_throw::SavingThrowKind,
        spells::{
            spell::{MagicSchool, Spell},
            spellbook::Spellbook,
        },
    },
    engine::event::{ActionData, CallbackResult, Event, EventKind},
    registry,
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub static SPELL_REGISTRY: LazyLock<HashMap<SpellId, Spell>> =
    LazyLock::new(|| HashMap::from([(COUNTERSPELL_ID.clone(), COUNTERSPELL.to_owned())]));

pub static COUNTERSPELL_ID: LazyLock<SpellId> =
    LazyLock::new(|| SpellId::from_str("spell.counterspell"));

static COUNTERSPELL: LazyLock<Spell> = LazyLock::new(|| {
    Spell::new(
        COUNTERSPELL_ID.clone(),
        "You attempt to interrupt a creature in the process of casting a spell. \
            The creature makes a Constitution saving throw. On a failed save, the \
            spell dissipates with no effect, and the action, Bonus Action, or \
            Reaction used to cast it is wasted. If that spell was cast with a spell \
            slot, the slot isn’t expended."
            .to_string(),
        3,
        MagicSchool::Abjuration,
        ActionKind::Reaction {
            reaction: Arc::new(|game_state, reaction_data| {
                let reactor = reaction_data.reactor;
                let trigger_event = &reaction_data.event;
                let reaction_context = &reaction_data.context;

                let trigger_action = match &trigger_event.kind {
                    EventKind::ActionRequested { action } => action,
                    EventKind::ReactionRequested { reaction } => &ActionData::from(reaction),
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
                            EventKind::D20CheckResolved(_, result_kind, _) => {
                                match result_kind {
                                    D20ResultKind::SavingThrow { result, .. } => {
                                        let result: ReactionResult = if result.success {
                                            // Successful save, Counterspell fails
                                            ReactionResult::NoEffect
                                        } else {
                                            // Spell slots are not consumed by Counterspell
                                            let mut resources_refunded = ResourceAmountMap::new();
                                            resources_refunded.insert(
                                                ResourceId::from_str("resource.spell_slot"),
                                                trigger_action
                                                    .resource_cost
                                                    .get(&ResourceId::from_str(
                                                        "resource.spell_slot",
                                                    ))
                                                    .cloned()
                                                    .unwrap(),
                                            );
                                            // Failed save, Counterspell succeeds
                                            ReactionResult::CancelEvent {
                                                event: trigger_event.clone().into(),
                                                resources_refunded,
                                            }
                                        };

                                        CallbackResult::Event(Event::action_performed_event(
                                            &game_state,
                                            reactor,
                                            &COUNTERSPELL_ID.clone().into(),
                                            &reaction_context,
                                            &ResourceAmountMap::from([(
                                                ResourceId::from_str("resource.reaction"),
                                                ResourceAmount::Flat(1),
                                            )]),
                                            trigger_action.actor,
                                            ActionKindResult::Reaction { result },
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
            ResourceId::from_str("resource.reaction"),
            ResourceAmount::Flat(1),
        )]),
        Arc::new(|_, _, _| TargetingContext {
            kind: TargetingKind::Single,
            range: TargetingRange::new::<foot>(60.0),
            require_line_of_sight: true,
            allowed_targets: EntityFilter::not_dead(),
        }),
        Some(Arc::new(|reactor, trigger_event| {
            let trigger_action = match &trigger_event.kind {
                EventKind::ActionRequested { action } => action,
                EventKind::ReactionRequested { reaction } => &ActionData::from(reaction),
                _ => return false,
            };
            if reactor == trigger_action.actor {
                // Cannot counterspell yourself
                return false;
            }
            // TODO: Can we just counterspell spells of any level?
            match &trigger_action.context {
                ActionContext::Spell { .. } => true,
                _ => false,
            }
        })),
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
