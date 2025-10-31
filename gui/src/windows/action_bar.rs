use hecs::{Entity, World};
use imgui::{ChildFlags, MouseButton};
use nat20_rs::{
    components::{
        actions::{
            action::{ActionContext, ActionMap},
            targeting::{TargetInstance, TargetingError, TargetingKind},
        },
        id::{ActionId, Name},
        resource::{ResourceAmountMap, ResourceMap},
        speed::Speed,
    },
    engine::{
        event::{ActionData, ActionDecision, ActionError, ActionPrompt},
        game_state::GameState,
    },
    registry,
    systems::{
        self,
        actions::ActionUsabilityError,
        geometry::{RaycastHit, RaycastHitKind},
        movement::PathResult,
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
                render_button_disabled_conditionally, render_progress_bar,
            },
        },
        world::mesh::Wireframe,
    },
    state::gui_state::GuiState,
    windows::anchor::{AUTO_RESIZE, BOTTOM_CENTER, WindowManager},
};

const ACTION_BAR_MIN_SIZE: [f32; 2] = [500.0, 200.0];

#[derive(Debug, Clone)]
pub struct PotentialTarget {
    pub entity: Entity,
    pub validation_result: Result<(), ActionError>,
    pub path_to_target: Option<PathResult>,
}

impl PotentialTarget {
    pub fn new(
        game_state: &mut GameState,
        action: &ActionData,
        pathfind_if_out_of_range: bool,
    ) -> Self {
        let result = game_state.validate_action(action, false);

        let path_to_target = if pathfind_if_out_of_range
            && let Err(action_error) = &result
            && let ActionError::Usability(usability_error) = action_error
            && let ActionUsabilityError::TargetingError(targeting_error) = usability_error
            && let TargetingError::OutOfRange { target, .. }
            | TargetingError::NoLineOfSight { target } = targeting_error
        {
            let target_position = match target {
                TargetInstance::Entity(entity) => {
                    let (_, shape_pose) =
                        systems::geometry::get_shape(&game_state.world, *entity).unwrap();
                    shape_pose.translation.vector.into()
                }
                TargetInstance::Point(point) => *point,
            };

            println!(
                "[potential target] Cannot reach target {:?}, error: {:?}, attempting pathfinding...",
                target_position, action_error
            );

            let targeting_context = systems::actions::targeting_context(
                &game_state.world,
                action.actor,
                &action.action_id,
                &action.context,
            );

            if let Ok(path_result) = systems::movement::path_in_range_of_point(
                game_state,
                action.actor,
                target_position,
                targeting_context.range.max(),
                true,
                false,
                targeting_context.require_line_of_sight,
                true,
            ) {
                Some(path_result)
            } else {
                None
            }
        } else {
            None
        };

        PotentialTarget {
            entity: action.targets[0],
            validation_result: result,
            path_to_target: path_to_target,
        }
    }
}

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
        potential_target: Option<PotentialTarget>,
    },
}

pub struct ActionBarWindow {
    pub state: ActionBarState,
    pub entity: Entity,
}

