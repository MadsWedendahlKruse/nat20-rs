fn reaction_trigger(context) {
    context.event.is_own_failed_d20_check(context, "Skill")
}

fn reaction_body(context) {
    ReactionPlan::modify_d20_result("1d10")
}