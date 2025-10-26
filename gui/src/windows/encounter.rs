use std::collections::{HashMap, HashSet};

use hecs::Entity;
use imgui::{ChildFlags, MouseButton};
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::{TargetType, TargetingContext, TargetingKind},
        },
        id::{ActionId, Name},
        resource::{ResourceAmountMap, ResourceMap},
        speed::Speed,
    },
    engine::{
        encounter::{Encounter, EncounterId, ParticipantsFilter},
        event::{
            ActionData, ActionDecision, ActionDecisionPartial, ActionPrompt, Event, ReactionData,
        },
        game_state::GameState,
    },
    registry,
    systems::{
        self,
        geometry::{RaycastFilter, RaycastHitKind},
    },
};

use crate::{
    render::{
        common::utils::{RenderableMutWithContext, RenderableWithContext},
        ui::{
            components::{LOW_HEALTH_BG_COLOR, LOW_HEALTH_COLOR, SPEED_COLOR, SPEED_COLOR_BG},
            engine::LogLevel,
            entities::CreatureRenderMode,
            text::{TextKind, TextSegments},
            utils::{
                ImguiRenderable, ImguiRenderableWithContext, ProgressBarColor,
                SELECTED_BUTTON_COLOR, render_button_disabled_conditionally,
                render_button_selectable, render_empty_button, render_progress_bar,
            },
        },
        world::camera::OrbitCamera,
    },
    state::gui_state::GuiState,
    table_with_columns,
    windows::anchor::{self, AUTO_RESIZE, BOTTOM_CENTER, BOTTOM_RIGHT, CENTER, WindowManager},
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

    pub fn matches_prompt(&self, prompt: &ActionPrompt) -> bool {
        match (self, prompt) {
            (
                ActionDecisionProgress::Action { actor, .. },
                ActionPrompt::Action {
                    actor: prompt_actor,
                },
            ) => actor == prompt_actor,

            (
                ActionDecisionProgress::Reactions { event, .. },
                ActionPrompt::Reactions {
                    event: prompt_event,
                    ..
                },
            ) => event.id == prompt_event.id,

            _ => false,
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

impl RenderableMutWithContext<&mut GameState> for EncounterWindow {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        game_state: &mut GameState,
    ) {
        // raw pointer sidesteps borrow checker temporarily
        let window_manager_ptr =
            unsafe { &mut *(&mut gui_state.window_manager as *mut WindowManager) };

        window_manager_ptr.render_window(
            ui,
            &format!("Encounter##{:?}", self.id),
            &anchor::CENTER_LEFT,
            AUTO_RESIZE,
            &mut true,
            || {
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
                            };
                        }
                    }

                    EncounterWindowState::EncounterRunning { decision_progress } => {
                        // First borrow: get the encounter
                        let encounter_ptr = game_state
                            .encounters
                            .get_mut(&self.id)
                            .map(|enc| enc as *mut Encounter); // raw pointer sidesteps borrow checker temporarily

                        if let Some(encounter_ptr) = encounter_ptr {
                            // SAFETY: we know no other mutable borrow of the encounter exists at this point
                            let encounter = unsafe { &mut *encounter_ptr };

                            encounter.render_mut_with_context(
                                ui,
                                gui_state,
                                (game_state, decision_progress),
                            );
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
            },
        );
    }
}

impl RenderableMutWithContext<(&mut GameState, &mut Option<ActionDecisionProgress>)> for Encounter {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        (game_state, decision_progress): (&mut GameState, &mut Option<ActionDecisionProgress>),
    ) {
        ui.separator_with_text("Participants");

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
                    ui.text(initiative.total().to_string());
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
                        .render_with_context(ui, (&game_state.world, &CreatureRenderMode::Compact));
                }
            }

            table.end();
        }

        ui.separator();
        ui.text(format!("Round: {}", self.round()));

        // TODO: Pretty much everything below this line is completely spaghetti

        let next_prompt = self.next_pending_prompt();
        if next_prompt.is_none() {
            ui.text("No action prompt available.");
            return;
        }
        let next_prompt = next_prompt.unwrap();

        // TODO: Match check is a bit of a hacky workaround for now
        if decision_progress.is_none()
            || !decision_progress
                .as_ref()
                .unwrap()
                .matches_prompt(&next_prompt)
        {
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

                    let mut end_turn = false;

                    let disabled_token = if actions_disabled {
                        // Render whatever the actual decision progress is
                        decision_progress.render_mut_with_context(
                            ui,
                            gui_state,
                            (game_state, &mut end_turn),
                        );

                        // Render placeholder action selection UI
                        let token = Some(ui.begin_disabled(actions_disabled));
                        Some(ActionDecisionProgress::from_prompt(&ActionPrompt::Action {
                            actor: current_entity,
                        }))
                        .render_mut_with_context(
                            ui,
                            gui_state,
                            (game_state, &mut end_turn),
                        );

                        token
                    } else {
                        // Render the actual action selection UI
                        decision_progress.render_mut_with_context(
                            ui,
                            gui_state,
                            (game_state, &mut end_turn),
                        );
                        None
                    };

                    ui.separator();

                    if end_turn {
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

impl RenderableMutWithContext<(&mut GameState, &mut bool)> for Option<ActionDecisionProgress> {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        (game_state, end_turn): (&mut GameState, &mut bool),
    ) {
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
                let window_manager_ptr =
                    unsafe { &mut *(&mut gui_state.window_manager as *mut WindowManager) };

                let mut cancel_action = false;

                window_manager_ptr.render_window(
                    ui,
                    format!(
                        "Actions - {}",
                        systems::helpers::get_component::<Name>(&game_state.world, *actor).as_str()
                    )
                    .as_str(),
                    &anchor::BOTTOM_CENTER,
                    AUTO_RESIZE,
                    &mut true,
                    || {
                        ui.child_window("Actions")
                            .child_flags(
                                ChildFlags::ALWAYS_AUTO_RESIZE
                                    | ChildFlags::AUTO_RESIZE_X
                                    | ChildFlags::AUTO_RESIZE_Y,
                            )
                            .build(|| {
                                ui.separator_with_text("Actions");
                                if action_options.is_empty() {
                                    *action_options = systems::actions::available_actions(
                                        &game_state.world,
                                        *actor,
                                    );
                                }
                                for (action_id, contexts_and_costs) in action_options {
                                    // Don't render reactions
                                    if contexts_and_costs.iter().all(|(_, cost)| {
                                        cost.contains_key(&registry::resources::REACTION_ID)
                                    }) {
                                        continue;
                                    }

                                    if ui.button(&action_id.to_string()) && chosen_action.is_none()
                                    {
                                        *chosen_action = Some(action_id.clone());
                                        if contexts_and_costs.len() == 1 {
                                            *chosen_context_and_cost =
                                                Some(contexts_and_costs[0].clone());
                                        } else {
                                            *context_and_cost_options = contexts_and_costs.clone();
                                        }
                                    }

                                    if ui.is_item_hovered() {
                                        ui.tooltip(|| {
                                            (action_id, contexts_and_costs).render_with_context(
                                                ui,
                                                (&game_state.world, *actor),
                                            );
                                        });
                                    }
                                }

                                ui.separator();

                                if ui.button("End Turn") {
                                    *end_turn = true;
                                }

                                if chosen_action.is_some() && chosen_context_and_cost.is_none() {
                                    gui_state.window_manager.render_window(
                                        ui,
                                        "Action Contexts",
                                        &BOTTOM_CENTER,
                                        AUTO_RESIZE,
                                        &mut true,
                                        || {
                                            for (context, cost) in context_and_cost_options {
                                                ui.same_line();

                                                if ui.button(format!("{:#?}\n{:#?}", context, cost))
                                                {
                                                    *chosen_context_and_cost =
                                                        Some((context.clone(), cost.clone()));
                                                }
                                            }
                                        },
                                    );
                                }

                                if chosen_action.is_some() && chosen_context_and_cost.is_some() {
                                    let window_manager_ptr = unsafe {
                                        &mut *(&mut gui_state.window_manager as *mut WindowManager)
                                    };

                                    window_manager_ptr.render_window(
                                        ui,
                                        "Target Selection",
                                        &BOTTOM_RIGHT,
                                        AUTO_RESIZE,
                                        &mut true,
                                        || {
                                            TextSegments::new(vec![
                                                (
                                                    systems::helpers::get_component::<Name>(
                                                        &game_state.world,
                                                        *actor,
                                                    )
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

                                            let targeting_context =
                                                systems::actions::targeting_context(
                                                    &game_state.world,
                                                    *actor,
                                                    chosen_action.as_ref().unwrap(),
                                                    &chosen_context_and_cost.as_ref().unwrap().0,
                                                );

                                            let encounter_id = game_state
                                                .encounter_for_entity(actor)
                                                .unwrap()
                                                .clone();
                                            targeting_context.render_with_context(
                                                ui,
                                                gui_state,
                                                (
                                                    game_state,
                                                    encounter_id,
                                                    targets,
                                                    targets_confirmed,
                                                ),
                                            );
                                            ui.tooltip(|| {
                                                ui.text(
                                                    chosen_action.as_ref().unwrap().to_string(),
                                                );
                                            });

                                            ui.separator();
                                            if ui.button("Confirm Targets") {
                                                *targets_confirmed = true;
                                            }
                                            ui.separator();
                                            if ui.button("Cancel Action") {
                                                cancel_action = true;
                                            }
                                        },
                                    );
                                }
                            });

                        ui.same_line();

                        ui.child_window("Resources")
                            .child_flags(
                                ChildFlags::ALWAYS_AUTO_RESIZE
                                    | ChildFlags::AUTO_RESIZE_X
                                    | ChildFlags::AUTO_RESIZE_Y,
                            )
                            .build(|| {
                                ui.separator_with_text("Resources");
                                systems::helpers::get_component::<ResourceMap>(
                                    &game_state.world,
                                    *actor,
                                )
                                .render(ui);

                                ui.separator_with_text("Speed");
                                let speed = systems::helpers::get_component::<Speed>(
                                    &game_state.world,
                                    *actor,
                                );

                                let total_speed = speed.get_total_speed();
                                let remaining_speed = speed.remaining_movement();
                                render_progress_bar(
                                    ui,
                                    remaining_speed.value,
                                    total_speed.value,
                                    remaining_speed.value / total_speed.value,
                                    150.0,
                                    "Speed",
                                    Some("m"),
                                    Some(ProgressBarColor {
                                        color_full: SPEED_COLOR,
                                        color_empty: LOW_HEALTH_COLOR,
                                        color_full_bg: SPEED_COLOR_BG,
                                        color_empty_bg: LOW_HEALTH_BG_COLOR,
                                    }),
                                );
                            });
                    },
                );

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

            ActionDecisionProgress::Reactions {
                event,
                options,
                choices,
            } => {
                let window_manager_ptr =
                    unsafe { &mut *(&mut gui_state.window_manager as *mut WindowManager) };

                window_manager_ptr.render_window(
                    ui,
                    "Reactions",
                    &CENTER,
                    AUTO_RESIZE,
                    &mut true,
                    || {
                        event.render_with_context(ui, &(&game_state.world, &LogLevel::Info));

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
                    },
                );

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

impl RenderableWithContext<(&mut GameState, EncounterId, &mut Vec<Entity>, &mut bool)>
    for TargetingContext
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        (game_state, encounter, targets, confirm_targets): (
            &mut GameState,
            EncounterId,
            &mut Vec<Entity>,
            &mut bool,
        ),
    ) {
        match &self.valid_target {
            TargetType::Entity { .. } => {
                let filter = ParticipantsFilter::from(self.valid_target.clone());
                self.kind.render_with_context(
                    ui,
                    gui_state,
                    (game_state, encounter, targets, confirm_targets, filter),
                );
            }
            TargetType::Point => todo!(),
        }
    }
}

impl
    RenderableWithContext<(
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
        gui_state: &mut GuiState,
        (game_state, encounter, targets, confirm_targets, filter): (
            &mut GameState,
            EncounterId,
            &mut Vec<Entity>,
            &mut bool,
            ParticipantsFilter,
        ),
    ) {
        let participants = game_state
            .encounter(&encounter)
            .unwrap()
            .participants(&game_state.world, filter);
        match &self {
            TargetingKind::Single => {
                ui.text("Select a single target:");

                if let Some(raycast) = &gui_state.cursor_ray_result
                    && let Some(closest) = raycast.closest()
                    && let RaycastHitKind::Creature(entity) = &closest.kind
                    && ui.is_mouse_clicked(MouseButton::Left)
                {
                    targets.clear();
                    targets.push(*entity);
                    gui_state.cursor_ray_result.take();
                }

                for entity in participants {
                    if let Ok(name) = game_state.world.query_one_mut::<&Name>(entity) {
                        if render_button_selectable(
                            ui,
                            format!("{}##{:?}", name.as_str(), entity),
                            [200.0, 20.0],
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

                if let Some(raycast) = &gui_state.cursor_ray_result
                    && let Some(closest) = raycast.closest()
                {
                    if let RaycastHitKind::Creature(entity) = &closest.kind
                        && ui.is_mouse_clicked(MouseButton::Left)
                        && targets.len() < max_targets
                    {
                        targets.push(*entity);
                        gui_state.cursor_ray_result.take();
                    }
                }

                if ui.is_mouse_clicked(MouseButton::Right) {
                    targets.pop();
                    gui_state.cursor_ray_result.take();
                }

                ui.separator_with_text(format!(
                    "Selected targets ({}/{})",
                    targets.len(),
                    max_targets
                ));
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
