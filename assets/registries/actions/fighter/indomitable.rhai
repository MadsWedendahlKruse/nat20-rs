fn reaction_trigger(context) {
    context.event.is_own_failed_d20_check(context, "SavingThrow")
}

fn reaction_body(context) {
    ReactionPlan::reroll_d20_result("class.fighter.level", true)
}