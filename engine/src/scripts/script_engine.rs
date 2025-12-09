use std::collections::HashMap;

use crate::scripts::{
    script_api::{ReactionBodyContext, ReactionTriggerContext, ScriptReactionPlan},
    script::{Script, ScriptError, ScriptLanguage},
};

pub trait ScriptEngine {
    /// Pure predicate: should the reaction trigger?
    fn evaluate_reaction_trigger(
        &mut self,
        script: &Script,
        context: &ReactionTriggerContext,
    ) -> Result<bool, ScriptError>;

    /// Compute the reaction plan to execute.
    fn evaluate_reaction_body(
        &mut self,
        script: &Script,
        context: &ReactionBodyContext,
    ) -> Result<ScriptReactionPlan, ScriptError>;
}

pub type ScriptEngineMap = HashMap<ScriptLanguage, Box<dyn ScriptEngine>>;
