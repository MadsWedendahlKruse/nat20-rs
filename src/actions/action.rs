use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::{
    actions::targeting::{TargetTypeInstance, TargetingContext},
    combat::damage::{
        AttackRoll, AttackRollResult, DamageMitigationResult, DamageRoll, DamageRollResult,
    },
    creature::character::Character,
    dice::dice::{DiceSetRoll, DiceSetRollResult},
    items::equipment::{equipment::HandSlot, weapon::WeaponType},
    registry,
    resources::resources::{RechargeRule, ResourceError},
    stats::saving_throw::SavingThrowDC,
    utils::id::{ActionId, EffectId, ResourceId},
};

/// Represents the context in which an action is performed.
/// This can be used to determine the type of action (e.g. weapon, spell, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionContext {
    // TODO: Not sure if Weapon needs more info?
    Weapon {
        weapon_type: WeaponType,
        hand: HandSlot,
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
        damage: Arc<dyn Fn(&Character, &ActionContext) -> DamageRoll + Send + Sync>,
    },
    /// Actions that require an attack roll to hit a target, and deal damage on hit.
    /// Some actions may have a damage roll on a failed attack roll (e.g. Acid Arrow)
    AttackRollDamage {
        attack_roll: Arc<dyn Fn(&Character, &ActionContext) -> AttackRoll + Send + Sync>,
        damage: Arc<dyn Fn(&Character, &ActionContext) -> DamageRoll + Send + Sync>,
        damage_on_failure:
            Option<Arc<dyn Fn(&Character, &ActionContext) -> DamageRoll + Send + Sync>>,
    },
    /// Actions that require a saving throw to avoid or reduce damage.
    /// Most of the time, these actions will deal damage on a failed save,
    /// and half damage on a successful save.
    SavingThrowDamage {
        // TODO: Is action context ever relevant for saving throws?
        saving_throw: Arc<dyn Fn(&Character, &ActionContext) -> SavingThrowDC + Send + Sync>,
        half_damage_on_save: bool,
        damage: Arc<dyn Fn(&Character, &ActionContext) -> DamageRoll + Send + Sync>,
    },
    /// Actions that apply an effect to a target without requiring an attack roll or
    /// saving throw. TODO: Not sure if this is actually needed, since most effects
    /// will require either an attack roll or a saving throw.
    UnconditionalEffect { effect: EffectId },
    /// Actions that require a saving throw to avoid or reduce an effect.
    SavingThrowEffect {
        saving_throw: Arc<dyn Fn(&Character, &ActionContext) -> SavingThrowDC + Send + Sync>,
        effect: EffectId,
    },
    /// Actions that apply a beneficial effect to a target, and therefore do not require
    /// an attack roll or saving throw (e.g. Bless, Shield of Faith).
    BeneficialEffect { effect: EffectId },
    /// Actions that heal a target. These actions do not require an attack roll or saving throw.
    /// They simply heal the target for a certain amount of hit points.
    Healing {
        heal: Arc<dyn Fn(&Character, &ActionContext) -> DiceSetRoll + Send + Sync>,
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
    /// The closure should return a `SpellKindSnapshot` that describes the effect of the spell.
    /// Please note that this should only be used for actions that don't fit into the
    /// standard categories.
    Custom(Arc<dyn Fn(&Character, &ActionContext) -> ActionKindSnapshot + Send + Sync>),
}

/// For some actions we run into a borrowing issue, where we need to borrow the
/// character performing the action immutably (to read their stats, abilities, etc.),
/// and also borrow the target mutably (to apply damage, effects, etc.).
/// To avoid this, we create a snapshot of the action that contains all the
/// precomputed results, and then we can apply those results to the target
/// without needing to borrow the character immutably again.
#[derive(Debug, Clone)]
pub enum ActionKindSnapshot {
    UnconditionalDamage {
        damage_roll: DamageRollResult,
    },
    AttackRollDamage {
        attack_roll: AttackRollResult,
        damage_roll: DamageRollResult,
        damage_on_failure: Option<DamageRollResult>,
    },
    SavingThrowDamage {
        saving_throw: SavingThrowDC,
        half_damage_on_save: bool,
        damage_roll: DamageRollResult,
    },
    UnconditionalEffect {
        effect: EffectId,
    },
    SavingThrowEffect {
        saving_throw: SavingThrowDC,
        effect: EffectId,
    },
    BeneficialEffect {
        effect: EffectId,
    },
    Healing {
        healing: DiceSetRollResult,
    },
    Utility,
    Composite {
        actions: Vec<ActionKindSnapshot>,
    },
    Custom {
        // TODO: Add more fields as needed for custom spells
    },
}

