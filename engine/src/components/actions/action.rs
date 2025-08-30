use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    sync::Arc,
};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::targeting::{TargetTypeInstance, TargetingContext},
        d20::D20CheckResult,
        damage::{
            AttackRoll, AttackRollResult, DamageMitigationResult, DamageRoll, DamageRollResult,
        },
        dice::{DiceSetRoll, DiceSetRollResult},
        health::life_state::LifeState,
        id::{ActionId, EffectId, EntityIdentifier, ResourceId},
        items::equipment::{armor::ArmorClass, slots::EquipmentSlot},
        resource::{RechargeRule, ResourceCostMap, ResourceError, ResourceMap},
        saving_throw::{self, SavingThrowDC, SavingThrowSet},
        spells::spellbook::Spellbook,
    },
    systems::{self},
};

/// Represents the context in which an action is performed.
/// This can be used to determine the type of action (e.g. weapon, spell, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionContext {
    // TODO: Not sure if Weapon needs more info?
    Weapon {
        slot: EquipmentSlot,
    },
    /// When casting a spell it is important to know the spell level, since
    /// most spells have different effects based on the level at which they are cast.
    /// For example, Fireball deals more damage when cast at a higher level.
    Spell {
        level: u8,
    },
    // TODO: Not sure if Other is needed
    Other,
}

/// Represents the kind of action that can be performed.
#[derive(Clone)]
pub enum ActionKind {
    /// Actions that deal unconditional damage. Is this only Magic Missile?
    UnconditionalDamage {
        damage: Arc<dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync>,
    },
    /// Actions that require an attack roll to hit a target, and deal damage on hit.
    /// Some actions may have a damage roll on a failed attack roll (e.g. Acid Arrow)
    AttackRollDamage {
        attack_roll: Arc<dyn Fn(&World, Entity, &ActionContext) -> AttackRoll + Send + Sync>,
        damage: Arc<dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync>,
        damage_on_failure:
            Option<Arc<dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync>>,
    },
    /// Actions that require a saving throw to avoid or reduce damage.
    /// Most of the time, these actions will deal damage on a failed save,
    /// and half damage on a successful save.
    SavingThrowDamage {
        // TODO: Is action context ever relevant for saving throws?
        saving_throw: Arc<dyn Fn(&World, Entity, &ActionContext) -> SavingThrowDC + Send + Sync>,
        half_damage_on_save: bool,
        damage: Arc<dyn Fn(&World, Entity, &ActionContext) -> DamageRoll + Send + Sync>,
    },
    /// Actions that apply an effect to a target without requiring an attack roll or
    /// saving throw. TODO: Not sure if this is actually needed, since most effects
    /// will require either an attack roll or a saving throw.
    UnconditionalEffect { effect: EffectId },
    /// Actions that require a saving throw to avoid or reduce an effect.
    SavingThrowEffect {
        saving_throw: Arc<dyn Fn(&World, Entity, &ActionContext) -> SavingThrowDC + Send + Sync>,
        effect: EffectId,
    },
    /// Actions that apply a beneficial effect to a target, and therefore do not require
    /// an attack roll or saving throw (e.g. Bless, Shield of Faith).
    BeneficialEffect { effect: EffectId },
    /// Actions that heal a target. These actions do not require an attack roll or saving throw.
    /// They simply heal the target for a certain amount of hit points.
    Healing {
        heal: Arc<dyn Fn(&World, Entity, &ActionContext) -> DiceSetRoll + Send + Sync>,
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
    Composite { actions: Vec<ActionKind> },
    /// Custom actions can have any kind of effect, including damage, healing, or utility.
    /// Please note that this should only be used for actions that don't fit into the
    /// standard categories.
    Custom(Arc<dyn Fn(&World, Entity, &ActionContext) -> ActionKindResult + Send + Sync>),
}

/// The result of applying an action to a target.
/// This is the final result of the action, which includes any damage dealt,
/// effects applied, or healing done.
#[derive(Debug, Clone)]
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
        damage_roll: DamageRollResult,
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
    Custom {
        // TODO: Add more fields as needed for custom spells
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactionKind {
    ModifyAction {
        action: ActionId,
        // TODO: implement
        // modification:
        //     Arc<dyn Fn(&World, Entity, &ActionContext) -> ActionKindSnapshot + Send + Sync>,
    },
    NewAction {
        action: ActionId,
        context: ActionContext,
        targets: Vec<Entity>,
    },
    CancelAction {
        action: ActionId,
        context: ActionContext,
        targets: Vec<Entity>,
        consume_resources: bool,
    },
}

#[derive(Clone)]
pub struct Action {
    pub id: ActionId,
    pub kind: ActionKind,
    pub targeting: Arc<dyn Fn(&World, Entity, &ActionContext) -> TargetingContext + Send + Sync>,
    /// e.g. Action, Bonus Action, Reaction
    pub resource_cost: HashMap<ResourceId, u8>,
    /// Optional cooldown for the action
    pub cooldown: Option<RechargeRule>,
    /// If the action is a reaction, this will describe what triggers the reaction.
    /// * World: The game world in which the action is being performed.
    /// * (first)  Entity: The entity that is performing the reaction.
    /// * (second) Entity: The entity that is performing the action which triggers
    ///   the reaction.
    /// * ActionId: The ID of the action that is being performed.
    /// * ActionContext: The context in which the action is being performed.
    /// * &\[Entity\]: The targets of the action.
    // TODO: Struct with a nice name instead of all these fields?
    pub reaction_trigger: Option<
        Arc<
            dyn Fn(
                    &World,
                    Entity,
                    Entity,
                    &ActionId,
                    &ActionContext,
                    &[Entity],
                ) -> Option<ReactionKind>
                + Send
                + Sync,
        >,
    >,
}

/// Represents the result of performing an action on a single target. For actions that affect multiple targets,
/// multiple `ActionResult` instances can be collected.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub target: TargetTypeInstance,
    pub result: ActionKindResult,
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
    fn all_actions(&self) -> ActionMap;

    /// Returns a collection of available actions for the character. i.e. actions
    /// that can be performed at the current time.
    fn available_actions(&self) -> ActionMap;
}

impl ActionKind {
    pub fn perform(
        &self,
        world: &mut World,
        performer: Entity,
        context: &ActionContext,
        target: Entity,
    ) -> ActionKindResult {
        match self {
            ActionKind::UnconditionalDamage { .. }
            | ActionKind::AttackRollDamage { .. }
            | ActionKind::SavingThrowDamage { .. } => {
                systems::health::damage(world, performer, target, self, context)
            }

            ActionKind::UnconditionalEffect { effect } => ActionKindResult::UnconditionalEffect {
                effect: effect.clone(),
                applied: true, // TODO: Unconditional effects are always applied
            },

            ActionKind::SavingThrowEffect {
                saving_throw,
                effect,
            } => {
                let saving_throw = saving_throw(world, performer, context);
                ActionKindResult::SavingThrowEffect {
                    saving_throw: saving_throw.clone(),
                    effect: effect.clone(),
                    applied: systems::helpers::get_component::<SavingThrowSet>(world, target)
                        .check_dc(&saving_throw, world, target)
                        .success,
                }
            }

            ActionKind::BeneficialEffect { effect } => {
                systems::effects::add_effect(world, target, effect);
                ActionKindResult::BeneficialEffect {
                    effect: effect.clone(),
                    applied: true, // TODO: Beneficial effects are always applied?
                }
            }

            ActionKind::Healing { heal } => {
                let healing = heal(world, performer, context).roll();
                let new_life_state = systems::health::heal(world, target, healing.subtotal as u32);
                ActionKindResult::Healing {
                    healing,
                    new_life_state,
                }
            }

            ActionKind::Utility { .. } => ActionKindResult::Utility,

            ActionKind::Composite { actions } => ActionKindResult::Composite {
                actions: actions
                    .iter()
                    .map(|a| a.perform(world, performer, context, target))
                    .collect(),
            },

            ActionKind::Custom(custom) => custom(world, target, context),
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
            ActionKind::Custom(_) => write!(f, "CustomAction"),
        }
    }
}

impl Action {
    pub fn perform(
        &mut self,
        world: &mut World,
        performer: Entity,
        context: &ActionContext,
        targets: &[Entity],
    ) -> Vec<ActionResult> {
        // TODO: Not a fan of having to clone to avoid borrowing issues, but
        // hopefully since most of the effect just have a no-op as their
        // on_action component it'll be cheap to clone
        let hooks: Vec<_> = systems::effects::effects(world, performer)
            .iter()
            .filter_map(|effect| Some(effect.on_action.clone()))
            .collect();

        for hook in hooks {
            hook(world, performer, self, context);
        }

        // TODO: Resource might error?
        let _ = self.spend_resources(world, performer, context);

        targets
            .iter()
            .map(|target| ActionResult {
                target: TargetTypeInstance::Entity(EntityIdentifier::from_world(world, *target)),
                result: self.kind.perform(world, performer, context, *target),
            })
            .collect()
    }

