fn reaction_trigger(context) {
    let event = context.event;

    // Only care about "action-like" events (ActionRequested / ReactionRequested)
    if !event.is_action_requested() {
        return false;
    }

    let action = event.as_action_requested();

    // Cannot counterspell yourself
    if action.actor == context.reactor {
        return false;
    }

    // Only react to spells (for now any spell; you can refine later)
    if !action.action_context.is_spell() {
        return false;
    }

    true
}

fn reaction_body(context) {
    // Counterspell requires the target to make a Constitution saving throw
    // against the reactor's spell save DC.
    let dc = SavingThrow::dc("reactor", "spell_save_dc;constitution");

    // If the target succeeds, counterspell does nothing.
    let on_success = ReactionPlan::none();
    // If the target fails, the spell is canceled and the spell slot is refunded.
    let on_failure = ReactionPlan::cancel_trigger_event(["nat20_rs::resource.spell_slot"]);

    // Combine into a saving throw reaction plan where the actor which triggered
    // the reaction is the one making the saving throw.
    ReactionPlan::require_saving_throw(
        "actor",
        dc,
        on_success,
        on_failure,
    )
}

