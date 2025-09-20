use std::{
    collections::{HashMap, HashSet},
    thread::{sleep, sleep_ms},
    time::Duration,
};

use hecs::{Entity, World};
use imgui::{ChildFlags, TreeNodeFlags};
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::{TargetType, TargetingContext, TargetingKind},
        },
        id::{ActionId, Name},
        resource::{ResourceAmountMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
    engine::{
        encounter::{self, Encounter, EncounterId, ParticipantsFilter},
        event::{
            ActionData, ActionDecision, ActionDecisionPartial, ActionPrompt, Event, ReactionData,
        },
        game_state::{self, GameState},
    },
    registry, systems,
};
use strum::IntoEnumIterator;

use crate::{
    render::{
        engine::LogLevel,
        entities::CreatureRenderMode,
        text::{TextKind, TextSegments},
        utils::{
            ImguiRenderable, ImguiRenderableMutWithContext, ImguiRenderableWithContext,
            SELECTED_BUTTON_COLOR, render_button_disabled_conditionally, render_button_selectable,
            render_empty_button, render_window_at_cursor,
        },
    },
    table_with_columns,
};

#[derive(Debug)]
enum ActionDecisionProgress {
    Action {
        actor: Entity,
        action_options: ActionMap,
        chosen_action: Option<ActionId>,
        context_and_cost_options: Vec<(ActionContext, ResourceAmountMap)>,
        chosen_context_and_cost: Option<(ActionContext, ResourceAmountMap)>,
        targets: Vec<Entity>,
        targets_confirmed: bool,
    },
    Reactions {
        event: Event,
        options: HashMap<Entity, Vec<ReactionData>>,
        choices: HashMap<Entity, Option<ReactionData>>,
    },
}

impl ActionDecisionProgress {
    pub fn from_prompt(prompt: &ActionPrompt) -> Self {
        match prompt {
            ActionPrompt::Action { actor } => Self::action_with_actor(*actor),

            ActionPrompt::Reactions { event, options } => Self::Reactions {
                event: event.clone(),
                options: options.clone(),
                choices: HashMap::new(),
            },
        }
    }

    pub fn action_with_actor(actor: Entity) -> Self {
        Self::Action {
            actor,
            action_options: ActionMap::new(),
            chosen_action: None,
            context_and_cost_options: Vec::new(),
            chosen_context_and_cost: None,
            targets: Vec::new(),
            targets_confirmed: false,
        }
    }

    pub fn add_partial_decision(&mut self, decision: ActionDecisionPartial) {
        match (self, decision) {
            (
                ActionDecisionProgress::Action {
                    actor,
                    chosen_action,
                    chosen_context_and_cost,
                    targets,
                    targets_confirmed,
                    ..
                },
                ActionDecisionPartial::Action {
                    action:
                        ActionData {
                            actor: decision_actor,
                            action_id,
                            context,
                            resource_cost,
                            targets: decision_targets,
                        },
                },
            ) => {
                assert_eq!(*actor, decision_actor);
                *chosen_action = Some(action_id);
                *chosen_context_and_cost = Some((context, resource_cost));
                *targets = decision_targets;
                *targets_confirmed = true;
            }

            (
                ActionDecisionProgress::Reactions { choices, .. },
                ActionDecisionPartial::Reaction {
                    reactor, choice, ..
                },
            ) => {
                choices.insert(reactor, choice);
            }

            _ => {
                panic!("Mismatched decision type");
            }
        }
    }

    pub fn actors(&self) -> Vec<Entity> {
        match self {
            ActionDecisionProgress::Action { actor, .. } => vec![*actor],
            ActionDecisionProgress::Reactions { options, .. } => options.keys().cloned().collect(),
        }
    }

