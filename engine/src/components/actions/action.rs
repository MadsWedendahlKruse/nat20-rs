use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use hecs::{Entity, World};
use serde::Deserialize;
use tracing_subscriber::filter::targets;

use crate::{
    components::{
        actions::targeting::{TargetInstance, TargetingContext},
        d20::D20CheckResult,
        damage::{
            AttackRoll, AttackRollResult, DamageMitigationResult, DamageRoll, DamageRollResult,
        },
        dice::{DiceSetRoll, DiceSetRollResult},
        health::life_state::LifeState,
        id::{ActionId, EffectId, EntityIdentifier, IdProvider, ScriptId, SpellId},
        items::equipment::{armor::ArmorClass, slots::EquipmentSlot},
        modifier::ModifierSource,
        resource::{RechargeRule, ResourceAmountMap},
        saving_throw::SavingThrowDC,
        spells::spellbook::SpellSource,
    },
    engine::{
        event::{ActionData, Event},
        game_state::GameState,
    },
    registry::serialize::action::ActionDefinition,
    systems::{self, d20::D20CheckDCKind},
};

/// Represents the context in which an action is performed.
/// This can be used to determine the type of action (e.g. weapon, spell, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionContext {
    // TODO: Not sure if Weapon needs more info?
    Weapon {
        slot: EquipmentSlot,
    },
    Spell {
        id: SpellId,
        /// Having the source here allows us to track whether the spell is coming
        /// from a class, subclass, item, feat, etc., which is useful for determining
        /// e.g. spellcasting ability for spell save DCs and spell attack rolls.
        source: SpellSource,
        /// When casting a spell it is important to know the spell level, since
        /// most spells have different effects based on the level at which they are cast.
        /// For example, Fireball deals more damage when cast at a higher level.
        level: u8,
    },
    // TODO: Not sure if Other is needed
    Other,
}

pub type DamageFunction = dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync;
pub type AttackRollFunction =
    dyn Fn(&World, Entity, Entity, &ActionContext) -> AttackRoll + Send + Sync;
pub type SavingThrowFunction =
    dyn Fn(&World, Entity, &ActionContext) -> SavingThrowDC + Send + Sync;
pub type HealFunction = dyn Fn(&World, Entity, &ActionContext) -> DiceSetRoll + Send + Sync;

/// Represents the kind of action that can be performed.
#[derive(Clone)]
pub enum ActionKind {
    /// Actions that deal unconditional damage. Is this only Magic Missile?
    UnconditionalDamage {
        damage: Arc<DamageFunction>,
    },
    /// Actions that require an attack roll to hit a target, and deal damage on hit.
    /// Some actions may have a damage roll on a failed attack roll (e.g. Acid Arrow)
    AttackRollDamage {
        attack_roll: Arc<AttackRollFunction>,
        damage: Arc<DamageFunction>,
        damage_on_miss: Option<Arc<DamageFunction>>,
    },
    /// Actions that require a saving throw to avoid or reduce damage.
    /// Most of the time, these actions will deal damage on a failed save,
    /// and half damage on a successful save.
    SavingThrowDamage {
        // TODO: Is action context ever relevant for saving throws?
        saving_throw: Arc<SavingThrowFunction>,
        half_damage_on_save: bool,
        damage: Arc<DamageFunction>,
    },
    /// Actions that apply an effect to a target without requiring an attack roll or
    /// saving throw. TODO: Not sure if this is actually needed, since most effects
    /// will require either an attack roll or a saving throw.
    UnconditionalEffect {
        effect: EffectId,
    },
    /// Actions that require a saving throw to avoid or reduce an effect.
    SavingThrowEffect {
        saving_throw: Arc<SavingThrowFunction>,
        effect: EffectId,
    },
    /// Actions that apply a beneficial effect to a target, and therefore do not require
    /// an attack roll or saving throw (e.g. Bless, Shield of Faith).
    BeneficialEffect {
        effect: EffectId,
    },
    /// Actions that heal a target. These actions do not require an attack roll or saving throw.
    /// They simply heal the target for a certain amount of hit points.
    Healing {
        heal: Arc<HealFunction>,
    },
    /// Utility actions that do not deal damage or heal, but have some other effect.
    /// These actions may include buffs, debuffs, or other effects that do not fit into the
    /// other categories (e.g. teleportation, Knock, etc.).
    Utility {
        // E.g. Arcane Lock, Invisibility, etc.
        // Add hooks or custom closures as needed
    },
    /// A composite action that combines multiple actions into one.
    /// This can be used for actions that have multiple effects, such as a spell
    /// that deals damage and applies a beneficial effect.
    Composite {
        actions: Vec<ActionKind>,
    },

