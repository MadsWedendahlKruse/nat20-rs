fn reaction_trigger(context) {
    let event = context.event;

    if event.is_action_requested() {
        let action = event.as_action_requested();
        // Cannot use Shield as a reaction to your own spell
        if action.actor == context.reactor {
            return false;
        }
        // Note that we're checking for the ID of the Magic Missile *action*
        // here, not the spell itself.
        return action.action_id == "nat20_core::action.magic_missile";
    }

    if event.is_d20_check_performed() {
        let d20_check = event.as_d20_check_performed();

        // Shield can be used to react to an attack roll against the caster
        let result = d20_check.result;
        return result.dc_kind.label == "AttackRoll" && 
            result.dc_kind.target == context.reactor &&
            result.is_success == true;
    }

    return false;
}

fn post_damage_mitigation_hook(entity_view, damage_mitigation_result) {
    if damage_mitigation_result.source == "nat20_core::spell.magic_missile" {
        damage_mitigation_result.add_immunity();
    }
}