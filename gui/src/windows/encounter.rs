use std::collections::HashSet;

use hecs::{Entity, World};
use imgui::ChildFlags;
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::{TargetType, TargetingContext, TargetingKind},
        },
        id::{ActionId, EncounterId, Name},
        resource::ResourceMap,
        spells::spellbook::Spellbook,
    },
    engine::{
        encounter::{ActionDecision, ActionPrompt, Encounter, ParticipantsFilter},
        game_state::{ActionData, EventLog, GameEvent, GameState, ReactionData},
    },
    registry, systems,
};

use crate::{
    render::{
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
        context_options: Vec<ActionContext>,
        chosen_context: Option<ActionContext>,
        targets: Vec<Entity>,
    },
    Reaction {
        reactor: Entity,
        action: ActionData,
        options: Vec<ReactionData>,
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
                options: options.clone(),
                choice: None,
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
                options,
                choice,
            } => ActionDecision::Reaction {
                reactor,
                action,
                choice,
            },
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
                    };
                }
            }

            EncounterWindowState::EncounterRunning {
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
        let current_name =
            systems::helpers::get_component_clone::<Name>(world, current_entity).to_string();

        if let Some(table) = table_with_columns!(ui, "Initiative Order", "", "Participant",) {
            for (entity, initiative) in initiative_order {
                if let Ok(_) = world.query_one_mut::<&Name>(*entity) {
                    // Initiative column
                    ui.table_next_column();
                    ui.text(initiative.total.to_string());
                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.separator_with_text("Initiative");
                            initiative.render(ui);
                        });
                    }

                    if self.current_entity() == *entity {
                        ui.table_set_bg_color(imgui::TableBgTarget::all(), SELECTED_BUTTON_COLOR);
                    }

                    // Participant column
                    ui.table_next_column();
                    entity.render_with_context(ui, (world, CreatureRenderMode::Compact));
                }
            }

            table.end();
        }

        ui.separator();
        ui.text(format!("Round: {}", self.round()));

        let next_prompt = self.next_prompt();
        if next_prompt.is_none() {
            ui.text("No actions pending");
            return;
        }
        let next_prompt = next_prompt.unwrap();

        // TODO: If it's not a characters turn, the AI can make a decision here?
        // Also a bit odd maybe to have it in the render function?

        if decision_progress.is_none() {
            *decision_progress = Some(ActionDecisionProgress::from_prompt(next_prompt));
            println!(
                "Starting action decision progress for prompt: {:?}",
                next_prompt
            );
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
            decision_progress.render_mut_with_context(ui, (world, self));

            // Render placeholder action selection UI
            let token = Some(ui.begin_disabled(actions_disabled));
            Some(ActionDecisionProgress::from_prompt(&ActionPrompt::Action {
                actor: current_entity,
            }))
            .render_mut_with_context(ui, (world, self));

            token
        } else {
            // Render the actual action selection UI
            decision_progress.render_mut_with_context(ui, (world, self));
            None
        };

        ui.separator();

        if ui.button("End Turn") {
            decision_progress.take(); // Clear decision progress
            self.end_turn(world, current_entity);
        }

        if let Some(token) = disabled_token {
            token.end();
        }
    }
}

