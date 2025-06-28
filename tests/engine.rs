extern crate nat20_rs;

mod tests {
    use std::{sync::Arc, vec};

    use nat20_rs::{
        actions::{
            action::{ActionContext, ActionKindResult},
            targeting::TargetingKind,
        },
        effects::effects::{Effect, EffectDuration},
        engine::engine::CombatEngine,
        registry,
        stats::modifier::ModifierSource,
        test_utils::fixtures,
        utils::id::{ActionId, EffectId, SpellId},
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
        assert!(initiative[0].1.total >= initiative[1].1.total);
        assert_eq!(engine.current_character().id(), initiative[0].0);
    }

    #[test]
    fn available_actions() {
        let mut hero = fixtures::creatures::heroes::fighter();
        fixtures::creatures::heroes::add_initiative(&mut hero);
        println!("{}", hero);
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
        assert!(actions.contains_key(&registry::actions::WEAPON_MELEE_ATTACK_ID));
    }

    #[test]
    fn weapon_attack() {
        let mut hero = fixtures::creatures::heroes::fighter();
        fixtures::creatures::heroes::add_initiative(&mut hero);
        let hero_id = hero.id();
        // Make sure the hero hits the goblin warrior
        let mut test_effect = Effect::new(
            EffectId::from_str("effect.test_effect"),
            ModifierSource::Custom("Test Effect".to_string()),
            EffectDuration::Persistent,
        );
        test_effect.pre_attack_roll = Arc::new(|_, attack_roll| {
            attack_roll
                .d20_check
                .add_modifier(ModifierSource::Custom("Test Effect".to_string()), 20);
        });
        hero.add_effect(test_effect);

        let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();
        let goblin_warrior_id = goblin_warrior.id();

        let mut engine = CombatEngine::new(vec![&mut hero, &mut goblin_warrior]);

        // Check that hero is the current character (he has massive initiative for this test)
        assert!(engine.current_character().id() == hero_id);

        let actions = engine.available_actions();
        let action_id = &registry::actions::WEAPON_MELEE_ATTACK_ID;
        let context = actions.get(&action_id).unwrap()[0].clone();

        println!("=== Action ===");
        println!("id: {:?}, context: {:?}", action_id, context);

        let targeting_context = engine
            .current_character()
            .targeting_context(action_id, &context);
        // TODO: Check the targeting context is correct and populates the target list
        println!("=== Targeting Context ===");
        println!("{:?}", targeting_context);
        assert!(targeting_context.kind == TargetingKind::Single);

        let action_result = engine.submit_action(&action_id, &context, vec![goblin_warrior_id]);

        assert_eq!(
            engine
                .current_character()
                .resource(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            0,
            "Expected attack to cost an action"
        );
        assert!(action_result.is_ok());
        let action_result = action_result.unwrap();
        assert!(
            action_result.len() == 1,
            "Expected exactly one action result"
        );
        let action_result = action_result.get(0).unwrap();

        println!("=== Action Result ===");
        println!("{}", action_result);

        let damage = match &action_result.result {
            ActionKindResult::AttackRollDamage { damage_roll, .. } => damage_roll,
            _ => panic!(
                "Expected an AttackRollDamage result, but got: {:?}",
                action_result
            ),
        };

        assert_eq!(
            goblin_warrior.hp(),
            (goblin_warrior.max_hp() as i32 - damage.total).max(0) as u32
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

        // TODO: Placeholder ID
        let spell_level = 2;

        let action_id = ActionId::from_str("spell.magic_missile");
        let context = ActionContext::Spell { level: spell_level };
        println!("=== Action ===");
        println!("{:?}", (&action_id, &context));

        let targeting_context = engine
            .current_character()
            .targeting_context(&action_id, &context);
        let num_targets = match targeting_context.kind {
            TargetingKind::Multiple { max_targets } => max_targets,
            _ => panic!("Unexpected targeting kind: {:?}", targeting_context.kind),
        };

        // Level 2 Magic Missile should have 4 missiles
        assert_eq!(num_targets, 4, "Expected Magic Missile to have 4 missiles");
        let mut targets = Vec::new();
        for _ in 0..num_targets {
            targets.push(goblin_warrior_id);
        }

        let spell_slots = engine
            .current_character()
            .spellbook()
            .spell_slots_for_level(spell_level);

        let action_result = engine.submit_action(&action_id, &context, targets);

        assert!(action_result.is_ok());
        let action_result = action_result.unwrap();

        assert_eq!(
            engine
                .current_character()
                .spellbook()
                .spell_slots_for_level(spell_level),
            &spell_slots - 1,
            "Expected one spell slot to be used for casting Magic Missile"
        );
        assert_eq!(
            engine
                .current_character()
                .resource(&registry::resources::ACTION)
                .unwrap()
                .current_uses(),
            0,
            "Expected spell to cost an action"
        );
        assert_eq!(
            action_result.len(),
            4,
            "Expected exactly four action results for Magic Missile"
        );
        let total_damage: i32 = action_result
            .iter()
            .map(|result| {
                if let ActionKindResult::UnconditionalDamage { damage_roll, .. } = &result.result {
                    damage_roll.total
                } else {
                    panic!(
                        "Expected a UnconditionalDamage result, but got: {:?}",
                        result
                    );
                }
            })
            .sum();
        assert!(
            total_damage > 0,
            "Expected total damage to be greater than 0, but got: {}",
            total_damage
        );
        assert_eq!(
            goblin_warrior.hp(),
            (goblin_warrior.max_hp() as i32 - total_damage).max(0) as u32,
            "Expected Goblin Warrior's HP to be reduced by the total damage dealt"
        );

        println!("=== Action Result ===");
        for result in action_result {
            println!("{}", result);
        }
    }
}
