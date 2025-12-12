use std::{collections::HashMap, str::FromStr};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionContext,
        dice::{DiceSet, DiceSetRoll},
        id::{ActionId, ResourceId},
        items::equipment::{armor::ArmorType, loadout::Loadout, weapon::WeaponKind},
        modifier::{ModifierSet, ModifierSource},
        resource::{ResourceAmount, ResourceAmountMap, ResourceBudgetKind, ResourceMap},
    },
    engine::event::{ActionData, Event, EventKind, ReactionData},
    registry::serialize::{
        d20::SavingThrowProvider,
        parser::{DiceExpression, Evaluable, IntExpression, Parser},
        variables::PARSER_VARIABLES,
    },
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEntity {
    pub id: u64,
}

impl From<Entity> for ScriptEntity {
    fn from(entity: Entity) -> Self {
        ScriptEntity {
            id: u64::from(entity.to_bits()),
        }
    }
}

impl Into<Entity> for ScriptEntity {
    fn into(self) -> Entity {
        Entity::from_bits(self.id).unwrap()
    }
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

// TODO
pub struct ScriptDiceRollResult {
    pub subtotal: i32,
    pub die_results: Vec<u32>,
}

pub struct ScriptDamageRollResult {
    pub total_damage: i32,
    // add more fields as needed
}

/// High-level event view that scripts can work with.
#[derive(Clone)]
pub enum ScriptEventView {
    D20CheckPerformed(ScriptD20CheckPerformedView),
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
                    ScriptD20CheckPerformedView::from_parts(*performer, result_kind, dc_kind),
                ))
            }

            // A direct action request
            EventKind::ActionRequested { action } => Some(ScriptEventView::ActionRequested(
                ScriptActionView::from(action),
            )),

            // A reaction request that is itself an action (e.g. reaction spell)
            EventKind::ReactionRequested { reaction } => {
                let action = ActionData::from(reaction);
                Some(ScriptEventView::ActionRequested(ScriptActionView::from(
                    &action,
                )))
            }

            _ => None, // extend with more variants as needed
        }
    }

    pub fn is_d20_check_performed(&self) -> bool {
        matches!(self, ScriptEventView::D20CheckPerformed(_))
    }

    pub fn as_d20_check_performed(&self) -> &ScriptD20CheckPerformedView {
        if let ScriptEventView::D20CheckPerformed(view) = self {
            view
        } else {
            panic!("Not a D20CheckPerformed event view");
        }
    }

    pub fn is_action(&self) -> bool {
        matches!(self, ScriptEventView::ActionRequested(_))
    }

    pub fn as_action(&self) -> &ScriptActionView {
        if let ScriptEventView::ActionRequested(view) = self {
            view
        } else {
            panic!("Not an ActionRequested event view");
        }
    }
}

/// View of a "D20CheckPerformed" event.
#[derive(Clone)]
pub struct ScriptD20CheckPerformedView {
    pub performer: ScriptEntity,
    pub result: ScriptD20Result,
    pub dc_kind: ScriptD20CheckDCKind,
}

impl ScriptD20CheckPerformedView {
    pub fn from_parts(
        performer: Entity,
        result_kind: &D20ResultKind,
        dc_kind: &D20CheckDCKind,
    ) -> Self {
        ScriptD20CheckPerformedView {
            performer: ScriptEntity::from(performer),
            result: ScriptD20Result::from(result_kind, dc_kind),
            dc_kind: ScriptD20CheckDCKind::from(dc_kind),
        }
    }
}

#[derive(Clone)]
pub struct ScriptActionContext {
    pub inner: ActionContext,
}

impl ScriptActionContext {
    pub fn is_spell(&self) -> bool {
        matches!(self.inner, ActionContext::Spell { .. })
    }

    pub fn is_weapon_attack(&self) -> bool {
        matches!(self.inner, ActionContext::Weapon { .. })
    }
}

impl From<&ActionContext> for ScriptActionContext {
    fn from(context: &ActionContext) -> Self {
        ScriptActionContext {
            inner: context.clone(),
        }
    }
}

/// Resource cost passed to/from scripts.
/// Tracks if it was modified inside the script, so we know whether to update the
/// original cost outside the script.
#[derive(Clone)]
pub struct ScriptResourceCost {
    pub inner: ResourceAmountMap,
    pub modified: bool,
}

impl ScriptResourceCost {
    pub fn costs_resource(&self, resource_id: &ResourceId) -> bool {
        self.inner.contains_key(resource_id)
    }

    pub fn replace_resource(&mut self, from: &ResourceId, to: &ResourceId, amount: ResourceAmount) {
        if let Some(from_amount) = self.inner.get(from) {
            if &amount >= from_amount {
                self.inner.remove(from);
            } else {
                self.inner
                    .entry(from.clone())
                    .and_modify(|e| *e -= amount.clone());
            }
        }

        self.inner
            .entry(to.clone())
            .and_modify(|e| *e += amount.clone())
            .or_insert(amount);

        self.modified = true;
    }

    pub fn apply_modifications(&self, cost: &mut ResourceAmountMap) {
        if self.modified {
            *cost = self.inner.clone();
        }
    }
}

impl From<&ResourceAmountMap> for ScriptResourceCost {
    fn from(cost: &ResourceAmountMap) -> Self {
        ScriptResourceCost {
            inner: cost.clone(),
            modified: false,
        }
    }
}