    pub fn decision_from(&self, entity: Entity) -> Option<ActionDecisionPartial> {
        match self {
            ActionDecisionProgress::Action {
                actor,
                chosen_action,
                chosen_context_and_cost,
                targets,
                targets_confirmed,
                ..
            } => {
                if *actor == entity
                    && chosen_action.is_some()
                    && chosen_context_and_cost.is_some()
                    && *targets_confirmed
                {
                    let (context, resource_cost) =
                        chosen_context_and_cost.as_ref().unwrap().clone();
                    Some(ActionDecisionPartial::Action {
                        action: ActionData {
                            actor: *actor,
                            action_id: chosen_action.as_ref().unwrap().clone(),
                            context,
                            resource_cost,
                            targets: targets.clone(),
                        },
                    })
                } else {
                    None
                }
            }

            ActionDecisionProgress::Reactions { choices, event, .. } => {
                if let Some(choice) = choices.get(&entity) {
                    Some(ActionDecisionPartial::Reaction {
                        reactor: entity,
                        event: event.clone(),
                        choice: choice.clone(),
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn has_decision_from(&self, entity: Entity) -> bool {
        self.decision_from(entity).is_some()
    }

    pub fn is_complete(&self) -> bool {
        match self {
            ActionDecisionProgress::Action {
                chosen_action,
                chosen_context_and_cost: chosen_context,
                targets_confirmed,
                ..
            } => chosen_action.is_some() && chosen_context.is_some() && *targets_confirmed,

            ActionDecisionProgress::Reactions {
                choices, options, ..
            } => {
                // All reactors must have made a choice
                for (reactor, reactor_options) in options {
                    if !choices.contains_key(reactor) {
                        return false;
                    }

                    // If they made a choice, it must be valid
                    if let Some(choice) = &choices[reactor] {
                        if !reactor_options.iter().any(|opt| opt == choice) {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    pub fn finalize(self) -> ActionDecision {
        match self {
            ActionDecisionProgress::Action {
                actor,
                chosen_action,
                chosen_context_and_cost,
                targets,
                ..
            } => {
                let (context, resource_cost) = chosen_context_and_cost.unwrap();
                ActionDecision::Action {
                    action: ActionData {
                        actor: actor.clone(),
                        action_id: chosen_action.unwrap(),
                        context,
                        resource_cost,
                        targets: targets.clone(),
                    },
                }
            }
            ActionDecisionProgress::Reactions { event, choices, .. } => {
                ActionDecision::Reactions { event, choices }
            }
        }
    }
}

enum EncounterWindowState {
    EncounterCreation {
        participants: HashSet<Entity>,
    },
    EncounterRunning {
        decision_progress: Option<ActionDecisionProgress>,
        auto_scroll_combat_log: bool,
        log_render_level: LogLevel,
    },
    EncounterFinished,
}

pub struct EncounterWindow {
    state: EncounterWindowState,
    id: EncounterId,
}

impl EncounterWindow {
    pub fn new() -> Self {
        Self {
            state: EncounterWindowState::EncounterCreation {
                participants: HashSet::new(),
            },
            id: EncounterId::new_v4(),
        }
    }

    pub fn id(&self) -> &EncounterId {
        &self.id
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, EncounterWindowState::EncounterFinished)
    }
}

impl ImguiRenderableMutWithContext<&mut GameState> for EncounterWindow {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, game_state: &mut GameState) {
        match &mut self.state {
            EncounterWindowState::EncounterCreation { participants } => {
                ui.separator_with_text("Encounter creation");
                ui.text("Select participants:");

                game_state
                    .world
                    .query::<&Name>()
                    .into_iter()
                    .for_each(|(entity, name)| {
                        let is_selected = participants.contains(&entity);
                        if render_button_selectable(
                            ui,
                            format!("{}##{:?}", name.as_str(), entity),
                            [100.0, 20.0],
                            is_selected,
                        ) {
                            if is_selected {
                                participants.remove(&entity);
                            } else {
                                participants.insert(entity);
                            }
                        }
                    });

                ui.separator();
                if render_button_disabled_conditionally(
                    ui,
                    "Start Encounter",
                    [0.0, 0.0],
                    participants.len() < 2,
                    "You must have at least two participants to start an encounter.",
                ) {
                    game_state.start_encounter_with_id(participants.clone(), self.id);
                    self.state = EncounterWindowState::EncounterRunning {
                        decision_progress: None,
                        auto_scroll_combat_log: true,
                        log_render_level: LogLevel::Info,
                    };
                }
            }

            EncounterWindowState::EncounterRunning {
                decision_progress,
                auto_scroll_combat_log,
                log_render_level,
            } => {
                // First borrow: get the encounter
                let encounter_ptr = game_state
                    .encounters
                    .get_mut(&self.id)
                    .map(|enc| enc as *mut Encounter); // raw pointer sidesteps borrow checker temporarily

                if let Some(encounter_ptr) = encounter_ptr {
                    // Now safe to mutably borrow world
                    // let world = &mut game_state.world;

                    // SAFETY: we know no other mutable borrow of the encounter exists at this point
                    let encounter = unsafe { &mut *encounter_ptr };

                    ui.child_window(format!("Encounter: {}", self.id))
                        .child_flags(
                            ChildFlags::ALWAYS_AUTO_RESIZE
                                | ChildFlags::AUTO_RESIZE_X
                                | ChildFlags::AUTO_RESIZE_Y,
                        )
                        .build(|| {
                            encounter.render_mut_with_context(ui, (game_state, decision_progress));
                        });

                    ui.same_line();
                    ui.child_window(format!("Combat Log: {}", self.id))
                        .child_flags(
                            ChildFlags::ALWAYS_AUTO_RESIZE
                                | ChildFlags::AUTO_RESIZE_X
                                | ChildFlags::AUTO_RESIZE_Y,
                        )
                        .build(|| {
                            ui.separator_with_text("Combat Log");

                            ui.child_window("Combat Log Content")
                                .child_flags(
                                    ChildFlags::ALWAYS_AUTO_RESIZE
                                        | ChildFlags::AUTO_RESIZE_X
                                        | ChildFlags::BORDERS,
                                )
                                .size([0.0, 500.0])
                                .build(|| {
                                    encounter.combat_log().render_with_context(
                                        ui,
                                        &(&game_state.world, log_render_level),
                                    );

                                    if *auto_scroll_combat_log
                                        && ui.scroll_y() >= ui.scroll_max_y() - 5.0
                                    {
                                        ui.set_scroll_here_y_with_ratio(1.0);
                                    }
                                });

                            ui.checkbox("Auto-scroll", auto_scroll_combat_log);

                            // let mut current_log_level = log_render_level.clone() as i32;
                            // let log_level_options = LogRenderLevel::iter()
                            //     .map(|lvl| lvl.to_string())
                            //     .collect::<Vec<String>>();
                            // let log_level_options_str: Vec<&str> =
                            //     log_level_options.iter().map(|s| s.as_str()).collect();
                            // ui.list_box(
                            //     "Log level",
                            //     &mut current_log_level,
                            //     &log_level_options_str[..],
                            //     3,
                            // );
                            // *log_render_level = LogRenderLevel::from(current_log_level);

                            let mut current_log_level = log_render_level.clone() as usize;
                            let width_token = ui.push_item_width(60.0);
                            if ui.combo(
                                "Log level",
                                &mut current_log_level,
                                &LogLevel::iter().collect::<Vec<_>>()[..],
                                |lvl| lvl.to_string().into(),
                            ) {
                                *log_render_level = LogLevel::from(current_log_level);
                            }
                            width_token.end();
                        });

                    ui.same_line();
                    ui.child_window(format!("Encounter Debug##{}", self.id))
                        .child_flags(
                            ChildFlags::ALWAYS_AUTO_RESIZE
                                | ChildFlags::AUTO_RESIZE_X
                                | ChildFlags::AUTO_RESIZE_Y,
                        )
                        .build(|| {
                            ui.separator_with_text("Encounter Debug Info");

                            ui.child_window("Debug Info Content")
                                .child_flags(
                                    ChildFlags::ALWAYS_AUTO_RESIZE
                                        | ChildFlags::AUTO_RESIZE_X
                                        | ChildFlags::BORDERS,
                                )
                                .size([0.0, 500.0])
                                .build(|| {
                                    ui.text(self.id.to_string());

                                    if ui.collapsing_header("Participants", TreeNodeFlags::FRAMED) {
                                        for participant in encounter.participants(
                                            &game_state.world,
                                            ParticipantsFilter::All,
                                        ) {
                                            let name =
                                                systems::helpers::get_component_clone::<Name>(
                                                    &game_state.world,
                                                    participant,
                                                )
                                                .to_string();
                                            ui.text(format!("{} ({:?})", name, participant));
                                        }
                                    }

                                    if ui
                                        .collapsing_header("Pending Prompts", TreeNodeFlags::FRAMED)
                                    {
                                        for prompt in encounter.pending_prompts() {
                                            ui.text(format!("{:#?}", prompt));
                                        }
                                    }

                                    if ui.collapsing_header(
                                        "Decision Progress",
                                        TreeNodeFlags::FRAMED,
                                    ) {
                                        if let Some(progress) = decision_progress {
                                            for actor in progress.actors() {
                                                let name = systems::helpers::get_component_clone::<
                                                    Name,
                                                >(
                                                    &game_state.world, actor
                                                )
                                                .to_string();

                                                if let Some(decision) =
                                                    progress.decision_from(actor)
                                                {
                                                    ui.text(format!(
                                                        "{}'s decision: {:#?}",
                                                        name, decision
                                                    ));
                                                } else {
                                                    ui.text(format!(
                                                        "{} has not decided yet",
                                                        name
                                                    ));
                                                }

                                                ui.separator();
                                            }

                                            ui.text(format!("{:#?}", progress));
                                        } else {
                                            ui.text("No decision in progress");
                                        }
                                    }
                                });
                        });
                } else {
                    ui.text("Encounter not found!");
                }

                ui.separator();
                if ui.button("End Encounter") {
                    self.state = EncounterWindowState::EncounterFinished;
                    game_state.end_encounter(&self.id);
                }
            }

            EncounterWindowState::EncounterFinished => {
                ui.text("Encounter finished!");
            }
        }
    }
}

impl ImguiRenderableMutWithContext<(&mut GameState, &mut Option<ActionDecisionProgress>)>
    for Encounter
{
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        context: (&mut GameState, &mut Option<ActionDecisionProgress>),
    ) {
        ui.separator_with_text("Participants");
        let (game_state, decision_progress) = context;

        let initiative_order = self.initiative_order();
        let current_entity = self.current_entity();
        let current_name =
            systems::helpers::get_component_clone::<Name>(&game_state.world, current_entity)
                .to_string();

        if let Some(table) = table_with_columns!(ui, "Initiative Order", "", "Participant",) {
            for (entity, initiative) in initiative_order {
                if let Ok(_) = &game_state.world.query_one_mut::<&Name>(*entity) {
                    // Initiative column
                    ui.table_next_column();
                    ui.text(initiative.total.to_string());
                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.separator_with_text("Initiative");
                            initiative.render(ui);
                        });
                    }

                    if current_entity == *entity {
                        ui.table_set_bg_color(imgui::TableBgTarget::all(), SELECTED_BUTTON_COLOR);
                    }

                    // Participant column
                    ui.table_next_column();
                    entity
                        .render_with_context(ui, (&game_state.world, CreatureRenderMode::Compact));
                }
            }

            table.end();
        }

        ui.separator();
        ui.text(format!("Round: {}", self.round()));

        let next_prompt = self.next_pending_prompt();
        if next_prompt.is_none() {
            ui.text("No action prompt available.");
            return;
        }
        let next_prompt = next_prompt.unwrap();

        if decision_progress.is_none() {
            *decision_progress = Some(ActionDecisionProgress::from_prompt(&next_prompt));
        }

        // Check which actors have note yet made a decision
        let actors = next_prompt.actors();
        let mut rendered_player_ui = false;
        for actor in actors {
            // TODO: Bit of a hacky workaround for the fact that the decision
            // progress gets cleared when an action is submitted during the
            // loop
            if decision_progress.is_none() {
                break;
            }

            if !decision_progress.as_ref().unwrap().has_decision_from(actor) {
                // Check if any of the actors are AI controlled
                if !systems::ai::is_player_controlled(&game_state.world, actor) {
                    if let Some(decision) =
                        systems::ai::decide_action(&game_state.world, self, &next_prompt, actor)
                    {
                        println!("AI decided on action: {:?}", decision);
                        decision_progress
                            .as_mut()
                            .unwrap()
                            .add_partial_decision(decision);
                        if decision_progress.as_ref().unwrap().is_complete() {
                            let finalized_decision = decision_progress.take().unwrap().finalize();
                            let result = game_state.submit_decision(finalized_decision);
                            match result {
                                Ok(event) => {
                                    println!("Action processed successfully: {:?}", event);
                                }
                                Err(err) => {
                                    println!("Error processing action: {:?}", err);
                                }
                            }
                        }
                    } else {
                        // TODO: This is probably going to break something at some point...
                        println!(
                            "AI could not decide on action for prompt: {:?}. Assuming end turn.",
                            next_prompt
                        );
                        decision_progress.take();
                        self.end_turn(game_state, actor);
                        break;
                    }
                } else {
                    if rendered_player_ui {
                        continue;
                    } else {
                        rendered_player_ui = true;
                    }

                    // For the sake of visual clarity the current entity's available actions
                    // are always rendered. In the event of a reaction prompt, the actions
                    // will then be disabled. This requires a little bit of special treatment

                    let actions_disabled = !matches!(
                        decision_progress.as_ref().unwrap(),
                        ActionDecisionProgress::Action { .. }
                    );

                    ui.separator_with_text(format!("Current turn: {}", current_name));

                    let disabled_token = if actions_disabled {
                        // Render whatever the actual decision progress is
                        decision_progress.render_mut_with_context(ui, game_state);

                        // Render placeholder action selection UI
                        let token = Some(ui.begin_disabled(actions_disabled));
                        Some(ActionDecisionProgress::from_prompt(&ActionPrompt::Action {
                            actor: current_entity,
                        }))
                        .render_mut_with_context(ui, game_state);

                        token
                    } else {
                        // Render the actual action selection UI
                        decision_progress.render_mut_with_context(ui, game_state);
                        None
                    };

                    ui.separator();

                    if ui.button("End Turn") {
                        decision_progress.take(); // Clear decision progress
                        self.end_turn(game_state, current_entity);
                        break;
                    }

                    if let Some(token) = disabled_token {
                        token.end();
                    }
                }
            }
        }
    }
}

impl ImguiRenderableMutWithContext<&mut GameState> for Option<ActionDecisionProgress> {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, game_state: &mut GameState) {
        if self.is_none() {
            ui.text("No action decision in progress.");
            return;
        }

        match self.as_mut().unwrap() {
            ActionDecisionProgress::Action {
                actor,
                action_options,
                chosen_action,
                context_and_cost_options,
                chosen_context_and_cost,
                targets,
                targets_confirmed,
            } => {
                ui.separator_with_text("Resources");
                systems::helpers::get_component::<ResourceMap>(&game_state.world, *actor)
                    .render(ui);

                ui.separator_with_text("Actions");
                if action_options.is_empty() {
                    *action_options =
                        systems::actions::available_actions(&game_state.world, *actor);
                }
                for (action_id, contexts_and_costs) in action_options {
                    // Don't render reactions
                    if contexts_and_costs
                        .iter()
                        .all(|(_, cost)| cost.contains_key(&registry::resources::REACTION_ID))
                    {
                        continue;
                    }

                    if ui.button(&action_id.to_string()) && chosen_action.is_none() {
                        *chosen_action = Some(action_id.clone());
                        if contexts_and_costs.len() == 1 {
                            *chosen_context_and_cost = Some(contexts_and_costs[0].clone());
                        } else {
                            *context_and_cost_options = contexts_and_costs.clone();
                        }
                    }
                }

                if chosen_action.is_some() && chosen_context_and_cost.is_none() {
                    render_window_at_cursor(ui, "Action Contexts", true, || {
                        for (context, cost) in context_and_cost_options {
                            if ui.button(format!("{:#?}\n{:#?}", context, cost)) {
                                *chosen_context_and_cost = Some((context.clone(), cost.clone()));
                            }
                        }
                    });
                }

                let mut cancel_action = false;
                if chosen_action.is_some() && chosen_context_and_cost.is_some() {
                    render_window_at_cursor(ui, "Target Selection", true, || {
                        TextSegments::new(vec![
                            (
                                systems::helpers::get_component::<Name>(&game_state.world, *actor)
                                    .to_string(),
                                TextKind::Actor,
                            ),
                            ("is using".to_string(), TextKind::Normal),
                            (
                                chosen_action.as_ref().unwrap().to_string(),
                                TextKind::Action,
                            ),
                        ])
                        .render(ui);

                        let targeting_context = systems::actions::targeting_context(
                            &game_state.world,
                            *actor,
                            chosen_action.as_ref().unwrap(),
                            &chosen_context_and_cost.as_ref().unwrap().0,
                        );

                        let encounter_id = game_state.encounter_for_entity(actor).unwrap().clone();
                        targeting_context.render_with_context(
                            ui,
                            (game_state, encounter_id, targets, targets_confirmed),
                        );

                        ui.separator();
                        if ui.button("Confirm Targets") {
                            *targets_confirmed = true;
                        }
                        ui.separator();
                        if ui.button("Cancel Action") {
                            cancel_action = true;
                        }
                    });

                    if *targets_confirmed {
                        let decision = self.take().unwrap().finalize();
                        let result = game_state.submit_decision(decision);
                        match result {
                            Ok(event) => {
                                println!("Action processed successfully: {:?}", event);
                            }
                            Err(err) => {
                                println!("Error processing action: {:?}", err);
                            }
                        }
                    }

                    if cancel_action {
                        self.take().unwrap();
                    }
                }
            }

            ActionDecisionProgress::Reactions {
                event,
                options,
                choices,
            } => {
                render_window_at_cursor(ui, "Reactions", true, || {
                    // TODO: Render the event that triggered the reaction
                    event.render_with_context(ui, &(&game_state.world, &LogLevel::Info));

                    // TextSegments::new(vec![
                    //     (
                    //         systems::helpers::get_component_clone::<Name>(world, action.actor)
                    //             .to_string(),
                    //         TextKind::Actor,
                    //     ),
                    //     ("used".to_string(), TextKind::Normal),
                    //     (action.action_id.to_string(), TextKind::Action),
                    //     ("on".to_string(), TextKind::Normal),
                    //     (
                    //         action
                    //             .targets
                    //             .iter()
                    //             .map(|t| {
                    //                 systems::helpers::get_component_clone::<Name>(world, *t)
                    //                     .to_string()
                    //             })
                    //             .collect::<Vec<_>>()
                    //             .join(", "),
                    //         TextKind::Target,
                    //     ),
                    // ])
                    // .render(ui);

                    ui.text("Choose how to react:");

                    for (reactor, options) in options {
                        ui.separator_with_text(
                            systems::helpers::get_component_clone::<Name>(
                                &game_state.world,
                                *reactor,
                            )
                            .as_str(),
                        );

                        for option in options {
                            let option_selected = if let Some(choice) = choices.get(reactor) {
                                choice.as_ref() == Some(option)
                            } else {
                                false
                            };

                            if render_button_selectable(
                                ui,
                                // format!(
                                //     "{}: {:#?}\nCost: {:#?}##{:?}",
                                //     option.reaction_id,
                                //     option.context,
                                //     option.resource_cost,
                                //     reactor
                                // ),
                                format!(
                                    "{}:\nCost: {:#?}##{:?}",
                                    option.reaction_id, option.resource_cost, reactor
                                ),
                                [0., 0.],
                                option_selected,
                            ) {
                                choices.insert(*reactor, Some(option.clone()));
                            }

                            if ui.button(format!("Don't react##{:?}", reactor)) {
                                choices.insert(*reactor, None);
                            }
                        }
                    }

                    ui.separator();
                });

                if self.as_ref().unwrap().is_complete() {
                    let decision = self.take().unwrap().finalize();
                    let result = game_state.submit_decision(decision).unwrap();
                    match result {
                        _ => {
                            println!("{:?}", result);
                        }
                    }
                }
            }
        }
    }
}

impl ImguiRenderableWithContext<(&mut GameState, EncounterId, &mut Vec<Entity>, &mut bool)>
    for TargetingContext
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        context: (&mut GameState, EncounterId, &mut Vec<Entity>, &mut bool),
    ) {
        let (game_state, encounter, targets, confirm_targets) = context;

        for target_type in &self.valid_target_types {
            match target_type {
                TargetType::Entity { .. } => {
                    let filter = ParticipantsFilter::from(target_type.clone());
                    self.kind.render_with_context(
                        ui,
                        (game_state, encounter, targets, confirm_targets, filter),
                    );
                }
            }
        }
    }
}

impl
    ImguiRenderableWithContext<(
        &mut GameState,
        EncounterId,
        &mut Vec<Entity>,
        &mut bool,
        ParticipantsFilter,
    )> for TargetingKind
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        context: (
            &mut GameState,
            EncounterId,
            &mut Vec<Entity>,
            &mut bool,
            ParticipantsFilter,
        ),
    ) {
        let (game_state, encounter, targets, confirm_targets, filter) = context;

        let participants = game_state
            .encounter(&encounter)
            .unwrap()
            .participants(&game_state.world, filter);
        match &self {
            TargetingKind::Single => {
                ui.text("Select a single target:");
                for entity in participants {
                    if let Ok(name) = game_state.world.query_one_mut::<&Name>(entity) {
                        if render_button_selectable(
                            ui,
                            format!("{}##{:?}", name.as_str(), entity),
                            [100.0, 20.0],
                            targets.contains(&entity),
                        ) {
                            if targets.len() > 0 {
                                targets.clear();
                            }
                            targets.push(entity);
                        }
                    }
                }
            }

            TargetingKind::Multiple { max_targets } => {
                let max_targets = *max_targets as usize;
                ui.text(format!(
                    "Selected {}/{} targets:",
                    targets.len(),
                    max_targets
                ));
                ui.separator_with_text("Possible targets");
                for entity in participants {
                    if let Ok(name) = game_state.world.query_one_mut::<&Name>(entity) {
                        if ui.button(format!("{}##{:?}", name.as_str(), entity))
                            && targets.len() < max_targets
                        {
                            targets.push(entity);
                        }
                    }
                }
                ui.separator_with_text("Selected targets");
                let mut remove_target = None;
                for (i, target) in (&mut *targets).iter().enumerate() {
                    if let Ok(name) = game_state.world.query_one_mut::<&Name>(*target) {
                        if ui.button(format!("{}##{}", name.as_str(), i)) {
                            remove_target = Some(i);
                        }
                    }
                }
                for i in targets.len()..max_targets {
                    render_empty_button(ui, &format!("Empty##{}", i));
                }
                if let Some(target_index) = remove_target {
                    targets.remove(target_index);
                }
            }

            TargetingKind::SelfTarget => {
                targets.push(game_state.encounter(&encounter).unwrap().current_entity());
                *confirm_targets = true;
            }

            _ => {
                ui.text(format!("Targeting kind {:?} is not implemented yet.", self));
            }
        }
    }
}