    Reaction {
        reaction: ScriptId,
    },
    /// Custom actions can have any kind of effect, including damage, healing, or utility.
    /// Please note that this should only be used for actions that don't fit into the
    /// standard categories.
    Custom(Arc<dyn Fn(&World, Entity, &ActionContext) + Send + Sync>),
}

/// The result of applying an action to a target.
/// This is the final result of the action, which includes any damage dealt,
/// effects applied, or healing done.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionKindResult {
    UnconditionalDamage {
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
        new_life_state: Option<LifeState>,
    },
    AttackRollDamage {
        attack_roll: AttackRollResult,
        /// Armor class of the target being attacked
        armor_class: ArmorClass,
        damage_roll: Option<DamageRollResult>,
        damage_taken: Option<DamageMitigationResult>,
        new_life_state: Option<LifeState>,
    },
    SavingThrowDamage {
        saving_throw_dc: SavingThrowDC,
        saving_throw_result: D20CheckResult,
        half_damage_on_save: bool,
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
        new_life_state: Option<LifeState>,
    },
    UnconditionalEffect {
        effect: EffectId,
        applied: bool,
    },
    SavingThrowEffect {
        saving_throw: SavingThrowDC,
        effect: EffectId,
        applied: bool,
    },
    BeneficialEffect {
        effect: EffectId,
        applied: bool,
    },
    Healing {
        healing: DiceSetRollResult,
        new_life_state: Option<LifeState>,
    },
    Utility,
    Composite {
        actions: Vec<ActionKindResult>,
    },
    Reaction {
        result: ReactionResult,
    },
    Custom {
        // TODO: Add more fields as needed for custom spells
    },
}

#[derive(Clone)]
pub enum ReactionResult {
    ModifyEvent {
        modification: Arc<dyn Fn(&World, &mut Event) + Send + Sync>,
    },
    CancelEvent {
        event: Box<Event>,
        resources_refunded: ResourceAmountMap,
    },
    NoEffect,
}

impl Debug for ReactionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReactionResult::ModifyEvent { .. } => write!(f, "ModifyEvent"),
            ReactionResult::CancelEvent {
                event,
                resources_refunded,
            } => f
                .debug_struct("CancelEvent")
                .field("event", &event.id)
                .field("resources_refunded", resources_refunded)
                .finish(),
            ReactionResult::NoEffect => write!(f, "NoEffect"),
        }
    }
}

impl PartialEq for ReactionResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReactionResult::ModifyEvent { .. }, ReactionResult::ModifyEvent { .. }) => true,
            (
                ReactionResult::CancelEvent { event: e1, .. },
                ReactionResult::CancelEvent { event: e2, .. },
            ) => e1.id == e2.id,
            (ReactionResult::NoEffect, ReactionResult::NoEffect) => true,
            _ => false,
        }
    }
}

pub type TargetingFunction =
    dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync;

#[derive(Clone, Deserialize)]
#[serde(from = "ActionDefinition")]
pub struct Action {
    pub id: ActionId,
    pub description: String,
    pub kind: ActionKind,
    pub targeting: Arc<TargetingFunction>,
    /// e.g. Action, Bonus Action, Reaction
    pub resource_cost: ResourceAmountMap,
    /// Optional cooldown for the action
    pub cooldown: Option<RechargeRule>,
    /// If the action is a reaction, this will describe what triggers the reaction.
    pub reaction_trigger: Option<ScriptId>,
}

