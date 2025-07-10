extern crate nat20_rs;

mod tests {
    use nat20_rs::{actions::action::ActionProvider, registry, test_utils::fixtures};

    #[test]
    fn fighter_action_surge() {
        let mut fighter = fixtures::creatures::heroes::fighter();

        // Check that the fighter has the Action Surge action
        let available_actions = fighter.all_actions();
        let action_id = registry::actions::ACTION_SURGE_ID.clone();
        assert!(
            available_actions.contains_key(&action_id),
            "Fighter should have Action Surge action"
        );
        let context = available_actions.get(&action_id).unwrap();

        // Check that the fighter has one charge of Action Surge
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION_SURGE)
                .unwrap()
                .current_uses(),
            1
        );

        // Check that the fighter has one action before using Action Surge
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            1
        );

        let snapshots = fighter.perform_action(&action_id, &context[0], 1);
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

        // Check that the Action Surge action is on cooldown
        assert!(fighter.is_on_cooldown(&action_id).is_some());

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

        // Check that the Action Surge action is out of charges
        assert_eq!(
            fighter
                .resources()
                .get(&registry::resources::ACTION_SURGE)
                .unwrap()
                .current_uses(),
            0
        );
    }
}
