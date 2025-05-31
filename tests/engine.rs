extern crate nat20_rs;

mod tests {
    use std::sync::Arc;

    use nat20_rs::{
        combat::action::{CombatAction, CombatActionResult},
        effects::effects::{Effect, EffectDuration},
        engine::engine::CombatEngine,
        items::equipment::{equipment::HandSlot, weapon::WeaponType},
        spells::spell::{SpellKindResult, TargetingContext},
        stats::modifier::ModifierSource,
        test_utils::fixtures,
    };

    #[test]
    fn initiative_order() {
        let mut hero = fixtures::creatures::heroes::fighter();
        let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();

        let engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

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
        let mut hero = fixtures::creatures::heroes::fighter();
        fixtures::creatures::heroes::add_initiative(&mut hero);
        let hero_id = hero.id();
        let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();

        let engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

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
        let mut hero = fixtures::creatures::heroes::fighter();
        fixtures::creatures::heroes::add_initiative(&mut hero);
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

        let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();
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
        println!("{}", action_result);

        let damage_result = match action_result {
            nat20_rs::combat::action::CombatActionResult::WeaponAttack {
                target,
                target_armor_class: _,
                attack_roll_result,
                damage_roll_result: _,
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
            (goblin_warrior.max_hp() - damage_result.total).max(0)
        );
    }

    #[test]
    fn cast_spell() {
        let mut hero = fixtures::creatures::heroes::wizard();
        fixtures::creatures::heroes::add_initiative(&mut hero);
        let hero_id = hero.id();

        let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();
        let goblin_warrior_id = goblin_warrior.id();

        let mut engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

        // Check that hero is the current character (he has massive initiative for this test)
        assert!(engine.current_character().id() == hero_id);

        let actions = engine.available_actions();
        println!("=== Available Actions ===");
        for action in actions.clone() {
            println!("{:?}", action);
        }

        let spell_id = "MAGIC_MISSILE";
        let spell_level = 2;

        let spell_action = CombatAction::CastSpell {
            id: spell_id.to_string(),
            level: spell_level,
        };

        assert!(
            actions.contains(&spell_action),
            "Expected to find Magic Missile action in available actions"
        );

        let spell = engine
            .current_character()
            .spell_snapshot(spell_id, spell_level)
            .unwrap()
            .unwrap();

        let targeting_context = spell.targeting_context;
        let mut targets = Vec::new();
        match targeting_context {
            TargetingContext::Multiple(count) => {
                assert!(
                    count == 3 + 1,
                    "Expected {} targets for level {} Magic Missile, but got {}",
                    3 + 1, // Magic Missile always hits 3 targets at level 1
                    spell_level,
                    count,
                );
                for _ in 0..count {
                    targets.push(goblin_warrior_id);
                }
            }
            _ => panic!(
                "Expected a spell with multiple targets, but got: {:?}",
                targeting_context
            ),
        }

        println!("=== Chosen Action ===");
        println!("{:?}", spell_action);

        let action_request = spell_action.request_with_targets(targets);
        assert!(action_request.is_some());

        let result = engine.submit_action(action_request.unwrap());
        assert!(result.is_ok());
        let action_result = result.unwrap();
        println!("=== Action Result ===");
        println!("{}", action_result);

        let spell_result = match action_result {
            CombatActionResult::CastSpell { result } => result,
            _ => panic!("Expected a CastSpell result, but got: {:?}", action_result),
        };

        assert!(
            !spell_result.is_empty(),
            "Expected spell result to not be empty"
        );

        let total_damage: i32 = spell_result
            .iter()
            .map(|r| match &r.result {
                SpellKindResult::Damage {
                    damage_roll: _,
                    damage_taken,
                } => {
                    if let Some(damage) = damage_taken {
                        return damage.total;
                    }
                    0
                }
                _ => panic!("Expected a Damage spell result, but got: {:?}", r.result),
            })
            .sum();
        assert!(
            total_damage > 0,
            "Expected total damage to be greater than 0"
        );

        assert_eq!(
            goblin_warrior.hp(),
            (goblin_warrior.max_hp() - total_damage).max(0)
        );
    }
}