/// Represents the result of performing an action on a single target. For actions
/// that affect multiple targets, multiple `ActionResult` instances can be collected.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionResult {
    pub performer: EntityIdentifier,
    pub target: TargetInstance,
    pub kind: ActionKindResult,
}

/// Represents a provider of actions, which can be used to retrieve available actions
/// from a character or other entity that can perform actions.
pub trait ActionProvider {
    // TODO: Should probably find a way to avoid rebuilding the action collection every time.

    /// Returns a collection of ALL possible actions for the character, including
    /// actions that are not currently available (e.g. on cooldown, out of resources, etc.).
    /// Each action is paired with its context, which provides additional information
    /// about how the action can be performed (e.g. weapon type, spell level, etc.)
    /// as well as the resource cost of the action.
    fn actions(&self) -> ActionMap;
}

impl ActionKind {
    pub fn perform(
        &self,
        game_state: &mut GameState,
        action_data: &ActionData,
        targets: &[Entity],
    ) {
        match self {
            ActionKind::UnconditionalDamage { .. }
            | ActionKind::AttackRollDamage { .. }
            | ActionKind::SavingThrowDamage { .. } => {
                for target in targets {
                    systems::health::damage(game_state, action_data, self, *target);
                }
            }

            ActionKind::UnconditionalEffect { effect } => {
                for target in targets {
                    systems::effects::add_effect(
                        &mut game_state.world,
                        *target,
                        effect,
                        &ModifierSource::Action(action_data.action_id.clone()),
                    );
                }
                let _ = game_state.process_event(Event::action_performed_event(
                    &game_state,
                    action_data,
                    targets
                        .iter()
                        .map(|target| {
                            (
                                *target,
                                ActionKindResult::UnconditionalEffect {
                                    effect: effect.clone(),
                                    applied: true, // TODO: Unconditional effects are always applied?
                                },
                            )
                        })
                        .collect(),
                ));
            }

            ActionKind::SavingThrowEffect {
                saving_throw,
                effect,
            } => {
                for target in targets {
                    let saving_throw =
                        saving_throw(&game_state.world, action_data.actor, &action_data.context);
                    let saving_throw_result = systems::d20::check(
                        game_state,
                        *target,
                        &D20CheckDCKind::SavingThrow(saving_throw),
                    );
                    todo!("Implement saving throw effect application logic (callbacks, etc.)");
                }
            }

            ActionKind::BeneficialEffect { effect } => {
                for target in targets {
                    systems::effects::add_effect(
                        &mut game_state.world,
                        *target,
                        effect,
                        &ModifierSource::Action(action_data.action_id.clone()),
                    );
                }
                let _ = game_state.process_event(Event::action_performed_event(
                    &game_state,
                    action_data,
                    targets
                        .iter()
                        .map(|target| {
                            (
                                *target,
                                ActionKindResult::BeneficialEffect {
                                    effect: effect.clone(),
                                    applied: true, // TODO: Beneficial effects are always applied?
                                },
                            )
                        })
                        .collect(),
                ));
            }

            ActionKind::Healing { heal } => {
                let mut results = Vec::new();
                for target in targets {
                    let healing =
                        heal(&game_state.world, action_data.actor, &action_data.context).roll();
                    let new_life_state = systems::health::heal(
                        &mut game_state.world,
                        *target,
                        healing.subtotal as u32,
                    );
                    results.push((
                        *target,
                        ActionKindResult::Healing {
                            healing,
                            new_life_state,
                        },
                    ));
                }
                let _ = game_state.process_event(Event::action_performed_event(
                    &game_state,
                    action_data,
                    results,
                ));
            }

            ActionKind::Utility { .. } => {
                let _ = game_state.process_event(Event::action_performed_event(
                    &game_state,
                    action_data,
                    targets
                        .iter()
                        .map(|target| (*target, ActionKindResult::Utility))
                        .collect(),
                ));
            }

            ActionKind::Composite { actions } => {
                for action in actions {
                    match action {
                        ActionKind::Reaction { .. } => {
                            // Assume this is being performed as part of a reaction
                            // TODO: Also seems like a bit of a hack
                            continue;
                        }
                        _ => action.perform(game_state, action_data, targets),
                    }
                }
            }

            ActionKind::Reaction { .. } => {
                panic!(
                    "ActionKind::Reaction should be performed via systems::actions::perform_reaction"
                );
            }

            ActionKind::Custom(custom) => {
                // custom(game_state.world, target, context)
                todo!("Custom actions are not yet implemented");
            }
        }
    }
}