    fn spend_resources(
        &self,
        world: &mut World,
        entity: Entity,
        context: &ActionContext,
    ) -> Result<(), ResourceError> {
        let mut resource_cost = self.resource_cost.clone();
        for effects in systems::effects::effects(world, entity).iter() {
            (effects.on_resource_cost)(world, entity, context, &mut resource_cost);
        }

        for (resource, amount) in &resource_cost {
            let mut resources = systems::helpers::get_component_mut::<ResourceMap>(world, entity);
            if let Some(resource) = resources.get_mut(resource) {
                resource.spend(*amount)?;
            }
        }
        // TODO: Not really a fan of this special treatment for spell slots
        match context {
            ActionContext::Spell { level } => {
                systems::helpers::get_component_mut::<Spellbook>(world, entity)
                    .use_spell_slot(*level);
            }
            _ => {
                // Other action contexts might not require resource spending
            }
        }
        Ok(())
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

    pub fn resource_cost(&self) -> &HashMap<ResourceId, u8> {
        &self.resource_cost
    }

    pub fn resource_cost_mut(&mut self) -> &mut HashMap<ResourceId, u8> {
        &mut self.resource_cost
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("resource_cost", &self.resource_cost)
            .finish()
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        // TODO: For now we just assume actions are equal if their IDs are the same.
        self.id == other.id
    }
}

// TODO: Combine these two?
pub type ActionMap = HashMap<ActionId, (Vec<ActionContext>, ResourceCostMap)>;

pub type ActionCooldownMap = HashMap<ActionId, RechargeRule>;

pub type ReactionSet = HashSet<ActionId>;
