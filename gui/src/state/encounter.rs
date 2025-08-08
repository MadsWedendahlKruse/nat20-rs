use std::collections::HashSet;

use hecs::{Entity, World};
use imgui::{ChildFlags, TreeNodeFlags};
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::TargetingKind,
        },
        id::{ActionId, EncounterId},
        resource::ResourceMap,
    },
    engine::{
        encounter::{
            ActionData, ActionDecision, ActionDecisionResult, ActionPrompt, CombatLog, Encounter,
            ReactionData,
        },
        game_state::GameState,
    },
    entities::character::CharacterTag,
    systems,
};

use crate::{
    render::{
        text::{TextKind, TextSegment, TextSegments},
        utils::{
            ImguiRenderable, ImguiRenderableMutWithContext, ImguiRenderableWithContext,
            render_button_disabled_conditionally, render_button_selectable,
            render_window_at_cursor,
        },
    },
    table_with_columns,
};

enum ActionDecisionProgress {
    Action {
        actor: Entity,
        action_options: ActionMap,
        chosen_action: Option<ActionId>,
        context_options: Vec<ActionContext>,
        chosen_context: Option<ActionContext>,
        targets: Vec<Entity>,
    },
    Reaction {
        reactor: Entity,
        action: ActionData,
        choice: Option<ReactionData>,
    },
}

impl ActionDecisionProgress {
    pub fn from_prompt(prompt: &ActionPrompt) -> Self {
        match prompt {
            ActionPrompt::Action { actor } => Self::Action {
                actor: *actor,
                action_options: ActionMap::new(),
                chosen_action: None,
                context_options: Vec::new(),
                chosen_context: None,
                targets: Vec::new(),
            },
            ActionPrompt::Reaction {
                reactor,
                action,
                options,
            } => Self::Reaction {
                reactor: *reactor,
                action: action.clone(),
                choice: options.first().cloned(),
            },
        }
    }

    pub fn finalize(self) -> ActionDecision {
        match self {
            ActionDecisionProgress::Action {
                actor,
                action_options,
                chosen_action,
                context_options,
                chosen_context,
                targets,
            } => ActionDecision::Action {
                action: ActionData {
                    actor: actor.clone(),
                    action_id: chosen_action.unwrap(),
                    context: chosen_context.unwrap(),
                    targets: targets.clone(),
                },
            },
            ActionDecisionProgress::Reaction {
                reactor,
                action,
                choice,
            } => ActionDecision::Reaction {
                reactor,
                action,
                choice,
            },
        }
    }
}

enum EncounterGuiState {
    EncounterCreation {
        participants: HashSet<Entity>,
    },
    EncounterRunning {
        decision_progress: Option<ActionDecisionProgress>,
        auto_scroll_combat_log: bool,
    },
    EncounterFinished,
}

pub struct EncounterGui {
    state: EncounterGuiState,
    id: EncounterId,
}

impl EncounterGui {
    pub fn new() -> Self {
        Self {
            state: EncounterGuiState::EncounterCreation {
                participants: HashSet::new(),
            },
            id: EncounterId::new_v4(),
        }
    }

    pub fn id(&self) -> &EncounterId {
        &self.id
    }

    pub fn finished(&self) -> bool {
        matches!(self.state, EncounterGuiState::EncounterFinished)
    }
}

