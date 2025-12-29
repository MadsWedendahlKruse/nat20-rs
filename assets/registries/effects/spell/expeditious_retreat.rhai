export const ACTION_RESOURCE_ID = "nat20_rs::resource.action";
export const BONUS_ACTION_RESOURCE_ID = "nat20_rs::resource.bonus_action";

fn resource_cost_hook(action_view, entity_view) {
    if action_view.action_id != "nat20_rs::action.dash" {
        return;
    }

    action_view.resource_cost.replace_resource(
        global::ACTION_RESOURCE_ID,
        global::BONUS_ACTION_RESOURCE_ID,
        "1"
    );
}