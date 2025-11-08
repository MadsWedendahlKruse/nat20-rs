use hecs::{Entity, World};
use imgui::{ChildFlags, MouseButton};
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::{AreaShape, TargetInstance, TargetingKind},
        },
        id::{ActionId, Name},
        resource::{ResourceAmountMap, ResourceMap},
        speed::Speed,
    },
    engine::{
        event::{ActionData, ActionDecision, ActionDecisionKind, ActionPromptKind},
        game_state::GameState,
    },
    registry,
    systems::{
        self,
        actions::TargetPathFindingResult,
        geometry::{RaycastHit, RaycastHitKind},
    },
};
use uom::si::length::meter;

use crate::{
    render::{
        common::utils::RenderableMutWithContext,
        ui::{
            components::{LOW_HEALTH_BG_COLOR, LOW_HEALTH_COLOR, SPEED_COLOR, SPEED_COLOR_BG},
            utils::{
                ImguiRenderable, ImguiRenderableWithContext, ProgressBarColor,
                render_button_disabled_conditionally, render_capacity_meter, render_progress_bar,
            },
        },
        world::mesh::Wireframe,
    },
    state::gui_state::GuiState,
    windows::{
        anchor::{AUTO_RESIZE, BOTTOM_CENTER, WindowManager},
        reactions::ReactionsWindow,
    },
};

const ACTION_BAR_MIN_SIZE: [f32; 2] = [500.0, 200.0];

#[derive(Debug, Clone)]
pub enum ActionBarState {
    Action {
        actions: ActionMap,
    },
    Context {
        action: ActionId,
        contexts_and_costs: Vec<(ActionContext, ResourceAmountMap)>,
    },
    Targets {
        action: ActionData,
        potential_target: Option<(TargetInstance, TargetPathFindingResult)>,
    },
}

pub struct ActionBarWindow {
    pub state: ActionBarState,
    pub entity: Entity,
    // TOOD: In a way these seem tightly connected, but could also live somewhere else?
    pub reaction_window: Option<ReactionsWindow>,
}

impl ActionBarWindow {
    pub fn new(world: &World, entity: Entity) -> Self {
        Self {
            state: ActionBarState::Action {
                actions: systems::actions::available_actions(world, entity),
            },
            entity,
            reaction_window: None,
        }
    }
}

impl RenderableMutWithContext<&mut GameState> for ActionBarWindow {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        game_state: &mut GameState,
    ) {
        let disabled = if let Some(encounter_id) = &game_state.in_combat.get(&self.entity)
            && let Some(encounter) = game_state.encounters.get(encounter_id)
            && (encounter.current_entity() != self.entity
                || encounter.next_pending_prompt().is_some_and(|prompt| {
                    matches!(prompt.kind, ActionPromptKind::Reactions { .. })
                })) {
            true
        } else {
            false
        };

        let disabled_token = ui.begin_disabled(disabled);
        let style_token = ui.push_style_var(imgui::StyleVar::WindowMinSize(ACTION_BAR_MIN_SIZE));

        let window_manager_ptr =
            unsafe { &mut *(&mut gui_state.window_manager as *mut WindowManager) };

        let mut opened = true;

        window_manager_ptr.render_window(
            ui,
            format!(
                "Actions - {}",
                systems::helpers::get_component::<Name>(&game_state.world, self.entity).as_str()
            )
            .as_str(),
            &BOTTOM_CENTER,
            AUTO_RESIZE,
            &mut opened,
            || {
                let mut new_state = None;

                match &mut self.state {
                    ActionBarState::Action { actions } => {
                        render_actions(ui, game_state, self.entity, &mut new_state, actions);
                        ui.same_line();
                        render_resources(ui, game_state, self.entity);
                    }

                    ActionBarState::Context {
                        action,
                        contexts_and_costs,
                    } => {
                        ui.text(format!("Select context for action: {}", action));

                        for (i, (context, cost)) in contexts_and_costs.iter().enumerate() {
                            if i > 0 {
                                ui.same_line();
                            }

                            if ui.button(format!("{:#?}\n{:#?}", context, cost)) {
                                new_state = Some(ActionBarState::Targets {
                                    action: ActionData {
                                        actor: self.entity,
                                        action_id: action.clone(),
                                        context: context.clone(),
                                        resource_cost: cost.clone(),
                                        targets: Vec::new(),
                                    },
                                    potential_target: None,
                                });
                            }
                        }

                        ui.separator();

                        if ui.button("Cancel") {
                            new_state = Some(ActionBarState::Action {
                                actions: systems::actions::available_actions(
                                    &game_state.world,
                                    self.entity,
                                ),
                            });
                        }
                    }

                    ActionBarState::Targets {
                        action,
                        potential_target,
                    } => {
                        render_target_selection(
                            ui,
                            gui_state,
                            game_state,
                            &mut new_state,
                            action,
                            potential_target,
                        );
                    }
                }

                if let Some(state) = new_state {
                    self.state = state;
                }
            },
        );

        disabled_token.end();
        style_token.end();

        if let Some(reaction_window) = &mut self.reaction_window {
            reaction_window.render_mut_with_context(ui, gui_state, game_state);
        }

        if !opened {
            gui_state.selected_entity.take();
        }
    }
}

