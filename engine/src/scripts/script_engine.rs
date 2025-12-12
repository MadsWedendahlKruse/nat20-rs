use std::collections::HashMap;

use crate::{
    components::damage::DamageRollResult,
    scripts::{
        script::{Script, ScriptError, ScriptLanguage},
        script_api::{
            ScriptActionView, ScriptEntityView, ScriptReactionBodyContext, ScriptReactionPlan,
            ScriptReactionTriggerContext, ScriptResourceCost,
        },
    },
};

pub trait ScriptEngine {
    /// Pure predicate: should the reaction trigger?
    fn evaluate_reaction_trigger(
        &mut self,
        script: &Script,
        context: &ScriptReactionTriggerContext,
    ) -> Result<bool, ScriptError>;

    /// Compute the reaction plan to execute.
    fn evaluate_reaction_body(
        &mut self,
        script: &Script,
        context: &ScriptReactionBodyContext,
    ) -> Result<ScriptReactionPlan, ScriptError>;

    /// Compute the resource cost for an action.
    fn evaluate_resource_cost_hook(
        &mut self,
        script: &Script,
        action: &ScriptActionView,
        entity: &ScriptEntityView,
    ) -> Result<ScriptResourceCost, ScriptError>;

    /// Execute an action hook, returning the modified entity view.
    fn evaluate_action_hook(
        &mut self,
        script: &Script,
        action: &ScriptActionView,
        entity: &ScriptEntityView,
    ) -> Result<ScriptEntityView, ScriptError>;

    /// Execute an armor class hook, returning the modifier to apply.
    fn evaluate_armor_class_hook(
        &mut self,
        script: &Script,
        entity: &ScriptEntityView,
    ) -> Result<i32, ScriptError>;

    /// Execute a damage roll result hook, returning the modified damage roll result.
    fn evaluate_damage_roll_result_hook(
        &mut self,
        script: &Script,
        entity: &ScriptEntityView,
        damage_roll_result: &DamageRollResult,
    ) -> Result<DamageRollResult, ScriptError>;
}

pub type ScriptEngineMap = HashMap<ScriptLanguage, Box<dyn ScriptEngine>>;
