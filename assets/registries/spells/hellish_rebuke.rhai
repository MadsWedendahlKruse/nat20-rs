fn reaction_trigger(context) {
    let event = context.event;

    if !event.is_action_performed() {
        return false;
    }

    let performed = event.as_action_performed();

    // Cannot target yourself
    if performed.action.actor == context.reactor {
        return false;
    }

    for result in performed.results() {
        if result.target == context.reactor && result.kind.is_standard() {
            let standard_kind = result.kind.as_standard();
            if standard_kind.has_damage() {
                let damage = standard_kind.get_damage();
                if damage.damage_taken_total() > 0 {
                    return true;
                }
            }
        }
    }

    false
}