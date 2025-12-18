use std::{
    fmt::Debug,
    str::FromStr,
    sync::{Arc, RwLock},
};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionContext,
        damage::{DamageRollResult, DamageSource},
        dice::{DiceSet, DiceSetRoll},
        id::{ActionId, ResourceId},
        items::equipment::loadout::Loadout,
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

/// Thread-safe shared wrapper for script-exposed data. The primary purpose is to
/// allow the scripts to mutate data without cloning it back and forth, e.g.
/// exposing a resource cost map that the script can modify in place.
///
/// However, note that sometimes we want to expose data as immutable (read-only).
/// This is due to the following reasons:
///     1. There are different types of scripts, and not all of them are supposed
///        to modify data. For example, a reaction trigger script should only inspect
///        the event which triggerd it, and then decide whether to react or not. It
///        should not be able to modify the event data itself.
///     2. To avoid duplicating structs/enums for mutable vs immutable variants,
///        we use a single `ScriptShared<T>` type with a flag indicating mutability.
///        This simplifies the API and reduces code duplication.
/// The `mutable` flag indicates whether the data can be mutated via `write()`.
/// Only data that has been taken with `take_from()` is intended to be mutable.
#[derive(Debug, Clone)]
pub struct ScriptShared<T> {
    inner: Arc<RwLock<T>>,
    mutable: bool,
}

impl<T> ScriptShared<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            mutable: false,
        }
    }

    /// Moves the value out of `target` and replaces it with `T::default()`.
    pub fn take_from(target: &mut T) -> Self
    where
        T: Default,
    {
        Self {
            inner: Arc::new(RwLock::new(std::mem::take(target))),
            mutable: true,
        }
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    /// Moves the value out of this shared wrapper.
    /// Panics if the handle was cloned and is still shared.
    pub fn into_inner(self) -> T
    where
        T: Debug,
    {
        Arc::try_unwrap(self.inner)
            .expect("Shared value leaked (extra Arc clones exist)")
            .into_inner()
            .expect("RwLock poisoned")
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.inner.read().expect("RwLock poisoned")
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        if !self.mutable {
            panic!("Attempted to get mutable access to an immutable ScriptShared value");
        }
        self.inner.write().expect("RwLock poisoned")
    }
}

macro_rules! impl_script_shared_methods {
    ($wrapper:ty, $inner:ty) => {
        impl $wrapper {
            pub fn new(value: $inner) -> Self {
                Self {
                    inner: ScriptShared::new(value),
                }
            }

            pub fn take_from(target: &mut $inner) -> Self {
                Self {
                    inner: ScriptShared::take_from(target),
                }
            }

            pub fn into_inner(self) -> $inner {
                self.inner.into_inner()
            }
        }
    };
}

macro_rules! impl_take_replace_world {
    ($wrapper:ty, $inner:ty) => {
        impl $wrapper {
            pub fn new_from_world(world: &World, entity: Entity) -> Self {
                let component = systems::helpers::get_component_clone::<$inner>(world, entity);
                Self::new(component)
            }

            pub fn take_from_world(world: &mut World, entity: Entity) -> Self {
                if let Ok(component) = world.query_one_mut::<&mut $inner>(entity) {
                    Self::take_from(component)
                } else {
                    panic!(
                        "Entity {:?} does not have component {}",
                        entity,
                        std::any::type_name::<$inner>()
                    );
                }
            }

            pub fn replace_in_world(self, world: &mut World, entity: Entity) {
                if let Ok(component) = world.query_one_mut::<&mut $inner>(entity) {
                    *component = self.into_inner();
                }
            }
        }
    };
}

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
pub struct ScriptDamageRollResult {
    inner: ScriptShared<DamageRollResult>,
}

impl ScriptDamageRollResult {
    pub fn source(&self) -> DamageSource {
        self.inner.read().source.clone()
    }

    pub fn clamp_damage_dice_min(&mut self, minimum_roll: u32) {
        let mut inner = self.inner.write();

        for component in &mut inner.components {
            // This assumes DamageComponentResult has `result: DiceSetRollResult` etc.
            for roll in &mut component.result.rolls {
                if *roll < minimum_roll {
                    *roll = minimum_roll;
                }
            }
            component.result.recalculate_total();
        }

        inner.recalculate_total();
    }
}

