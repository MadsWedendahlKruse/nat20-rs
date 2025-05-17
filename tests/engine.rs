extern crate nat20_rs;

mod tests {
    use std::sync::Arc;

    use nat20_rs::{
        combat::action::CombatAction,
        effects::effects::{Effect, EffectDuration},
        engine::engine::CombatEngine,
        item::equipment::{equipment::HandSlot, weapon::WeaponType},
        stats::modifier::ModifierSource,
        test_utils::fixtures,
    };

    #[test]
    fn initiative_order() {
        let mut hero = fixtures::characters::hero();
        let mut goblin_warrior = fixtures::characters::goblin_warrior();

        let mut engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

        let initiative = engine.initiative_order();
        println!("=== Initiative Order ===");
        for (i, (id, result)) in initiative.iter().enumerate() {
            println!(
                "{}: {:?} {:?}",
                i + 1,
                engine.participant(id).unwrap().name(),
                result
            );
        }
        assert_eq!(initiative.len(), 2);
        assert!(initiative[0].1.total > initiative[1].1.total);
        assert_eq!(engine.current_character().id(), initiative[0].0);
    }

    #[test]
    fn available_actions() {
        let mut hero = fixtures::characters::hero_initiative();
        let hero_id = hero.id();
        let mut goblin_warrior = fixtures::characters::goblin_warrior();

        let mut engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

        // Check that hero is the current character (he has massive initiative for this test)
        assert!(engine.current_character().id() == hero_id);

        let actions = engine.available_actions();
        println!("=== Available Actions ===");
        for action in actions.clone() {
            println!("{:?}", action);
        }

        assert!(!actions.is_empty());
        assert!(actions.contains(&CombatAction::WeaponAttack {
            weapon_type: WeaponType::Melee,
            hand: HandSlot::Main
        }));
    }

    #[test]
    fn weapon_attack() {
        let mut hero = fixtures::characters::hero_initiative();
        let hero_id = hero.id();
        // Make sure the hero hits the goblin warrior
        let mut test_effect = Effect::new(
            ModifierSource::Custom("Test Effect".to_string()),
            EffectDuration::Persistent,
        );
        test_effect.pre_attack_roll = Arc::new(|_, d20_check| {
            d20_check.add_modifier(ModifierSource::Custom("Test Effect".to_string()), 20);
        });
        hero.add_effect(test_effect);

        let mut goblin_warrior = fixtures::characters::goblin_warrior();
        let goblin_warrior_id = goblin_warrior.id();

        let mut engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

        // Check that hero is the current character (he has massive initiative for this test)
        assert!(engine.current_character().id() == hero_id);

        let actions = engine.available_actions();
        assert!(actions.contains(&CombatAction::WeaponAttack {
            weapon_type: WeaponType::Melee,
            hand: HandSlot::Main
        }));

        let action = actions[0].clone();
        println!("=== Action ===");
        println!("{:?}", action);

        let action_request = action.request_with_targets(vec![goblin_warrior_id]);
        assert!(action_request.is_some());

        let result = engine.submit_action(action_request.unwrap());
        assert!(result.is_ok());
        let action_result = result.unwrap();
        println!("=== Action Result ===");
        println!("{:?}", action_result);

        let weapon_attack_result = match action_result {
            nat20_rs::combat::action::CombatActionResult::WeaponAttack {
                target,
                attack_roll_result,
                damage_result,
            } => {
                assert_eq!(target, goblin_warrior_id);
                assert!(attack_roll_result.total > 0);
                assert!(damage_result.is_some());
                damage_result.unwrap()
            }
            _ => panic!(
                "Expected a WeaponAttack result, but got: {:?}",
                action_result
            ),
        };
        assert_eq!(
            goblin_warrior.hp(),
            goblin_warrior.max_hp() - weapon_attack_result.total
        );
    }
}
