extern crate nat20_rs;

mod tests {
    use std::sync::Arc;

    use hecs::World;
    use nat20_rs::{
        components::{
            actions::action::{ActionContext, ActionKind},
            damage::{
                DamageComponentResult, DamageRoll, DamageRollResult, DamageSource, DamageType,
            },
            dice::{DiceSetRollResult, DieSize},
            health::hit_points::HitPoints,
            id::ActionId,
            items::equipment::slots::EquipmentSlot,
            modifier::ModifierSet,
            resource::{self, RechargeRule, ResourceAmountMap, ResourceMap},
        },
        engine::{
            event::{ActionData, ActionDecision},
            game_state::{self, GameState},
        },
        registry, systems,
        test_utils::fixtures,
    };

    #[test]
    fn fighter_action_surge() {
        let mut game_state = GameState::new();
        let fighter = fixtures::creatures::heroes::fighter(&mut game_state.world).id();

        // Check that the fighter has the Action Surge action
        let available_actions = systems::actions::available_actions(&game_state.world, fighter);
        let action_id = registry::actions::ACTION_SURGE_ID.clone();
        assert!(
            available_actions.contains_key(&action_id),
            "Fighter should have Action Surge action"
        );
        let contexts_and_costs = available_actions.get(&action_id).unwrap();

        {
            let resources =
                systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter);
            // Check that the fighter has one charge of Action Surge
            assert!(resources.can_afford(
                &registry::resources::ACTION_SURGE_ID,
                &registry::resources::ACTION_SURGE.build_amount(1)
            ));
            // Check that the fighter has one action before using Action Surge
            assert!(resources.can_afford(
                &registry::resources::ACTION_ID,
                &registry::resources::ACTION.build_amount(1)
            ));
        }

        let _ = game_state.submit_decision(ActionDecision::Action {
            action: ActionData {
                actor: fighter,
                action_id: action_id.clone(),
                context: contexts_and_costs[0].0.clone(),
                resource_cost: contexts_and_costs[0].1.clone(),
                targets: vec![fighter],
            },
        });

        {
            // Check that the Action Surge effect is applied
            let effects = systems::effects::effects(&game_state.world, fighter);
            let action_surge_effect = effects
                .iter()
                .find(|e| *e.id() == *registry::effects::ACTION_SURGE_ID);
            assert!(
                action_surge_effect.is_some(),
                "Action Surge effect should be applied"
            );
        }