impl ActionBarWindow {
    pub fn new(world: &World, entity: Entity) -> Self {
        Self {
            state: ActionBarState::Action {
                actions: systems::actions::available_actions(world, entity),
            },
            entity,
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
                || encounter
                    .next_pending_prompt()
                    .is_some_and(|prompt| matches!(prompt, ActionPrompt::Reactions { .. })))
        {
            true
        } else {
            false
        };

        let disabled_token = ui.begin_disabled(disabled);
        let style_token = ui.push_style_var(imgui::StyleVar::WindowMinSize(ACTION_BAR_MIN_SIZE));

        let window_manager_ptr =
            unsafe { &mut *(&mut gui_state.window_manager as *mut WindowManager) };

        window_manager_ptr.render_window(
            ui,
            format!(
                "Actions - {}",
                systems::helpers::get_component::<Name>(&game_state.world, self.entity).as_str()
            )
            .as_str(),
            &BOTTOM_CENTER,
            AUTO_RESIZE,
            &mut true,
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
    potential_target: &mut Option<PotentialTarget>,
) {
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

        if let Some(potential_target) = potential_target {
            let (preview_position, actor_has_to_move) = {
                if let Some(path) = &potential_target.path_to_target
                    && let Some(end) = path.taken_path.end()
                {
                    gui_state.path_cache.insert(action.actor, path.clone());
                    (*end, true)
                } else {
                    (
                        systems::geometry::get_foot_position(&game_state.world, action.actor)
                            .unwrap(),
                        false,
                    )
                }
            };

            if let Some((shape, shape_pose_at_preview)) =
                systems::geometry::get_shape_at_point(&game_state, action.actor, &preview_position)
                && let Some(mesh) = gui_state.mesh_cache.get(&format!("{:#?}", shape))
            {
                if actor_has_to_move {
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

                gui_state.line_renderer.add_line(
                    systems::geometry::get_shape(&game_state.world, potential_target.entity)
                        .map(|(_, shape_pose)| shape_pose.translation.vector.into())
                        .unwrap(),
                    shape_pose_at_preview.translation.vector.into(),
                    [1.0, 1.0, 1.0],
                );
            }

            if let Some(path) = &potential_target.path_to_target
                && path.reaches_goal()
                && ui.is_mouse_clicked(MouseButton::Left)
            {
                let result =
                    game_state.submit_movement(action.actor, *path.taken_path.end().unwrap());
            }
        } else {
            gui_state.path_cache.remove(&action.actor);
        }

        match targeting_context.kind {
            TargetingKind::SelfTarget => {
                if ui.is_mouse_clicked(MouseButton::Left) {
                    action.targets.clear();
                    action.targets.push(action.actor);
                    gui_state.cursor_ray_result.take();
                    submit_action = true;
                }
            }

            TargetingKind::Single => {
                if let Some(potential_target) = potential_target {
                    if ui.is_mouse_clicked(MouseButton::Left) {
                        action.targets.clear();
                        action.targets.push(potential_target.entity);
                        gui_state.cursor_ray_result.take();
                        submit_action = true;
                    }
                }
            }

            TargetingKind::Multiple { max_targets } => {
                let max_targets = max_targets as usize;
                if ui.is_mouse_clicked(MouseButton::Right) {
                    action.targets.pop();
                }
                if let Some(potential_target) = potential_target
                    && action.targets.len() < max_targets
                    && ui.is_mouse_clicked(MouseButton::Left)
                {
                    action.targets.push(potential_target.entity);
                    gui_state.cursor_ray_result.take();
                    if action.targets.len() == max_targets {
                        submit_action = true;
                    }
                }
            }

            TargetingKind::Area {
                shape,
                fixed_on_actor,
            } => todo!(),
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
        let result = game_state.submit_decision(ActionDecision::Action {
            action: action.clone(),
        });

        println!("Submitted action decision: {:#?}", result);
    }

    if ui.button("Cancel")
        || (action.targets.is_empty() && ui.is_mouse_clicked(MouseButton::Right))
        || submit_action
    {
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
    potential_target: &mut Option<PotentialTarget>,
    game_state: &mut GameState,
    action: &ActionData,
    closest: &RaycastHit,
) {
    let mut potential_action = action.clone();

    if let RaycastHitKind::Creature(entity) = &closest.kind {
        potential_action.targets.clear();
        potential_action.targets.push(*entity);

        if let Some(potential_target) = potential_target {
            if *entity != potential_target.entity {
                *potential_target = PotentialTarget::new(game_state, &potential_action, true);
            }
        } else {
            *potential_target = Some(PotentialTarget::new(game_state, &potential_action, true));
        }
    } else {
        *potential_target = None;
    }
}