impl ImguiRenderableMutWithContext<&mut GameState> for EncounterGui {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, game_state: &mut GameState) {
        match &mut self.state {
            EncounterGuiState::EncounterCreation { participants } => {
                ui.separator_with_text("Encounter creation");
                ui.text("Select participants:");
                let characters = game_state
                    .world
                    .query_mut::<(&String, &CharacterTag)>()
                    .into_iter()
                    .map(|(entity, (name, tag))| (entity, name.clone(), tag.clone()))
                    .collect::<Vec<_>>();
                for (entity, name, tag) in characters {
                    let is_selected = participants.contains(&entity);
                    if render_button_selectable(ui, name, [100.0, 20.0], is_selected) {
                        if is_selected {
                            participants.remove(&entity);
                        } else {
                            participants.insert(entity);
                        }
                    }
                }

                ui.separator();
                if render_button_disabled_conditionally(
                    ui,
                    "Start Encounter",
                    participants.len() < 2,
                    "You must have at least two participants to start an encounter.",
                ) {
                    game_state.start_encounter_with_id(participants.clone(), self.id);
                    self.state = EncounterGuiState::EncounterRunning {
                        decision_progress: None,
                        auto_scroll_combat_log: true,
                    };
                }
            }

            EncounterGuiState::EncounterRunning {
                decision_progress,
                auto_scroll_combat_log,
            } => {
                // First borrow: get the encounter
                let encounter_ptr = game_state
                    .encounters
                    .get_mut(&self.id)
                    .map(|enc| enc as *mut Encounter); // raw pointer sidesteps borrow checker temporarily

                if let Some(encounter_ptr) = encounter_ptr {
                    // Now safe to mutably borrow world
                    let world = &mut game_state.world;

                    // SAFETY: we know no other mutable borrow of the encounter exists at this point
                    let encounter = unsafe { &mut *encounter_ptr };

                    ui.child_window(format!("Encounter: {}", self.id))
                        .child_flags(
                            ChildFlags::ALWAYS_AUTO_RESIZE
                                | ChildFlags::AUTO_RESIZE_X
                                | ChildFlags::AUTO_RESIZE_Y,
                        )
                        .build(|| {
                            encounter.render_mut_with_context(ui, (world, decision_progress));
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
                                    encounter.combat_log().render_with_context(ui, world);

                                    if *auto_scroll_combat_log
                                        && ui.scroll_y() >= ui.scroll_max_y() - 5.0
                                    {
                                        ui.set_scroll_here_y_with_ratio(1.0);
                                    }
                                });

                            ui.checkbox("Auto-scroll", auto_scroll_combat_log);
                        });
                } else {
                    ui.text("Encounter not found!");
                }

                ui.separator();
                if ui.button("End Encounter") {
                    self.state = EncounterGuiState::EncounterFinished;
                }
            }

            EncounterGuiState::EncounterFinished => {
                // Handle finished encounter state
                ui.text("Encounter finished!");
                game_state.end_encounter(&self.id);
            }
        }
    }
}

