use std::collections::HashMap;

use rhai::{AST, Engine, Scope, exported_module};

use crate::{
    components::id::ScriptId,
    scripts::{
        rhai::rhai_types::{
            self, RhaiActionView, RhaiD20CheckDCKind, RhaiD20CheckPerformedView, RhaiD20Result,
            RhaiEventView, RhaiReactionPlan, RhaiSavingThrow, RhaiTriggerContext,
        },
        script::{Script, ScriptError},
        script_api::{ReactionBodyContext, ReactionTriggerContext, ScriptReactionPlan},
        script_engine::ScriptEngine,
    },
};

pub struct RhaiScriptEngine {
    pub engine: Engine,
    pub ast_cache: HashMap<ScriptId, AST>,
}

impl RhaiScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        engine
            .build_type::<RhaiTriggerContext>()
            .build_type::<RhaiEventView>()
            .build_type::<RhaiD20CheckPerformedView>()
            .build_type::<RhaiD20Result>()
            .build_type::<RhaiD20CheckDCKind>()
            .build_type::<RhaiSavingThrow>()
            .build_type::<RhaiReactionPlan>()
            .build_type::<RhaiActionView>();

        engine.register_static_module(
            "ReactionPlan",
            exported_module!(rhai_types::reaction_plan_module).into(),
        );
        engine.register_static_module(
            "SavingThrow",
            exported_module!(rhai_types::saving_throw_module).into(),
        );

        RhaiScriptEngine {
            engine,
            ast_cache: HashMap::new(),
        }
    }

    fn cache_script(&mut self, script: &Script) -> Result<(), ScriptError> {
        let ast = self
            .engine
            .compile(&script.content)
            .map_err(|e| ScriptError::LoadError(format!("Failed to compile Rhai script: {}", e)))?;
        self.ast_cache.insert(script.id.clone(), ast);
        Ok(())
    }

    fn get_ast(&mut self, script: &Script) -> Result<&AST, ScriptError> {
        if !self.ast_cache.contains_key(&script.id) {
            self.cache_script(script)?;
        }
        Ok(self
            .ast_cache
            .get(&script.id)
            .expect("AST must exist after caching"))
    }
}

impl ScriptEngine for RhaiScriptEngine {
    fn evaluate_reaction_trigger(
        &mut self,
        script: &Script,
        context: &ReactionTriggerContext,
    ) -> Result<bool, ScriptError> {
        if let Some(rhai_context) = RhaiTriggerContext::from_api(context) {
            // TODO: Don't clone AST every time (if it's actually a performance issue)
            let ast = self.get_ast(script).cloned()?;

            let mut scope = Scope::new();
            self.engine
                .call_fn::<bool>(&mut scope, &ast, "reaction_trigger", (rhai_context,))
                .map_err(|e| ScriptError::RuntimeError(format!("Rhai error: {}", e)))
        } else {
            Err(ScriptError::RuntimeError(
                "Failed to build RhaiTriggerContext".to_string(),
            ))
        }
    }

    fn evaluate_reaction_body(
        &mut self,
        script: &Script,
        context: &ReactionBodyContext,
    ) -> Result<ScriptReactionPlan, ScriptError> {
        // TODO: For now body only needs reactor + event; reuse same wrapper
        if let Some(rhai_context) = RhaiTriggerContext::from_api(&ReactionTriggerContext {
            reactor: context.reaction_data.reactor,
            event: context.reaction_data.event.as_ref().clone(),
        }) {
            // TODO: Don't clone AST every time (if it's actually a performance issue)
            let ast = self.get_ast(script).cloned()?;

            let mut scope = Scope::new();
            let plan = self
                .engine
                .call_fn::<RhaiReactionPlan>(&mut scope, &ast, "reaction_body", (rhai_context,))
                .map_err(|e| ScriptError::RuntimeError(format!("Rhai error: {}", e)))?;

            Ok(plan.inner)
        } else {
            Err(ScriptError::RuntimeError(
                "Failed to build RhaiTriggerContext".to_string(),
            ))
        }
    }
}
