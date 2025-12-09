use std::str::FromStr;

use hecs::Entity;

use crate::components::dice::{DiceSet, DiceSetRoll};
use crate::components::modifier::{ModifierSet, ModifierSource};
use crate::components::{actions::action::ActionContext, id::ResourceId};
use crate::engine::event::{ActionData, Event, EventKind, ReactionData};
use crate::registry::serialize::parser::Evaluable;
use crate::registry::serialize::{
    d20::SavingThrowProvider,
    parser::{DiceExpression, IntExpression},
    variables::PARSER_VARIABLES,
};
use crate::systems::d20::{D20CheckDCKind, D20ResultKind};

// Internally we keep using hecs::Entity in the API layer.
// Each backend (Rhai, Lua...) can decide how to represent it (e.g. integer).
pub type ScriptEntity = Entity;

/// What the trigger function logically receives.
#[derive(Clone)]
pub struct ReactionTriggerContext {
    pub reactor: ScriptEntity,
    pub event: Event,
}

/// What the body function logically receives (you can extend later).
#[derive(Clone)]
pub struct ReactionBodyContext {
    // pub reactor: ScriptEntity,
    pub reaction_data: ReactionData,
}

#[derive(Clone)]
pub struct ScriptD20CheckDCKind {
    // minimal content; you can refine it as needed
    pub label: String,
}

impl ScriptD20CheckDCKind {
    pub fn from(dc_kind: &D20CheckDCKind) -> Self {
        ScriptD20CheckDCKind {
            label: match dc_kind {
                D20CheckDCKind::SavingThrow(_) => "SavingThrow".to_string(),
                D20CheckDCKind::Skill(_) => "Skill".to_string(),
                D20CheckDCKind::AttackRoll(_, _) => "AttackRoll".to_string(),
            },
        }
    }
}

#[derive(Clone)]
pub struct ScriptD20Result {
    pub total: u32,
    pub kind: ScriptD20CheckDCKind,
    pub is_success: bool,
}

impl ScriptD20Result {
    pub fn from(result_kind: &D20ResultKind, dc_kind: &D20CheckDCKind) -> Self {
        let result = match result_kind {
            D20ResultKind::Skill { result, .. } | D20ResultKind::SavingThrow { result, .. } => {
                result
            }
            D20ResultKind::AttackRoll { result } => &result.roll_result,
        };
        ScriptD20Result {
            total: result.total(),
            kind: ScriptD20CheckDCKind::from(dc_kind),
            is_success: result_kind.is_success(dc_kind),
        }
    }
}

/// High-level event view that scripts can work with.
#[derive(Clone)]
pub enum ScriptEventView {
    D20CheckPerformed(D20CheckPerformedView),
    ActionRequested(ScriptActionView),
    // later:
    // DamageRollPerformed(DamageView),
    // ...
}

impl ScriptEventView {
    pub fn from_event(event: &Event) -> Option<Self> {
        match &event.kind {
            EventKind::D20CheckPerformed(performer, result_kind, dc_kind) => {
                Some(ScriptEventView::D20CheckPerformed(
                    D20CheckPerformedView::from_parts(*performer, result_kind, dc_kind),
                ))
            }

            // A direct action request
            EventKind::ActionRequested { action } => Some(ScriptEventView::ActionRequested(
                ScriptActionView::from_action_data(action),
            )),

            // A reaction request that is itself an action (e.g. reaction spell)
            EventKind::ReactionRequested { reaction } => {
                let action = ActionData::from(reaction);
                Some(ScriptEventView::ActionRequested(
                    ScriptActionView::from_action_data(&action),
                ))
            }

            _ => None, // extend with more variants as needed
        }
    }
}

/// View of a "D20CheckPerformed" event.
#[derive(Clone)]
pub struct D20CheckPerformedView {
    pub performer: ScriptEntity,
    pub result: ScriptD20Result,
    pub dc_kind: ScriptD20CheckDCKind,
}

