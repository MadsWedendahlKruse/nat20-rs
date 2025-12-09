use rhai::{Array, CustomType, TypeBuilder, plugin::*};

use crate::{
    components::id::ResourceId,
    engine::event::Event,
    scripts::script_api::{
        D20CheckPerformedView, ReactionTriggerContext, ScriptActionView, ScriptD20CheckDCKind,
        ScriptD20Result, ScriptEntityRole, ScriptEventRef, ScriptEventView, ScriptReactionPlan,
        ScriptSavingThrowSpec,
    },
};

#[derive(Clone, CustomType)]
#[rhai_type(name = "D20CheckDC", extra = Self::build_extra)]
pub struct RhaiD20CheckDCKind {
    #[rhai_type(skip)]
    inner: ScriptD20CheckDCKind,
}

impl RhaiD20CheckDCKind {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.with_get("label", |s: &mut Self| s.inner.label.clone());
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "D20Result", extra = Self::build_extra)]
pub struct RhaiD20Result {
    #[rhai_type(skip)]
    inner: ScriptD20Result,
}

impl RhaiD20Result {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_get("total", |s: &mut Self| s.inner.total)
            .with_get("kind", |s: &mut Self| RhaiD20CheckDCKind {
                inner: s.inner.kind.clone(),
            })
            .with_fn("is_success", |s: &mut Self| s.inner.is_success);
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "D20CheckPerformed", extra = Self::build_extra)]
pub struct RhaiD20CheckPerformedView {
    #[rhai_type(skip)]
    inner: D20CheckPerformedView,
}

impl RhaiD20CheckPerformedView {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_get("performer", |s: &mut Self| {
                RhaiD20CheckPerformedView::performer(s)
            })
            .with_get("result", |s: &mut Self| RhaiD20Result {
                inner: s.inner.result.clone(),
            })
            .with_get("dc_kind", |s: &mut Self| RhaiD20CheckDCKind {
                inner: s.inner.dc_kind.clone(),
            });
    }

    pub fn performer(&self) -> u64 {
        u64::from(self.inner.performer.to_bits())
    }
}

// TODO: Kind of similar to RhaiD20CheckDCKind??
#[derive(Clone, CustomType)]
#[rhai_type(name = "SavingThrow")]
pub struct RhaiSavingThrow {
    #[rhai_type(skip)]
    pub inner: ScriptSavingThrowSpec,
}

#[export_module]
pub mod saving_throw_module {
    use super::*;

    pub fn dc(entity_role: String, saving_throw: String) -> RhaiSavingThrow {
        let role = match entity_role.as_str() {
            "reactor" => ScriptEntityRole::Reactor,
            "trigger_actor" => ScriptEntityRole::TriggerActor,
            other => panic!("Unknown entity role in ReactionDC::spell_save: {}", other),
        };

        RhaiSavingThrow {
            inner: ScriptSavingThrowSpec {
                entity: role,
                saving_throw: saving_throw
                    .parse()
                    .expect("Failed to parse SavingThrowProvider"),
            },
        }
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "ActionView", extra = Self::build_extra)]
pub struct RhaiActionView {
    #[rhai_type(skip)]
    inner: ScriptActionView,
}

impl RhaiActionView {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_get("action_id", |s: &mut Self| s.inner.action_id.clone())
            // Expose the actor as a numeric entity id
            .with_get("actor", |s: &mut Self| u64::from(s.inner.actor.to_bits()))
            // Expose a convenience predicate
            .with_fn("is_spell", |s: &mut Self| s.inner.is_spell);
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "Event", extra = Self::build_extra)]
pub struct RhaiEventView {
    #[rhai_type(skip)]
    inner: ScriptEventView,
}