impl ImguiRenderableMutWithContext<(&mut World, &mut Encounter)>
    for Option<ActionDecisionProgress>
{
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, context: (&mut World, &mut Encounter)) {
        if self.is_none() {
            ui.text("No action decision in progress.");
            return;
        }

        let (world, encounter) = context;
        let current_entity = encounter.current_entity();
        match self.as_mut().unwrap() {
            ActionDecisionProgress::Action {
                actor: _,
                action_options,
                chosen_action,
                context_options,
                chosen_context,
                targets,
            } => {
                ui.separator_with_text("Resources");
                systems::helpers::get_component::<ResourceMap>(world, current_entity).render(ui);

                {
                    let spellbook =
                        systems::helpers::get_component::<Spellbook>(world, current_entity);
                    let spell_slots = spellbook.spell_slots();
                    if !spell_slots.is_empty() {
                        ui.separator_with_text("Spell Slots");
                        spell_slots.render(ui);
                    }
                }

                ui.separator_with_text("Actions");
                if action_options.is_empty() {
                    *action_options = systems::actions::available_actions(world, current_entity);
                }
                for (action_id, (contexts, resource_cost)) in action_options {
                    // Don't render reactions (actions that *only* cost a reaction)
                    if resource_cost.len() == 1
                        && resource_cost.contains_key(&registry::resources::REACTION)
                    {
                        continue;
                    }

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
                let mut cancel_action = false;
                if chosen_action.is_some() && chosen_context.is_some() {
                    render_window_at_cursor(ui, "Target Selection", true, || {
                        TextSegments::new(vec![
                            (
                                systems::helpers::get_component::<Name>(world, current_entity)
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
                            world,
                            current_entity,
                            chosen_action.as_ref().unwrap(),
                            chosen_context.as_ref().unwrap(),
                        );

                        targeting_context.render_with_context(
                            ui,
                            (world, encounter, targets, &mut confirm_targets),
                        );

                        ui.separator();
                        if ui.button("Confirm Targets") {
                            confirm_targets = true;
                        }
                        ui.separator();
                        if ui.button("Cancel Action") {
                            cancel_action = true;
                        }
                    });

                    if confirm_targets {
                        let decision = self.take().unwrap().finalize();
                        let result = encounter.process(world, decision);
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

            ActionDecisionProgress::Reaction {
                reactor,
                action,
                options,
                choice,
            } => {
                let mut confirm_reaction = false;
                render_window_at_cursor(ui, "Reaction", true, || {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component_clone::<Name>(world, action.actor)
                                .to_string(),
                            TextKind::Actor,
                        ),
                        ("used".to_string(), TextKind::Normal),
                        (action.action_id.to_string(), TextKind::Action),
                        ("on".to_string(), TextKind::Normal),
                        (
                            action
                                .targets
                                .iter()
                                .map(|t| {
                                    systems::helpers::get_component_clone::<Name>(world, *t)
                                        .to_string()
                                })
                                .collect::<Vec<_>>()
                                .join(", "),
                            TextKind::Target,
                        ),
                    ])
                    .render(ui);

                    ui.text("Choose how to react");

                    ui.separator_with_text(
                        systems::helpers::get_component_clone::<Name>(world, *reactor).as_str(),
                    );
                    for option in options {
                        if render_button_selectable(
                            ui,
                            format!(
                                "{}: {:?}\nCost: {:?}",
                                option.reaction_id, option.context, option.resource_cost
                            ),
                            [0., 0.],
                            choice.as_ref() == Some(option),
                        ) {
                            if choice.as_ref() == Some(option) {
                                *choice = None;
                            } else {
                                *choice = Some(option.clone());
                            }
                        }
                    }

                    ui.separator();

                    if ui.button("Confirm Reaction") {
                        confirm_reaction = true;
                    }
                });

                if confirm_reaction {
                    let decision = self.take().unwrap().finalize();
                    let result = encounter.process(world, decision).unwrap();
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

impl ImguiRenderableWithContext<(&mut World, &Encounter, &mut Vec<Entity>, &mut bool)>
    for TargetingContext
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        context: (&mut World, &Encounter, &mut Vec<Entity>, &mut bool),
    ) {
        let (world, encounter, targets, confirm_targets) = context;

        for target_type in &self.valid_target_types {
            match target_type {
                TargetType::Entity { .. } => {
                    let filter = ParticipantsFilter::from(target_type.clone());
                    self.kind.render_with_context(
                        ui,
                        (world, encounter, targets, confirm_targets, filter),
                    );
                }
            }
        }
    }
}

impl
    ImguiRenderableWithContext<(
        &mut World,
        &Encounter,
        &mut Vec<Entity>,
        &mut bool,
        ParticipantsFilter,
    )> for TargetingKind
{
    fn render_with_context(
        &self,
        ui: &imgui::Ui,
        context: (
            &mut World,
            &Encounter,
            &mut Vec<Entity>,
            &mut bool,
            ParticipantsFilter,
        ),
    ) {
        let (world, encounter, targets, confirm_targets, filter) = context;

        match &self {
            TargetingKind::Single => {
                ui.text("Select a single target:");
                for entity in encounter.participants(&world, filter) {
                    if let Ok(name) = world.query_one_mut::<&Name>(entity) {
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
                for entity in encounter.participants(&world, filter) {
                    if let Ok(name) = world.query_one_mut::<&Name>(entity) {
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
                    if let Ok(name) = world.query_one_mut::<&Name>(*target) {
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
                targets.push(encounter.current_entity());
                *confirm_targets = true;
            }

            _ => {
                ui.text(format!("Targeting kind {:?} is not implemented yet.", self));
            }
        }
    }
}