/// The result of applying an action snapshot to a target.
/// This is the final result of the action, which includes any damage dealt,
/// effects applied, or healing done.
#[derive(Debug)]
pub enum ActionKindResult {
    UnconditionalDamage {
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
    AttackRollDamage {
        attack_roll: AttackRollResult,
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
    },
    SavingThrowDamage {
        saving_throw: SavingThrowDC,
        half_damage_on_save: bool,
        damage_roll: DamageRollResult,
        damage_taken: Option<DamageMitigationResult>,
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
    },
    Utility,
    Composite {
        actions: Vec<ActionResult>,
    },
    Custom {
        // TODO: Add more fields as needed for custom spells
    },
}

#[derive(Clone)]
pub struct Action {
    pub id: ActionId,
    pub kind: ActionKind,
    pub targeting: Arc<dyn Fn(&Character, &ActionContext) -> TargetingContext + Send + Sync>,
    /// e.g. Action, Bonus Action, Reaction
    pub resource_cost: HashMap<ResourceId, u8>,
    /// Optional cooldown for the action
    pub cooldown: Option<RechargeRule>,
}

/// Represents the result of performing an action on a single target. For actions that affect multiple targets,
/// multiple `ActionResult` instances can be collected.
#[derive(Debug)]
pub struct ActionResult {
    // TODO: What if the target isn't a Character, but e.g. an object? Like if you cast
    // Knock on a door?
    pub target: TargetTypeInstance,
    pub result: ActionKindResult,
}

/// Represents a "ready-to-go" action that can be performed by a character. The
/// ID is used to look up the action in the action registry, and the context can
/// then be used to perform the action with the appropriate parameters.
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct ActionReference {
//     pub id: ActionId,
//     pub context: ActionContext,
// }

/// Represents a provider of actions, which can be used to retrieve available actions
/// from a character or other entity that can perform actions.
pub trait ActionProvider {
    // TODO: Should probably find a way to avoid rebuilding the action collection every time.

    /// Returns a collection of available actions for the character.
    /// Each action is paired with its context, which provides additional information
    /// about how the action can be performed (e.g. weapon type, spell level, etc.).
    fn actions(&self) -> HashMap<ActionId, Vec<ActionContext>>;
}

impl ActionKind {
    pub fn snapshot(&self, character: &Character, context: &ActionContext) -> ActionKindSnapshot {
        match self {
            ActionKind::UnconditionalDamage { damage } => ActionKindSnapshot::UnconditionalDamage {
                damage_roll: damage(character, context).roll(),
            },

            ActionKind::AttackRollDamage {
                attack_roll,
                damage,
                damage_on_failure,
            } => ActionKindSnapshot::AttackRollDamage {
                attack_roll: attack_roll(character, context).roll(character),
                damage_roll: damage(character, context).roll(),
                damage_on_failure: damage_on_failure
                    .as_ref()
                    .map(|f| f(character, context).roll()),
            },

            ActionKind::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage,
            } => ActionKindSnapshot::SavingThrowDamage {
                saving_throw: saving_throw(character, context),
                half_damage_on_save: *half_damage_on_save,
                damage_roll: damage(character, context).roll(),
            },

            ActionKind::UnconditionalEffect { effect } => ActionKindSnapshot::UnconditionalEffect {
                effect: effect.clone(),
            },

            ActionKind::SavingThrowEffect {
                saving_throw,
                effect,
            } => ActionKindSnapshot::SavingThrowEffect {
                saving_throw: saving_throw(character, context),
                effect: effect.clone(),
            },

            ActionKind::BeneficialEffect { effect } => ActionKindSnapshot::BeneficialEffect {
                effect: effect.clone(),
            },

            ActionKind::Healing { heal } => ActionKindSnapshot::Healing {
                healing: heal(character, context).roll(),
            },

            ActionKind::Utility { .. } => ActionKindSnapshot::Utility,

            ActionKind::Composite { actions } => ActionKindSnapshot::Composite {
                actions: actions
                    .iter()
                    .map(|a| a.snapshot(character, context))
                    .collect(),
            },

            ActionKind::Custom(custom) => custom(character, context),
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

impl ActionKindSnapshot {
    // TODO: Right now only characters can be targeted. I'd really like to avoid
    // using lifetimes here, but since we need a mutable reference to the target,
    // we're either going to have to:
    // 1. Use lifetimes
    // 2. Clone the target (which is not ideal, since it can be expensive)
    // 3. Pass the ID, but then we have to be able to look the ID up somewhere
    pub fn apply_to_character(&self, target: &mut Character) -> ActionResult {
        let result = match self {
            ActionKindSnapshot::UnconditionalDamage { damage_roll } => {
                ActionKindResult::UnconditionalDamage {
                    damage_roll: damage_roll.clone(),
                    damage_taken: target.take_damage(self),
                }
            }

            ActionKindSnapshot::AttackRollDamage {
                attack_roll,
                damage_roll,
                damage_on_failure,
            } => ActionKindResult::AttackRollDamage {
                attack_roll: attack_roll.clone(),
                damage_roll: damage_roll.clone(),
                damage_taken: target.take_damage(self),
            },

            ActionKindSnapshot::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage_roll,
            } => ActionKindResult::SavingThrowDamage {
                saving_throw: saving_throw.clone(),
                half_damage_on_save: *half_damage_on_save,
                damage_roll: damage_roll.clone(),
                damage_taken: target.take_damage(self),
            },

            ActionKindSnapshot::UnconditionalEffect { effect } => {
                ActionKindResult::UnconditionalEffect {
                    effect: effect.clone(),
                    applied: true, // TODO: Unconditional effects are always applied
                }
            }

            ActionKindSnapshot::SavingThrowEffect {
                saving_throw,
                effect,
            } => ActionKindResult::SavingThrowEffect {
                saving_throw: saving_throw.clone(),
                effect: effect.clone(),
                applied: !target.saving_throw_dc(saving_throw).success,
            },

            ActionKindSnapshot::BeneficialEffect { effect } => {
                target.add_effect(
                    registry::effects::EFFECT_REGISTRY
                        .get(&effect)
                        .unwrap()
                        .clone(),
                );
                ActionKindResult::BeneficialEffect {
                    effect: effect.clone(),
                    applied: true, // TODO: Beneficial effects are always applied?
                }
            }

            ActionKindSnapshot::Healing { healing } => {
                target.heal(healing.subtotal as u32);
                ActionKindResult::Healing {
                    healing: healing.clone(),
                }
            }

            ActionKindSnapshot::Utility => ActionKindResult::Utility,

            ActionKindSnapshot::Composite { actions } => ActionKindResult::Composite {
                actions: actions
                    .iter()
                    .map(|a| a.apply_to_character(target))
                    .collect(),
            },
            ActionKindSnapshot::Custom { .. } => {
                // Custom actions can have any kind of effect, so we return a placeholder
                ActionKindResult::Custom {}
            }
        };

        ActionResult {
            target: TargetTypeInstance::Character(target.id()),
            result,
        }
    }
}

impl Action {
    pub fn snapshot(&self, character: &Character, context: &ActionContext) -> ActionKindSnapshot {
        self.kind.snapshot(character, context)
    }

    pub fn perform(
        &self,
        performer: &mut Character,
        context: &ActionContext,
        num_snapshots: usize,
    ) -> Vec<ActionKindSnapshot> {
        // TODO: Resource might error?
        let _ = self.spend_resources(performer, context);

        let snapshots = (0..num_snapshots)
            .map(|_| self.snapshot(performer, context))
            .collect();
        snapshots
    }

    fn spend_resources(
        &self,
        performer: &mut Character,
        context: &ActionContext,
    ) -> Result<(), ResourceError> {
        for (resource, amount) in &self.resource_cost {
            if let Some(resource) = performer.resource_mut(resource) {
                resource.spend(*amount)?;
            }
        }
        // TODO: Not really a fan of this special treatment for spell slots
        match context {
            ActionContext::Spell { level } => {
                performer.spellbook_mut().use_spell_slot(*level);
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
    ) -> &Arc<dyn Fn(&Character, &ActionContext) -> TargetingContext + Send + Sync> {
        &self.targeting
    }

    pub fn resource_cost(&self) -> &HashMap<ResourceId, u8> {
        &self.resource_cost
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

impl Display for ActionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target: {:?}\n", self.target)?;
        match &self.result {
            ActionKindResult::UnconditionalDamage {
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage)?;
                }
            }
            ActionKindResult::AttackRollDamage {
                attack_roll,
                damage_roll,
                damage_taken,
            } => {
                write!(f, "\tAttack Roll: {}\n", attack_roll)?;
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage)?;
                }
            }
            ActionKindResult::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage_roll,
                damage_taken,
            } => {
                write!(
                    f,
                    "Saving Throw: {:?}, Half Damage on Save: {}\n",
                    saving_throw, half_damage_on_save
                )?;
                write!(f, "\tDamage Roll: {}\n", damage_roll)?;
                if let Some(damage) = damage_taken {
                    write!(f, "\tDamage Taken: {}\n", damage)?;
                }
            }
            ActionKindResult::UnconditionalEffect { effect, applied } => {
                write!(
                    f,
                    "Unconditional Effect: {}, Applied: {}\n",
                    effect, applied
                )?;
            }
            ActionKindResult::SavingThrowEffect {
                saving_throw,
                effect,
                applied,
            } => {
                write!(
                    f,
                    "\tSaving Throw Effect: {}, Applied: {}\n",
                    effect, applied
                )?;
            }
            ActionKindResult::BeneficialEffect { effect, applied } => {
                write!(f, "\tBeneficial Effect: {}, Applied: {}\n", effect, applied)?;
            }
            ActionKindResult::Healing { healing } => {
                write!(f, "\tHealing: {}\n", healing)?;
            }
            ActionKindResult::Utility => {
                write!(f, "\tUtility Action\n")?;
            }
            ActionKindResult::Composite { actions } => {
                write!(f, "\tComposite Actions:\n")?;
                for action in actions {
                    write!(f, "\t{}\n", action)?;
                }
            }
            ActionKindResult::Custom {} => {
                write!(f, "\tCustom Action\n")?;
            }
        }
        Ok(())
    }
}
