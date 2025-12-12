fn damage_roll_result_hook(entity_view, damage_roll_result) {
    // Only apply for melee attacks with a weapon wielded in both hands
    if damage_roll_result.source.kind != "Melee" 
        || !entity_view.loadout.wielding_with_both_hands("Melee") {
        return damage_roll_result;
    }

    // Re-roll 1s and 2s on damage dice
    // TODO: Feels like I'm wrestling with Rhai here a bit to get it to actually
    // let me mutate the rolls. There might be a cleaner way to do this.
    let new_damage_roll_result = damage_roll_result;
    let components = new_damage_roll_result.components;
    for i in 0..components.len() {
        let rolls = components[i].result.rolls;
        for j in 0..rolls.len() {
            if rolls[j] < 3 {
                rolls[j] = 3;
            }
        }
        components[i].result.rolls = rolls;
    }
    new_damage_roll_result.components = components;
    return new_damage_roll_result;
}