/// Script-facing view of an action (or a reaction treated as an action).
#[derive(Clone)]
pub struct ScriptActionView {
    pub action_id: String,
    pub actor: Entity,
    pub action_context: ScriptActionContext,
    pub resource_cost: ScriptResourceCost,
    // later: spell_id, spell_level, school, tags, etc.
}

impl ScriptActionView {
    pub fn new(
        action_id: &ActionId,
        actor: Entity,
        action_context: &ActionContext,
        resource_cost: &ResourceAmountMap,
    ) -> Self {
        ScriptActionView {
            action_id: action_id.to_string(),
            actor,
            action_context: ScriptActionContext::from(action_context),
            resource_cost: ScriptResourceCost::from(resource_cost),
        }
    }
}

impl From<&ActionData> for ScriptActionView {
    fn from(action: &ActionData) -> Self {
        ScriptActionView {
            action_id: action.action_id.to_string(),
            actor: action.actor,
            action_context: ScriptActionContext::from(&action.context),
            resource_cost: ScriptResourceCost::from(&action.resource_cost),
        }
    }
}

/// Which entity are we talking about? We keep this abstract so scripts do
/// not need entity IDs, only roles.
#[derive(Clone)]
pub enum ScriptEntityRole {
    /// The entity performing the action. For a reaction, this would be the
    /// entity which performed the event that triggered the reaction.
    Actor,
    /// The entity reacting to the event (only for reactions).
    Reactor,
    /// The target of the action/event.
    Target,
}

impl FromStr for ScriptEntityRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "actor" => Ok(ScriptEntityRole::Actor),
            "reactor" => Ok(ScriptEntityRole::Reactor),
            "target" => Ok(ScriptEntityRole::Target),
            _ => Err(format!("Unknown ScriptEntityRole: {}", s)),
        }
    }
}

/// Which event are we referring to?
#[derive(Clone)]
pub enum ScriptEventRef {
    TriggerEvent, // the event that caused this reaction
                  // later: SomeOtherEventById(EventId) if needed
}

/// How to compute a saving throw DC.
#[derive(Clone)]
pub struct ScriptSavingThrow {
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

impl FromStr for ScriptD20Bonus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(flat) = Parser::new(s).parse_int_expression() {
            Ok(ScriptD20Bonus::Flat(flat))
        } else if let Ok(expr) = Parser::new(s).parse_dice_expression() {
            Ok(ScriptD20Bonus::Dice(expr))
        } else {
            Err(format!("Invalid ScriptD20Bonus expression: {}", s))
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
        dc: ScriptSavingThrow,
        on_success: Box<ScriptReactionPlan>,
        on_failure: Box<ScriptReactionPlan>,
    },

    /// Cancel a specific event (usually the trigger) and maybe refund resources.
    CancelEvent {
        event: ScriptEventRef,
        resources_to_refund: Vec<ResourceId>, // e.g. spell slots
    },
}

/// Snapshot of a loadout for scripts to inspect.
#[derive(Debug, Clone)]
pub struct ScriptLoadoutView {
    pub loadout: Loadout,
}

impl From<&Loadout> for ScriptLoadoutView {
    fn from(loadout: &Loadout) -> Self {
        ScriptLoadoutView {
            loadout: loadout.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptResourceView {
    pub modified: bool,
    pub resources: ResourceMap,
}

impl From<&ResourceMap> for ScriptResourceView {
    fn from(resources: &ResourceMap) -> Self {
        ScriptResourceView {
            modified: false,
            resources: resources.clone(),
        }
    }
}

impl ScriptResourceView {
    pub fn can_afford_resource(&self, resource_id: &ResourceId, amount: &ResourceAmount) -> bool {
        return self.resources.can_afford(resource_id, amount);
    }

    pub fn add_resource(&mut self, resource_id: &ResourceId, amount: &ResourceAmount) {
        self.resources.add(
            resource_id.clone(),
            ResourceBudgetKind::from(amount.clone()),
            true,
        );
        self.modified = true;
    }

    pub fn apply_modifications(&self, resources: &mut ResourceMap) {
        if self.modified {
            *resources = self.resources.clone();
        }
    }
}

/// Snapshot of an entity for scripts to inspect.
#[derive(Debug, Clone)]
pub struct ScriptEntityView {
    // pub modified: bool,
    pub entity: ScriptEntity,
    pub resources: ScriptResourceView,
    pub loadout: ScriptLoadoutView,
    // Add more fields as needed
}

impl ScriptEntityView {
    pub fn from_world(world: &World, entity: Entity) -> Self {
        let resources = systems::helpers::get_component::<ResourceMap>(world, entity);
        let loadout = systems::loadout::loadout(world, entity);

        ScriptEntityView {
            // modified: false,
            entity: ScriptEntity::from(entity),
            resources: ScriptResourceView::from(&*resources),
            loadout: ScriptLoadoutView::from(&*loadout),
        }
    }

    pub fn apply_modifications(&self, world: &mut World) {
        let entity: Entity = self.entity.clone().into();
        self.resources.apply_modifications(
            &mut systems::helpers::get_component_mut::<ResourceMap>(world, entity),
        );
    }
}

/// What the trigger function logically receives.
#[derive(Clone)]
pub struct ScriptReactionTriggerContext {
    pub reactor: ScriptEntity,
    pub event: ScriptEventView,
}

/// What the body function logically receives (you can extend later).
/// TODO: Implement this properly - if it's even need at all?
#[derive(Clone)]
pub struct ScriptReactionBodyContext {
    pub reaction_data: ReactionData,
}
