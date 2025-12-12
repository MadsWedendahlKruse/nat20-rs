fn reaction_trigger(context) {
    context.is_own_failed_d20_check("SavingThrow")
}

fn reaction_body(context) {
    ReactionPlan::reroll_d20_result("class.fighter.level", true)
}