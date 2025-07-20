// extern crate nat20_rs;

// use std::collections::HashMap;

// use nat20_rs::{
//     actions::{
//         action::{Action, ActionContext, ActionKind, ActionProvider},
//         targeting::TargetingKind,
//     },
//     engine::encounter::Encounter,
//     test_utils::{cli::CliChoiceProvider, fixtures},
//     utils::id::ActionId,
// };

// fn main() {
//     let heros = vec![
//         fixtures::creatures::heroes::fighter(),
//         fixtures::creatures::heroes::wizard(),
//         fixtures::creatures::heroes::warlock(),
//     ];

//     let hero_index =
//         CliChoiceProvider::select_from_list("Select a hero to play with:", &heros, |hero| {
//             format!("{} (Level {:?})", hero.name(), hero.classes())
//         });

//     let mut hero = heros.into_iter().nth(hero_index).unwrap();
//     let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();

//     let mut engine = Encounter::new(vec![&mut hero, &mut goblin_warrior]);

//     let initiative = engine.initiative_order();
//     println!("=== Initiative Order ===");
//     for (i, (id, result)) in initiative.iter().enumerate() {
//         println!(
//             "{}: {}, {}",
//             i + 1,
//             engine.participant(id).unwrap().name(),
//             result
//         );
//     }

//     loop {
//         let round = engine.round();
//         println!("\n=== Round {} ===\n", round);

//         println!("{}", engine.current_character());

//         let mut available_actions = engine.available_actions();

//         // The "End Turn" action is always available and needs special handling
//         let end_turn_action_id = ActionId::from_str("action.end_turn");
//         available_actions.insert(
//             end_turn_action_id.clone(),
//             (vec![ActionContext::Other], HashMap::new()),
//         );

//         let actions_options: Vec<_> = available_actions
//             .iter()
//             .flat_map(|(action_id, (contexts, resource_costs))| {
//                 contexts.iter().map(move |context| {
//                     (action_id.clone(), context.clone(), resource_costs.clone())
//                 })
//             })
//             .collect();

//         let action_index = CliChoiceProvider::select_from_list(
//             "Select an action to perform:",
//             &actions_options,
//             |(id, context, resource_costs)| format!("{}, {:?}, {:?}", id, context, resource_costs),
//         );

//         let (action_id, action_context, _) = &actions_options[action_index];
//         if action_id == &end_turn_action_id {
//             // End turn action
//             engine.end_turn();
//             continue;
//         }

//         // If the action uses an attack roll, calculate the chance to hit each target
//         let action = engine.current_character().find_action(action_id).unwrap();
//         fn display<'a>(
//             id: &'a uuid::Uuid,
//             engine: &Encounter,
//             action: &Action,
//             action_context: &ActionContext,
//         ) -> String {
//             let target = engine.participant(id).unwrap();
//             let mut result = target.name().to_string();
//             match &action.kind {
//                 ActionKind::AttackRollDamage { attack_roll, .. } => {
//                     let hit_chance = (attack_roll)(engine.current_character(), action_context)
//                         .hit_chance(target, target.armor_class().total() as u32);
//                     result.push_str(&format!(" (Hit Chance: {:.2}%)", hit_chance * 100.0));
//                 }
//                 _ => {}
//             };
//             result
//         }

//         let targeting_context = engine
//             .current_character()
//             .targeting_context(action_id, action_context);

//         let all_participants = engine.participants();
//         let participants: Vec<_> = all_participants
//             .iter()
//             .filter(|world, entity| character.id() != engine.current_character().id())
//             .collect();
//         let participant_ids: Vec<_> = participants.iter().map(|c| c.id().clone()).collect();

//         let targets = match targeting_context.kind {
//             TargetingKind::SelfTarget => {
//                 vec![engine.current_character().id().clone()]
//             }

//             TargetingKind::Single => {
//                 let participant_index = CliChoiceProvider::select_from_list(
//                     "Select a target:",
//                     &participant_ids,
//                     |id| display(id, &engine, &action, action_context),
//                 );
//                 vec![participant_ids[participant_index].clone()]
//             }

//             TargetingKind::Multiple { max_targets } => {
//                 CliChoiceProvider::select_multiple(
//                     &format!(
//                         "Select up to {} targets (you can select the same target multiple times):",
//                         max_targets
//                     ),
//                     &participant_ids,
//                     max_targets,
//                     |id| display(id, &engine, &action, action_context),
//                     false, // Allow duplicates
//                 )
//             }

//             _ => {
//                 todo!("Implement targeting for other kinds of actions");
//             }
//         };

//         let results = engine
//             .submit_action(action_id, action_context, targets)
//             .unwrap();
//         for result in results {
//             println!("{}", result);
//         }
//     }
// }

fn main() {
    // This is a placeholder for the main function.
    // The actual game logic would be implemented here.
    println!("Combat CLI Demo is running...");
}