impl D20CheckPerformedView {
    pub fn from_parts(
        performer: Entity,
        result_kind: &D20ResultKind,
        dc_kind: &D20CheckDCKind,
    ) -> Self {
        D20CheckPerformedView {
            performer,
            result: ScriptD20Result::from(result_kind, dc_kind),
            dc_kind: ScriptD20CheckDCKind::from(dc_kind),
        }
    }
}

/// Script-facing view of an action (or a reaction treated as an action).
#[derive(Clone)]
pub struct ScriptActionView {
    pub action_id: String,
    pub actor: ScriptEntity,
    pub is_spell: bool,
    // later: spell_id, spell_level, school, tags, etc.
}

impl ScriptActionView {
    pub fn from_action_data(action: &ActionData) -> Self {
        ScriptActionView {
            action_id: action.action_id.to_string(),
            actor: action.actor,
            is_spell: matches!(action.context, ActionContext::Spell { .. }),
        }
    }
}

/// Which entity are we talking about? We keep this abstract so scripts do
/// not need entity IDs, only roles.
#[derive(Clone)]
pub enum ScriptEntityRole {
    Reactor, // the creature using the reaction
    TriggerActor, // the actor of the triggering action
             // later: Specific(Entity), Target, etc.
}

/// Which event are we referring to?
#[derive(Clone)]
pub enum ScriptEventRef {
    TriggerEvent, // the event that caused this reaction
                  // later: SomeOtherEventById(EventId) if needed
}

/// How to compute a saving throw DC.
#[derive(Clone)]
pub struct ScriptSavingThrowSpec {
    /// Entity role where the saving throw originates
    pub entity: ScriptEntityRole,
    pub saving_throw: SavingThrowProvider,
}

/// Bonus to apply to a D20 roll.
#[derive(Clone)]
pub enum ScriptD20Bonus {
    Flat(IntExpression),
    Dice(DiceExpression),
}

impl ScriptD20Bonus {
    pub fn evaluate(
        &self,
        world: &hecs::World,
        entity: Entity,
        action_context: &ActionContext,
    ) -> i32 {
        match self {
            ScriptD20Bonus::Flat(expr) => expr
                .evaluate(world, entity, action_context, &PARSER_VARIABLES)
                .unwrap(),
            ScriptD20Bonus::Dice(expr) => {
                let (num_dice, size, modifier) = expr
                    .evaluate(world, entity, action_context, &PARSER_VARIABLES)
                    .unwrap();

                DiceSetRoll {
                    dice: DiceSet::from_str(format!("{}d{}", num_dice, size).as_str()).unwrap(),
                    modifiers: ModifierSet::from(ModifierSource::Base, modifier),
                }
                .roll()
                .subtotal
            }
        }
    }
}

/// Plan/description of what the reaction actually does.
/// This is interpreted by Rust; scripts only *describe* the behaviour.
#[derive(Clone)]
pub enum ScriptReactionPlan {
    /// Do nothing.
    None,

    /// Execute multiple steps in order.
    Sequence(Vec<ScriptReactionPlan>),

    /// Add a flat modifier to the most recent D20 roll for this event.
    ModifyD20Result { bonus: ScriptD20Bonus },

    /// Reroll the most recent D20 roll for this event with an optional modifier.
    /// Can also be set to force using the new roll.
    RerollD20Result {
        bonus: Option<ScriptD20Bonus>,
        force_use_new: bool,
    },

    /// Ask an entity to make a saving throw against a DC.
    /// Then branch into `on_success` or `on_failure`.
    RequireSavingThrow {
        target: ScriptEntityRole,
        dc: ScriptSavingThrowSpec,
        on_success: Box<ScriptReactionPlan>,
        on_failure: Box<ScriptReactionPlan>,
    },

    /// Cancel a specific event (usually the trigger) and maybe refund resources.
    CancelEvent {
        event: ScriptEventRef,
        resources_to_refund: Vec<ResourceId>, // e.g. spell slots
    },
}