impl Debug for ActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionKind::UnconditionalDamage { .. } => write!(f, "UnconditionalDamage"),
            ActionKind::AttackRollDamage { .. } => write!(f, "AttackRollDamage"),
            ActionKind::SavingThrowDamage { .. } => write!(f, "SavingThrowDamage"),
            ActionKind::UnconditionalEffect { effect } => {
                write!(f, "UnconditionalEffect({})", effect)
            }
            ActionKind::SavingThrowEffect { effect, .. } => {
                write!(f, "SavingThrowEffect({})", effect)
            }
            ActionKind::BeneficialEffect { effect } => write!(f, "BeneficialEffect({})", effect),
            ActionKind::Healing { .. } => write!(f, "Healing"),
            ActionKind::Utility { .. } => write!(f, "Utility"),
            ActionKind::Composite { actions } => write!(f, "Composite({:?})", actions),
            ActionKind::Reaction { .. } => write!(f, "Reaction"),
            ActionKind::Custom(_) => write!(f, "CustomAction"),
        }
    }
}

impl Action {
    /// Targets are very explicitly passed as a separate parameter here, since the
    /// targets in the `ActionData` can also be points, so prior to calling `perform`
    /// the targetted entities are resolved.
    pub fn perform(
        &mut self,
        game_state: &mut GameState,
        action_data: &ActionData,
        targets: &[Entity],
    ) {
        // TODO: Not a fan of having to clone to avoid borrowing issues, but
        // hopefully since most of the effect just have a no-op as their
        // on_action component it'll be cheap to clone
        let hooks: Vec<_> = systems::effects::effects(&game_state.world, action_data.actor)
            .iter()
            .filter_map(|effect| Some(effect.on_action.clone()))
            .collect();

        for hook in hooks {
            hook(&mut game_state.world, action_data);
        }

        self.kind.perform(game_state, action_data, targets);
    }

    pub fn id(&self) -> &ActionId {
        &self.id
    }

    pub fn kind(&self) -> &ActionKind {
        &self.kind
    }

    pub fn targeting(
        &self,
    ) -> &Arc<dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync> {
        &self.targeting
    }

    pub fn resource_cost(&self) -> &ResourceAmountMap {
        &self.resource_cost
    }

    pub fn resource_cost_mut(&mut self) -> &mut ResourceAmountMap {
        &mut self.resource_cost
    }
}

impl IdProvider for Action {
    type Id = ActionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("resource_cost", &self.resource_cost)
            .field("cooldown", &self.cooldown)
            .finish()
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        // TODO: For now we just assume actions are equal if their IDs are the same.
        self.id == other.id
    }
}

impl ActionResult {
    pub fn new(world: &World, performer: Entity, target: Entity, kind: ActionKindResult) -> Self {
        ActionResult {
            performer: EntityIdentifier::from_world(world, performer),
            target: TargetInstance::Entity(target),
            kind,
        }
    }
}

// TODO: Combine these two?
pub type ActionMap = HashMap<ActionId, Vec<(ActionContext, ResourceAmountMap)>>;

pub type ActionCooldownMap = HashMap<ActionId, RechargeRule>;

pub type ReactionSet = HashSet<ActionId>;