        // Check that the fighter has two actions after using Action Surge
        assert!(
            systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::ACTION_ID,
                &registry::resources::ACTION.build_amount(2)
            ),
        );

        // Check that the Action Surge action is on cooldown
        assert!(systems::actions::on_cooldown(&game_state.world, fighter, &action_id).is_some());

        // Simulate the start of the turn to remove the Action Surge effect
        systems::time::pass_time(&mut game_state.world, fighter, &RechargeRule::Turn);

        // Check that the Action Surge effect is removed after the turn starts
        let effects = systems::effects::effects(&game_state.world, fighter);
        let action_surge_effect = effects
            .iter()
            .find(|e| *e.id() == *registry::effects::ACTION_SURGE_ID);
        assert!(
            action_surge_effect.is_none(),
            "Action Surge effect should be removed"
        );

        let resources = systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter);
        // Check that the fighter has one action after the turn starts
        assert!(
            !resources.can_afford(
                &registry::resources::ACTION_ID,
                &registry::resources::ACTION.build_amount(2)
            ) && resources.can_afford(
                &registry::resources::ACTION_ID,
                &registry::resources::ACTION.build_amount(1)
            )
        );

        // Check that the Action Surge action is out of charges
        assert!(!resources.can_afford(
            &registry::resources::ACTION_SURGE_ID,
            &registry::resources::ACTION_SURGE.build_amount(1)
        ));
    }

    #[test]
    fn fighter_second_wind() {
        let mut game_state = GameState::new();
        let fighter = fixtures::creatures::heroes::fighter(&mut game_state.world).id();

        // Check that the fighter has the Second Wind action
        let available_actions = systems::actions::available_actions(&game_state.world, fighter);
        let action_id = registry::actions::SECOND_WIND_ID.clone();
        assert!(
            available_actions.contains_key(&action_id),
            "Fighter should have Second Wind action"
        );
        let contexts_and_costs = available_actions.get(&action_id).unwrap();

        // Check that the fighter has two charges of Second Wind
        assert!(
            systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::SECOND_WIND_ID,
                &registry::resources::SECOND_WIND.build_amount(2)
            )
        );

        // Let the fighter take some damage
        let damage_source = ActionKind::UnconditionalDamage {
            damage: Arc::new(|_, _, _| {
                DamageRoll::new(
                    1,
                    DieSize::D4,
                    DamageType::Force,
                    DamageSource::Spell,
                    "Magic Missile".to_string(),
                )
            }),
        };
        systems::health::damage(
            &mut game_state,
            fighter,
            fighter,
            &ActionId::from_str("test.damage"),
            &damage_source,
            &ActionContext::Other,
            ResourceAmountMap::new(),
        );

        // Check that the fighter's HP is reduced
        let prev_hp = {
            let hit_points =
                systems::helpers::get_component::<HitPoints>(&game_state.world, fighter);
            assert!(hit_points.current() < hit_points.max());

            hit_points.current()
        };

        let result = systems::actions::perform_action(
            &mut game_state,
            &ActionData {
                actor: fighter,
                action_id: action_id.clone(),
                context: contexts_and_costs[0].0.clone(),
                resource_cost: contexts_and_costs[0].1.clone(),
                targets: vec![fighter],
            },
        );
        println!("Second Wind Result: {:?}", result);

        // Check that the Fighters HP is increased by the Second Wind healing
        assert!(
            systems::helpers::get_component::<HitPoints>(&game_state.world, fighter).current()
                > prev_hp
        );
    }

    #[test]
    fn fighter_extra_attack() {
        let mut game_state = GameState::new();
        let fighter = fixtures::creatures::heroes::fighter(&mut game_state.world).id();

        // Check that the fighter has the Extra Attack effect
        {
            let effects = systems::effects::effects(&game_state.world, fighter);
            let extra_attack_effect = effects
                .iter()
                .find(|e| *e.id() == *registry::effects::EXTRA_ATTACK_ID);
            assert!(
                extra_attack_effect.is_some(),
                "Fighter should have Extra Attack effect"
            );
        }

        // Check that the fighter has no stacks of Extra Attack (yet)
        assert!(
            !systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::EXTRA_ATTACK_ID,
                &registry::resources::EXTRA_ATTACK.build_amount(1)
            ),
            "Fighter should have no stacks of Extra Attack"
        );

        // Fighter makes a weapon attack, which costs one Action and grants one stack of Extra Attack
        let available_actions = systems::actions::available_actions(&game_state.world, fighter);
        let action_id = registry::actions::WEAPON_ATTACK_ID.clone();
        assert!(
            available_actions.contains_key(&action_id),
            "Fighter should have Weapon Attack action"
        );
        let contexts_and_costs = available_actions.get(&action_id).unwrap();
        let _ = game_state.submit_decision(ActionDecision::Action {
            action: ActionData {
                actor: fighter,
                action_id: action_id.clone(),
                context: contexts_and_costs[0].0.clone(),
                resource_cost: contexts_and_costs[0].1.clone(),
                targets: vec![],
            },
        });

        // Check that the fighter has one stack of Extra Attack
        assert!(
            systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::EXTRA_ATTACK_ID,
                &registry::resources::EXTRA_ATTACK.build_amount(1)
            ),
            "Fighter should have one stack of Extra Attack"
        );
        // Check that the fighter has no Actions left
        assert!(
            !systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::ACTION_ID,
                &registry::resources::ACTION.build_amount(1)
            ),
            "Fighter should have no Actions left"
        );

        // Fighter makes another attack, which should consume the Extra Attack stack
        let available_actions = systems::actions::available_actions(&game_state.world, fighter);
        let contexts_and_costs = available_actions.get(&action_id).unwrap();
        let _ = game_state.submit_decision(ActionDecision::Action {
            action: ActionData {
                actor: fighter,
                action_id: action_id.clone(),
                context: contexts_and_costs[0].0.clone(),
                resource_cost: contexts_and_costs[0].1.clone(),
                targets: vec![],
            },
        });

        // Check that the fighter has no stacks of Extra Attack left
        assert!(
            !systems::helpers::get_component::<ResourceMap>(&game_state.world, fighter).can_afford(
                &registry::resources::EXTRA_ATTACK_ID,
                &registry::resources::EXTRA_ATTACK.build_amount(1)
            ),
            "Fighter should have no stacks of Extra Attack left"
        );
    }
}
