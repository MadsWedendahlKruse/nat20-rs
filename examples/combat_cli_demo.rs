extern crate nat20_rs;

use std::collections::{HashMap, HashSet};

use nat20_rs::{
    actions::{
        action::{ActionContext, ActionProvider},
        targeting::TargetingKind,
    },
    engine::engine::CombatEngine,
    test_utils::{cli::CliChoiceProvider, fixtures},
    utils::id::ActionId,
};

fn main() {
    let mut hero = fixtures::creatures::heroes::fighter();
    let mut goblin_warrior = fixtures::creatures::monsters::goblin_warrior();

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

    loop {
        let round = engine.round();
        println!("\n=== Round {} ===\n", round);

        println!("{}", engine.current_character());

        let mut available_actions = engine.available_actions();

        // The "End Turn" action is always available and needs special handling
        let end_turn_action_id = ActionId::from_str("action.end_turn");
        available_actions.insert(
            end_turn_action_id.clone(),
            (vec![ActionContext::Other], HashMap::new()),
        );

        let actions_options: Vec<_> = available_actions
            .iter()
            .flat_map(|(action_id, (contexts, resource_costs))| {
                contexts.iter().map(move |context| {
                    (action_id.clone(), context.clone(), resource_costs.clone())
                })
            })
            .collect();

        let action_index = CliChoiceProvider::select_from_list(
            "Select an action to perform:",
            &actions_options,
            |(id, context, resource_costs)| format!("{}, {:?}, {:?}", id, context, resource_costs),
        );

        let selected_action = &actions_options[action_index];
        if selected_action.0 == end_turn_action_id {
            // End turn action
            engine.end_turn();
            continue;
        }

        let targeting_context = engine
            .current_character()
            .targeting_context(&selected_action.0, &selected_action.1);

        let participants = engine.participants();

        let targets = match targeting_context.kind {
            TargetingKind::SelfTarget => {
                vec![engine.current_character().id().clone()]
            }

            TargetingKind::Single => {
                let participants: Vec<_> = participants
                    .iter()
                    .filter(|character| character.id() != engine.current_character().id())
                    .collect();
                let participant_ids: Vec<_> = participants.iter().map(|c| c.id().clone()).collect();
                let participant_index = CliChoiceProvider::select_from_list(
                    "Select a target:",
                    &participant_ids,
                    |id| engine.participant(id).unwrap().name().to_string(),
                );
                vec![participant_ids[participant_index].clone()]
            }

            TargetingKind::Multiple { max_targets } => {
                let participants: Vec<_> = participants
                    .iter()
                    .filter(|character| character.id() != engine.current_character().id())
                    .collect();
                let participant_ids: HashSet<_> =
                    participants.iter().map(|c| c.id().clone()).collect();
                CliChoiceProvider::select_multiple(
                    "Select targets:",
                    &participant_ids,
                    max_targets,
                    |id| engine.participant(id).unwrap().name().to_string(),
                )
                .into_iter()
                .collect()
            }

            _ => {
                todo!("Implement targeting for other kinds of actions");
            }
        };

        let results = engine
            .submit_action(&selected_action.0, &selected_action.1, targets)
            .unwrap();
        for result in results {
            println!("{}", result);
        }
    }
}
