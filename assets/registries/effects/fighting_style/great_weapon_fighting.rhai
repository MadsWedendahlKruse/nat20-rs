fn damage_roll_result_hook(entity_view, damage_roll_result) {
    // Only apply for melee attacks with a weapon wielded in both hands
    if damage_roll_result.source != "Melee" 
        || !entity_view.loadout.wielding_with_both_hands("Melee") {
        return;
    }

    // Treat any damage dice that rolled a 1 or 2 as a 3
    damage_roll_result.clamp_damage_dice_min(3);
}