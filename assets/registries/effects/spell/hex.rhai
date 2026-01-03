export const HEX_REAPPLY_RESOURCE_ID = "nat20_rs::resource.warlock.hex_reapply";

fn pre_damage_mitigation_hook(entity_view, effect_view, damage_roll_result) {
    if damage_roll_result.is_action_attack_roll()
            && damage_roll_result.has_actor()
            && effect_view.has_applier() {
        let actor = damage_roll_result.get_actor();
        let applier = effect_view.get_applier();
        if actor == applier {
            damage_roll_result.add_damage("1d6", "necrotic")
        }
    }
}

fn death_hook(victim_entity_view, killer_entity_view, applier_entity_view) {
    if applier_entity_view.is_some {
        let applier = applier_entity_view.get();
        if !applier.resources.can_afford_resource(global::HEX_REAPPLY_RESOURCE_ID, "1") {
            applier.resources.add_resource(
                global::HEX_REAPPLY_RESOURCE_ID,
                "1"
            );
        }
    }
}