impl ImguiRenderableMutWithContext<(&mut World, &mut Option<ActionDecisionProgress>)>
    for Encounter
{
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        context: (&mut World, &mut Option<ActionDecisionProgress>),
    ) {
        ui.separator_with_text("Participants");
        let (world, decision_progress) = context;

        let initiative_order = self.initiative_order();
        let current_entity = self.current_entity();
        let current_name = systems::helpers::get_component_clone::<String>(world, current_entity);

        if let Some(table) =
            table_with_columns!(ui, "Initiative Order", "Initiative", "Participant")
        {
            for (entity, initiative) in initiative_order {
                if let Ok((name, tag)) = world.query_one_mut::<(&String, &CharacterTag)>(*entity) {
                    // Initiative column
                    ui.table_next_column();
                    ui.text(initiative.total.to_string());
                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            initiative.render(ui);
                        });
                    }
                    if self.current_entity() == *entity {
                        ui.table_set_bg_color(imgui::TableBgTarget::all(), [0.2, 0.2, 0.7, 1.0]);
                    }
                    // Participant column
                    ui.table_next_column();
                    if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                        (*entity, tag.clone()).render_mut_with_context(ui, world);
                    }
                }
            }

            table.end();
        }

        ui.separator();
        ui.text(format!("Round: {}", self.round));

        let next_prompt = self.next_prompt();
        if next_prompt.is_none() {
            ui.text("No actions pending");
            return;
        }
        let next_prompt = next_prompt.unwrap();
        if decision_progress.is_none() {
            *decision_progress = Some(ActionDecisionProgress::from_prompt(next_prompt));
        }

        let reaction_prompt_active = match next_prompt {
            ActionPrompt::Reaction { options, .. } => {
                render_window_at_cursor(ui, "Reaction", true, || {});
                true
            }
            _ => false,
        };

        let disabled_token = ui.begin_disabled(reaction_prompt_active);

        ui.separator_with_text(format!("Current turn: {}", current_name));

        match decision_progress.as_mut().unwrap() {
            ActionDecisionProgress::Action {
                actor: _,
                action_options,
                chosen_action,
                context_options,
                chosen_context,
                targets,
            } => {
                systems::helpers::get_component::<ResourceMap>(world, current_entity).render(ui);

                if action_options.is_empty() {
                    *action_options =
                        systems::actions::available_actions(world, self.current_entity());
                }
                for (action_id, (contexts, resource_cost)) in action_options {
                    if ui.button(&action_id.to_string()) && chosen_action.is_none() {
                        *chosen_action = Some(action_id.clone());
                        if contexts.len() == 1 {
                            *chosen_context = Some(contexts[0].clone());
                        } else {
                            *context_options = contexts.clone();
                        }
                    }
                }

                if chosen_action.is_some() && chosen_context.is_none() {
                    render_window_at_cursor(ui, "Action Contexts", true, || {
                        for context in context_options {
                            if ui.button(format!("{:?}", context)) {
                                *chosen_context = Some(context.clone());
                            }
                        }
                    });
                }

                let mut confirm_targets = false;
                if chosen_action.is_some() && chosen_context.is_some() {
                    render_window_at_cursor(ui, "Target Selection", true, || {
                        let targeting_context = systems::actions::targeting_context(
                            world,
                            self.current_entity(),
                            chosen_action.as_ref().unwrap(),
                            chosen_context.as_ref().unwrap(),
                        );
                        match targeting_context.kind {
                            TargetingKind::Single => {
                                ui.text("Select a single target:");
                                for entity in self.participants() {
                                    if let Ok(name) = world.query_one_mut::<&String>(*entity) {
                                        if render_button_selectable(
                                            ui,
                                            name.clone(),
                                            [100.0, 20.0],
                                            targets.contains(entity),
                                        ) {
                                            if targets.len() > 0 {
                                                targets.clear();
                                            }
                                            targets.push(*entity);
                                        }
                                    }
                                }
                            }

                            TargetingKind::Multiple { max_targets } => {
                                ui.text(format!(
                                    "Selected {}/{} targets:",
                                    targets.len(),
                                    max_targets
                                ));
                                ui.separator_with_text("Possible targets");
                                for entity in self.participants() {
                                    if let Ok(name) = world.query_one_mut::<&String>(*entity) {
                                        if ui.button(name.clone())
                                            && targets.len() < max_targets.into()
                                        {
                                            targets.push(*entity);
                                        }
                                    }
                                }
                                ui.separator_with_text("Selected targets");
                                let mut remove_target = None;
                                for (i, target) in (&mut *targets).iter().enumerate() {
                                    if let Ok(name) = world.query_one_mut::<&String>(*target) {
                                        if ui.button(format!("{}##{}", name, i)) {
                                            remove_target = Some(target.clone());
                                        }
                                    }
                                }
                                if let Some(target) = remove_target {
                                    targets.retain(|&e| e != target);
                                }
                            }

                            TargetingKind::SelfTarget => {
                                targets.push(current_entity);
                                confirm_targets = true;
                            }

                            _ => {
                                ui.text(format!(
                                    "Targeting kind {:?} is not implemented yet.",
                                    targeting_context.kind
                                ));
                            }
                        }

                        ui.separator();
                        if ui.button("Confirm Targets") {
                            confirm_targets = true;
                        }
                    });

                    if confirm_targets {
                        let decision = decision_progress.take().unwrap().finalize();
                        let result = self.process(world, decision).unwrap();
                        match result {
                            ActionDecisionResult::ActionPerformed { action, results } => {
                                // Handle action performed, e.g., apply effects, update state
                                println!("Action performed: {:?}", action);
                                for result in results {
                                    println!("{}", result);
                                }
                            }
                            _ => {
                                println!("{:?}", result);
                            }
                        }
                    }
                }
            }

            _ => {
                ui.text(format!("{:?} is not implemented yet :^)", next_prompt));
            }
        }

        ui.separator();

        if ui.button("End Turn") {
            decision_progress.take(); // Clear decision progress
            self.end_turn(world, current_entity);
        }

        disabled_token.end();
    }
}

impl ImguiRenderableWithContext<&World> for CombatLog {
    fn render_with_context(&self, ui: &imgui::Ui, world: &World) {
        for entry in self {
            match entry {
                ActionDecisionResult::ActionPerformed { action, results } => {
                    TextSegments::new(vec![
                        (
                            &systems::helpers::get_component::<String>(world, action.actor)
                                .to_string(),
                            TextKind::Actor,
                        ),
                        (&"used".to_string(), TextKind::Normal),
                        (&action.action_id.to_string(), TextKind::Action),
                    ])
                    .render(ui);

                    if action.targets.len() == 1 && action.targets[0] != action.actor {
                        ui.same_line();
                        TextSegments::new(vec![
                            ("on", TextKind::Normal),
                            (
                                &systems::helpers::get_component::<String>(
                                    world,
                                    action.targets[0],
                                ),
                                TextKind::Target,
                            ),
                        ])
                        .render(ui);
                    }

                    for result in results {
                        result.render_with_context(ui, 0);
                    }
                }
                ActionDecisionResult::ReactionTriggered { reactor, action } => {
                    ui.text(format!("Reaction triggered: {:?}", action));
                }
                ActionDecisionResult::ActionCancelled { reaction, action } => {
                    ui.text(format!("Action cancelled: {:?}", action));
                }
                _ => {
                    ui.text(format!("Unhandled log entry: {:?}", entry));
                }
            }
        }
    }
}