fn render_actions(
    ui: &imgui::Ui,
    game_state: &mut GameState,
    entity: Entity,
    new_state: &mut Option<ActionBarState>,
    actions: &mut ActionMap,
) {
    ui.child_window("Actions")
        .child_flags(
            ChildFlags::ALWAYS_AUTO_RESIZE | ChildFlags::AUTO_RESIZE_X | ChildFlags::AUTO_RESIZE_Y,
        )
        .build(|| {
            ui.separator_with_text("Actions");

            for (action_id, contexts_and_costs) in actions {
                // Don't render reactions
                if contexts_and_costs
                    .iter()
                    .all(|(_, cost)| cost.contains_key(&registry::resources::REACTION_ID))
                {
                    continue;
                }

                if ui.button(&action_id.to_string()) {
                    if contexts_and_costs.len() == 1 {
                        *new_state = Some(ActionBarState::Targets {
                            action: ActionData {
                                actor: entity,
                                action_id: action_id.clone(),
                                context: contexts_and_costs[0].0.clone(),
                                resource_cost: contexts_and_costs[0].1.clone(),
                                targets: Vec::new(),
                            },
                            potential_target: None,
                        });
                    } else {
                        *new_state = Some(ActionBarState::Context {
                            action: action_id.clone(),
                            contexts_and_costs: contexts_and_costs.clone(),
                        });
                    }

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            (action_id, contexts_and_costs)
                                .render_with_context(ui, (&game_state.world, entity));
                        });
                    }
                }
            }

            ui.separator();

            if game_state.in_combat.contains_key(&entity) {
                if ui.button("End Turn") {
                    game_state.end_turn(entity);
                }
            }
        });
}