impl RhaiEventView {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_fn(
                "is_d20_check_performed",
                RhaiEventView::is_d20_check_performed,
            )
            .with_fn(
                "as_d20_check_performed",
                RhaiEventView::as_d20_check_performed,
            )
            .with_fn(
                "is_own_failed_d20_check",
                |s: &mut Self, context: RhaiTriggerContext, kind: String| {
                    if !s.is_d20_check_performed() {
                        return false;
                    }
                    let d20_check = s.as_d20_check_performed();
                    return d20_check.performer() == context.reactor_id
                        && !d20_check.inner.result.is_success
                        && d20_check.inner.dc_kind.label == kind;
                },
            )
            .with_fn("is_action", RhaiEventView::is_action)
            .with_fn("as_action", RhaiEventView::as_action);
    }

    pub fn from_api(event: &Event) -> Option<Self> {
        let view = ScriptEventView::from_event(event)?;
        Some(RhaiEventView { inner: view })
    }

    pub fn is_d20_check_performed(&mut self) -> bool {
        matches!(self.inner, ScriptEventView::D20CheckPerformed(_))
    }

    pub fn as_d20_check_performed(&mut self) -> RhaiD20CheckPerformedView {
        if let ScriptEventView::D20CheckPerformed(v) = &self.inner {
            RhaiD20CheckPerformedView { inner: v.clone() }
        } else {
            panic!("as_d20_check_performed called on non-D20CheckPerformed event");
        }
    }

    pub fn is_action(&mut self) -> bool {
        matches!(self.inner, ScriptEventView::ActionRequested(_))
    }

    pub fn as_action(&mut self) -> RhaiActionView {
        if let ScriptEventView::ActionRequested(v) = &self.inner {
            RhaiActionView { inner: v.clone() }
        } else {
            panic!("as_action called on non-action event");
        }
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "TriggerContext", extra = Self::build_extra)]
pub struct RhaiTriggerContext {
    pub reactor_id: u64,
    pub event: RhaiEventView,
}

impl RhaiTriggerContext {
    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_get("reactor", |s: &mut Self| s.reactor_id)
            .with_get("event", |s: &mut Self| s.event.clone());
    }

    pub fn from_api(context: &ReactionTriggerContext) -> Option<Self> {
        let event = RhaiEventView::from_api(&context.event)?;
        Some(RhaiTriggerContext {
            reactor_id: u64::from(context.reactor.to_bits()),
            event,
        })
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(name = "ReactionPlan")]
pub struct RhaiReactionPlan {
    #[rhai_type(skip)]
    pub inner: ScriptReactionPlan,
}

#[export_module]
pub mod reaction_plan_module {
    use crate::{registry::serialize::parser::Parser, scripts::script_api::ScriptD20Bonus};

    use super::*;

    pub fn none() -> RhaiReactionPlan {
        RhaiReactionPlan {
            inner: ScriptReactionPlan::None,
        }
    }

    pub fn sequence(plans: Array) -> RhaiReactionPlan {
        let inner_plans = plans
            .into_iter()
            .map(|v| v.cast::<RhaiReactionPlan>().inner)
            .collect();

        RhaiReactionPlan {
            inner: ScriptReactionPlan::Sequence(inner_plans),
        }
    }

    fn parse_d20_bonus(bonus: String) -> ScriptD20Bonus {
        if let Ok(flat) = Parser::new(&bonus).parse_int_expression() {
            ScriptD20Bonus::Flat(flat)
        } else if let Ok(expr) = Parser::new(&bonus).parse_dice_expression() {
            ScriptD20Bonus::Dice(expr)
        } else {
            panic!("Failed to parse bonus expression: {}", bonus);
        }
    }

    pub fn modify_d20_result(bonus: String) -> RhaiReactionPlan {
        RhaiReactionPlan {
            inner: ScriptReactionPlan::ModifyD20Result {
                bonus: parse_d20_bonus(bonus),
            },
        }
    }

    pub fn reroll_d20_result(bonus: String, force_use_new: bool) -> RhaiReactionPlan {
        let bonus = if bonus.is_empty() {
            None
        } else {
            Some(parse_d20_bonus(bonus))
        };
        RhaiReactionPlan {
            inner: ScriptReactionPlan::RerollD20Result {
                bonus,
                force_use_new,
            },
        }
    }

    pub fn require_saving_throw(
        target_role: ImmutableString,
        dc: RhaiSavingThrow,
        on_success: RhaiReactionPlan,
        on_failure: RhaiReactionPlan,
    ) -> RhaiReactionPlan {
        let target = match target_role.as_str() {
            "reactor" => ScriptEntityRole::Reactor,
            "trigger_actor" => ScriptEntityRole::TriggerActor,
            other => panic!("Unknown entity role in require_saving_throw: {}", other),
        };

        RhaiReactionPlan {
            inner: ScriptReactionPlan::RequireSavingThrow {
                target,
                dc: dc.inner,
                on_success: Box::new(on_success.inner),
                on_failure: Box::new(on_failure.inner),
            },
        }
    }

    pub fn cancel_trigger_event(resources_to_refund: Array) -> RhaiReactionPlan {
        let resources: Vec<ResourceId> = resources_to_refund
            .into_iter()
            .map(|v| ResourceId::from_str(v.cast::<String>()))
            .collect();

        RhaiReactionPlan {
            inner: ScriptReactionPlan::CancelEvent {
                event: ScriptEventRef::TriggerEvent,
                resources_to_refund: resources,
            },
        }
    }
}
