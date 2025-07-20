// extern crate nat20_rs;

// mod tests {
//     use nat20_rs::{
//         components::{
//             actions::action::{ActionKindSnapshot, ActionProvider},
//             damage::{DamageComponentResult, DamageRollResult, DamageSource, DamageType},
//             dice::{DiceSetRollResult, DieSize},
//             modifier::ModifierSet,
//         },
//         registry,
//         test_utils::fixtures,
//     };

//     #[test]
//     fn fighter_action_surge() {
//         let mut fighter = fixtures::creatures::heroes::fighter();

//         // Check that the fighter has the Action Surge action
//         let available_actions = fighter.all_actions();
//         let action_id = registry::actions::ACTION_SURGE_ID.clone();
//         assert!(
//             available_actions.contains_key(&action_id),
//             "Fighter should have Action Surge action"
//         );
//         let (context, _) = available_actions.get(&action_id).unwrap();

//         // Check that the fighter has one charge of Action Surge
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::ACTION_SURGE)
//                 .unwrap()
//                 .current_uses(),
//             1
//         );

//         // Check that the fighter has one action before using Action Surge
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::ACTION)
//                 .unwrap()
//                 .current_uses(),
//             1
//         );

//         let snapshots = fighter.perform_action(&action_id, &context[0], 1);
//         snapshots[0].apply_to_character(&mut fighter);

//         // Check that the Action Surge effect is applied
//         let action_surge_effect = fighter
//             .effects()
//             .iter()
//             .find(|e| *e.id() == *registry::effects::ACTION_SURGE_ID);
//         assert!(
//             action_surge_effect.is_some(),
//             "Action Surge effect should be applied"
//         );

//         // Check that the fighter has two actions after using Action Surge
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::ACTION)
//                 .unwrap()
//                 .current_uses(),
//             2
//         );

//         // Check that the Action Surge action is on cooldown
//         assert!(fighter.is_on_cooldown(&action_id).is_some());

//         // Simulate the start of the turn to remove the Action Surge effect
//         fighter.on_turn_start();

//         // Check that the Action Surge effect is removed after the turn starts
//         let action_surge_effect = fighter
//             .effects()
//             .iter()
//             .find(|e| *e.id() == *registry::effects::ACTION_SURGE_ID);
//         assert!(
//             action_surge_effect.is_none(),
//             "Action Surge effect should be removed"
//         );

//         // Check that the fighter has one action after the turn starts
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::ACTION)
//                 .unwrap()
//                 .current_uses(),
//             1
//         );

//         // Check that the Action Surge action is out of charges
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::ACTION_SURGE)
//                 .unwrap()
//                 .current_uses(),
//             0
//         );
//     }

//     #[test]
//     fn fighter_second_wind() {
//         let mut fighter = fixtures::creatures::heroes::fighter();

//         // Check that the fighter has the Second Wind action
//         let available_actions = fighter.all_actions();
//         let action_id = registry::actions::SECOND_WIND_ID.clone();
//         assert!(
//             available_actions.contains_key(&action_id),
//             "Fighter should have Second Wind action"
//         );
//         let (context, _) = available_actions.get(&action_id).unwrap();

//         // Check that the fighter has two charges of Second Wind
//         assert_eq!(
//             fighter
//                 .resources()
//                 .get(&registry::resources::SECOND_WIND)
//                 .unwrap()
//                 .current_uses(),
//             2
//         );

//         // Let the fighter take some damage
//         let damage_source = ActionKindSnapshot::UnconditionalDamage {
//             damage_roll: DamageRollResult {
//                 label: "Test Damage".to_string(),
//                 components: vec![DamageComponentResult {
//                     result: DiceSetRollResult {
//                         label: "Test Damage".to_string(),
//                         die_size: DieSize::D10,
//                         rolls: vec![3, 4],
//                         modifiers: ModifierSet::new(),
//                         subtotal: 7,
//                     },
//                     damage_type: DamageType::Force,
//                 }],
//                 total: 10,
//                 source: DamageSource::Spell,
//             },
//         };
//         fighter.take_damage(&damage_source);

//         // Check that the fighter's HP is reduced
//         assert!(fighter.hp() < fighter.max_hp());

//         let prev_hp = fighter.hp();

//         let snapshots = fighter.perform_action(&action_id, &context[0], 1);
//         let result = snapshots[0].apply_to_character(&mut fighter);
//         println!("Second Wind Result: {:?}", result);

//         // Check that the Fighters HP is increased by the Second Wind healing
//         assert!(fighter.hp() > prev_hp);
//     }
// }