impl_script_shared_methods!(ScriptDamageRollResult, DamageRollResult);

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

#[derive(Clone)]
pub struct ScriptResourceCost {
    inner: ScriptShared<ResourceAmountMap>,
}

impl ScriptResourceCost {
    pub fn costs_resource(&self, resource_id: &ResourceId) -> bool {
        self.inner.read().contains_key(resource_id)
    }

    pub fn replace_resource(&mut self, from: &ResourceId, to: &ResourceId, amount: ResourceAmount) {
        let mut cost = self.inner.write();

        if let Some(from_amount) = cost.get(from) {
            if &amount >= from_amount {
                cost.remove(from);
            } else {
                cost.entry(from.clone())
                    .and_modify(|e| *e -= amount.clone());
            }
        }

        cost.entry(to.clone())
            .and_modify(|e| *e += amount.clone())
            .or_insert(amount);
    }
}

impl_script_shared_methods!(ScriptResourceCost, ResourceAmountMap);

impl From<&ResourceAmountMap> for ScriptResourceCost {
    fn from(cost: &ResourceAmountMap) -> Self {
        ScriptResourceCost::new(cost.clone())
    }
}

/// Script-facing view of an action (or a reaction treated as an action).
#[derive(Clone)]
pub struct ScriptActionView {
    pub action_id: String,
    pub actor: Entity,
    pub action_context: ScriptActionContext,
    pub resource_cost: ScriptResourceCost,
}

impl ScriptActionView {
    pub fn new(
        action_id: &ActionId,
        actor: Entity,
        action_context: &ActionContext,
        resource_cost: ScriptResourceCost,
    ) -> Self {
        ScriptActionView {
            action_id: action_id.to_string(),
            actor,
            action_context: ScriptActionContext::from(action_context),
            resource_cost,
        }
    }
}

impl From<&ActionData> for ScriptActionView {
    fn from(action: &ActionData) -> Self {
        ScriptActionView {
            action_id: action.action_id.to_string(),
            actor: action.actor,
            action_context: ScriptActionContext::from(&action.context),
            resource_cost: ScriptResourceCost::new(action.resource_cost.clone()),
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
    pub inner: ScriptShared<ResourceMap>,
}

impl ScriptResourceView {
    pub fn can_afford_resource(&self, resource_id: &ResourceId, amount: &ResourceAmount) -> bool {
        return self.inner.read().can_afford(resource_id, amount);
    }

    pub fn add_resource(&mut self, resource_id: &ResourceId, amount: &ResourceAmount) {
        self.inner.write().add(
            resource_id.clone(),
            ResourceBudgetKind::from(amount.clone()),
            true,
        );
    }
}

impl_script_shared_methods!(ScriptResourceView, ResourceMap);
impl_take_replace_world!(ScriptResourceView, ResourceMap);

/// Snapshot of an entity for scripts to inspect.
#[derive(Debug, Clone)]
pub struct ScriptEntityView {
    pub entity: ScriptEntity,
    pub resources: ScriptResourceView,
    pub loadout: ScriptLoadoutView,
    // Add more fields as needed
}

impl ScriptEntityView {
    pub fn new_from_world(world: &World, entity: Entity) -> Self {
        ScriptEntityView {
            entity: ScriptEntity::from(entity),
            resources: ScriptResourceView::new_from_world(world, entity),
            loadout: ScriptLoadoutView::from(&*systems::loadout::loadout(world, entity)),
        }
    }

    pub fn take_from_world(world: &mut World, entity: Entity) -> Self {
        ScriptEntityView {
            entity: ScriptEntity::from(entity),
            resources: ScriptResourceView::take_from_world(world, entity),
            loadout: ScriptLoadoutView::from(&*systems::loadout::loadout(world, entity)),
        }
    }

    pub fn replace_in_world(self, world: &mut World) {
        let entity: Entity = self.entity.clone().into();
        self.resources.replace_in_world(world, entity);
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
