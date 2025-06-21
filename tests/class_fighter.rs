extern crate nat20_rs;

mod tests {
    use nat20_rs::{actions::action::ActionProvider, registry, test_utils::fixtures};

    #[test]
    fn fighter_action_surge() {
        let mut fighter = fixtures::creatures::heroes::fighter();

        // Check that the fighter has the Action Surge action
        let available_actions = fighter.available_actions().clone();
        let (action_surge_action, context) = available_actions
            .iter()
            .find(|(action, _)| action.id == *registry::actions::ACTION_SURGE_ID)
            .unwrap()
            .clone();
        let action_surge_action = action_surge_action.clone();

        // Check that the fighter has one action before using Action Surge
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            1
        );

        let snapshots = action_surge_action.perform(&mut fighter, &context, 1);
        snapshots[0].apply_to_character(&mut fighter);

        // Check that the Action Surge effect is applied
        let action_surge_effect = fighter
            .effects()
            .iter()
            .find(|e| e.id == *registry::effects::ACTION_SURGE_ID);
        assert!(
            action_surge_effect.is_some(),
            "Action Surge effect should be applied"
        );

        // Check that the fighter has two actions after using Action Surge
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            2
        );

        // Simulate the start of the turn to remove the Action Surge effect
        fighter.on_turn_start();

        // Check that the Action Surge effect is removed after the turn starts
        let action_surge_effect = fighter
            .effects()
            .iter()
            .find(|e| e.id == *registry::effects::ACTION_SURGE_ID);
        assert!(
            action_surge_effect.is_none(),
            "Action Surge effect should be removed"
        );

        // Check that the fighter has one action after the turn starts
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            1
        );
    }
}