fn render_resources(ui: &imgui::Ui, game_state: &mut GameState, entity: Entity) {
    ui.child_window("Resources")
        .child_flags(
            ChildFlags::ALWAYS_AUTO_RESIZE | ChildFlags::AUTO_RESIZE_X | ChildFlags::AUTO_RESIZE_Y,
        )
        .build(|| {
            ui.separator_with_text("Resources");
            systems::helpers::get_component::<ResourceMap>(&game_state.world, entity).render(ui);

            ui.separator_with_text("Speed");
            let speed = systems::helpers::get_component::<Speed>(&game_state.world, entity);

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
}

fn render_target_selection(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    game_state: &mut GameState,
    new_state: &mut Option<ActionBarState>,
    action: &mut ActionData,
    potential_target: &mut Option<(TargetInstance, TargetPathFindingResult)>,
) {
    ui.tooltip_text(action.action_id.as_str());

    let targeting_context = systems::actions::targeting_context(
        &game_state.world,
        action.actor,
        &action.action_id,
        &action.context,
    );

    render_range_preview(gui_state, game_state, action, &targeting_context);

    let mut submit_action = false;

    if let Some(raycast) = &gui_state.cursor_ray_result
        && let Some(closest) = raycast.closest()
    {
        update_potential_target(potential_target, game_state, action, closest);

        let potential_target_instance = if let Some((target, path_result)) = potential_target {
            let (preview_position, path_to_target) = match path_result {
                TargetPathFindingResult::AlreadyInRange => (
                    systems::geometry::get_eye_position(&game_state.world, action.actor).unwrap(),
                    None,
                ),

                TargetPathFindingResult::PathFound(path) => {
                    if let Some(end) = path.taken_path.end() {
                        (*end, Some(path.clone()))
                    } else {
                        (
                            systems::geometry::get_eye_position(&game_state.world, action.actor)
                                .unwrap(),
                            None,
                        )
                    }
                }
            };

            if let Some((shape, shape_pose_at_preview)) = systems::geometry::get_shape_at_point(
                &game_state.world,
                &game_state.geometry,
                action.actor,
                &preview_position,
            ) && let Some(mesh) = gui_state.mesh_cache.get(&format!("{:#?}", shape))
            {
                if let Some(path_to_target) = &path_to_target {
                    gui_state
                        .path_cache
                        .insert(action.actor, path_to_target.clone());
                    mesh.draw(
                        gui_state.ig_renderer.gl_context(),
                        &gui_state.program,
                        &shape_pose_at_preview.to_homogeneous(),
                        [1.0, 1.0, 1.0, 0.75],
                        &Wireframe::Only {
                            color: [1.0, 1.0, 1.0, 0.75],
                            width: 2.0,
                        },
                    );
                }

                let line_end = match target {
                    TargetInstance::Entity(entity) => {
                        systems::geometry::get_shape(&game_state.world, *entity)
                            .map(|(_, shape_pose)| shape_pose.translation.vector.into())
                            .unwrap()
                    }
                    TargetInstance::Point(point) => *point,
                };
                gui_state.line_renderer.add_line(
                    preview_position.into(),
                    line_end.into(),
                    [1.0, 1.0, 1.0],
                );
            }

            if let Some(path_to_target) = &path_to_target
                && path_to_target.reaches_goal()
                && ui.is_mouse_clicked(MouseButton::Left)
            {
                let result = game_state
                    .submit_movement(action.actor, *path_to_target.taken_path.end().unwrap());
            }

            Some(target)
        } else {
            gui_state.path_cache.remove(&action.actor);
            None
        };

        match targeting_context.kind {
            TargetingKind::SelfTarget => {
                if ui.is_mouse_clicked(MouseButton::Left) {
                    action.targets.clear();
                    action.targets.push(TargetInstance::Entity(action.actor));
                    gui_state.cursor_ray_result.take();
                    submit_action = true;
                }
            }

            TargetingKind::Single => {
                if let Some(potential_target) = potential_target_instance {
                    if ui.is_mouse_clicked(MouseButton::Left) {
                        action.targets.clear();
                        action.targets.push(potential_target.clone());
                        gui_state.cursor_ray_result.take();
                        submit_action = true;
                    }
                }
            }

            TargetingKind::Multiple { max_targets } => {
                let max_targets = max_targets as usize;
                if ui.is_mouse_clicked(MouseButton::Right) {
                    gui_state.cursor_ray_result.take();
                    action.targets.pop();
                }
                if let Some(potential_target) = potential_target_instance
                    && action.targets.len() < max_targets
                    && ui.is_mouse_clicked(MouseButton::Left)
                {
                    action.targets.push(potential_target.clone());
                    gui_state.cursor_ray_result.take();
                    if action.targets.len() == max_targets {
                        submit_action = true;
                    }
                }

                ui.tooltip(|| {
                    ui.separator();
                    render_capacity_meter(
                        ui,
                        action.action_id.as_str(),
                        action.targets.len(),
                        max_targets,
                    );
                });
            }

            TargetingKind::Area {
                shape,
                fixed_on_actor,
            } => {
                if let Some(potential_target) = potential_target_instance {
                    // 1. Render the area shape at the potential target location
                    match &shape {
                        AreaShape::Sphere { radius } => {
                            let center = match &potential_target {
                                TargetInstance::Entity(entity) => {
                                    systems::geometry::get_foot_position(&game_state.world, *entity)
                                        .unwrap()
                                }
                                TargetInstance::Point(point) => *point,
                            };
                            gui_state.line_renderer.add_circle(
                                [center.x, center.y, center.z],
                                radius.get::<meter>(),
                                [1.0, 1.0, 1.0],
                            );
                        }
                        _ => { /* TODO other shapes */ }
                    }
                    // 2. Highlight entities within the area
                    // 3. On left click, select all entities within the area as targets
                    if ui.is_mouse_clicked(MouseButton::Left) {
                        action.targets.clear();
                        action.targets.push(potential_target.clone());
                        gui_state.cursor_ray_result.take();
                        submit_action = true;
                    }
                }
            }
        }
    }

    if render_button_disabled_conditionally(
        ui,
        "Confirm Targets",
        [0.0, 0.0],
        action.targets.len() == 0,
        "Must select at least one target",
    ) {
        submit_action = true;
    }

    if submit_action {
        let response_to = if let Some(encounter_id) = &game_state.in_combat.get(&action.actor)
            && let Some(encounter) = game_state.encounters.get(encounter_id)
            && let Some(prompt) = encounter.next_pending_prompt()
            && prompt.actors().contains(&action.actor)
        {
            println!(
                "[ActionBar] Submitting action in encounter {:?} in response to prompt: {:#?}",
                encounter_id, prompt
            );
            Some(prompt.id)
        } else {
            None
        };

        let action_kind = ActionDecisionKind::Action {
            action: action.clone(),
        };

        let result = if let Some(response_to) = response_to {
            game_state.submit_decision(ActionDecision {
                response_to,
                kind: action_kind,
            })
        } else {
            game_state.submit_decision(ActionDecision::without_response_to(action_kind))
        };

        println!("Submitted action decision: {:#?}", result);
    }

    // TODO: gui_state util function to handle checking for Some and taking?
    let right_click_cancel = if gui_state.cursor_ray_result.is_some()
        && ui.is_mouse_clicked(MouseButton::Right)
        && action.targets.is_empty()
    {
        gui_state.cursor_ray_result.take();
        true
    } else {
        false
    };

    if ui.button("Cancel") || right_click_cancel || submit_action {
        *new_state = Some(ActionBarState::Action {
            actions: systems::actions::available_actions(&game_state.world, action.actor),
        });
    }
}

fn render_range_preview(
    gui_state: &mut GuiState,
    game_state: &mut GameState,
    action: &mut ActionData,
    targeting_context: &nat20_rs::components::actions::targeting::TargetingContext,
) {
    let normal_range = targeting_context.range.normal().get::<meter>();
    let max_range = targeting_context.range.max().get::<meter>();
    let actor_position = systems::geometry::get_foot_position(&game_state.world, action.actor)
        .map(|point| [point.x, point.y, point.z])
        .unwrap();
    gui_state
        .line_renderer
        .add_circle(actor_position, normal_range, [1.0, 1.0, 1.0]);
    if normal_range < max_range {
        gui_state
            .line_renderer
            .add_circle(actor_position, max_range, [0.5, 0.5, 0.5]);
    }
}

fn update_potential_target(
    potential_target: &mut Option<(TargetInstance, TargetPathFindingResult)>,
    game_state: &mut GameState,
    action: &ActionData,
    closest: &RaycastHit,
) {
    let closest_target = match &closest.kind {
        RaycastHitKind::Creature(entity) => TargetInstance::Entity(*entity),
        RaycastHitKind::World => TargetInstance::Point(closest.poi),
        _ => return,
    };

    let mut potential_action = action.clone();
    potential_action.targets.clear();
    potential_action.targets.push(closest_target.clone());

    let is_new_target = if let Some((target, _)) = potential_target {
        target != &closest_target
    } else {
        true
    };

    if is_new_target {
        println!(
            "[update_potential_target] Finding path to new target {:?}",
            closest_target
        );
        match systems::actions::path_to_target(game_state, &potential_action, true) {
            Ok(result) => {
                println!(
                    "[update_potential_target] Found path to target {:?}: {:?}",
                    closest_target, result
                );
                *potential_target = Some((closest_target, result));
            }
            Err(err) => {
                println!(
                    "[update_potential_target] Error finding path to target {:?}: {:?}",
                    closest_target, err
                );
                *potential_target = None;
            }
        }
    }
}
