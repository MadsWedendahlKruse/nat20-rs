use hecs::Entity;
use nat20_rs::{
    components::actions::targeting::LineOfSightMode,
    engine::game_state::GameState,
    systems::{
        self,
        geometry::{LineOfSightResult, RaycastMode},
    },
};
use parry3d::query::Ray;

use crate::{
    render::common::utils::RenderableMutWithContext,
    state::{self, gui_state::GuiState},
    windows::anchor::{self, AUTO_RESIZE},
};

// TODO: No idea what to call this
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LineOfSightKind {
    Entity(Option<Entity>),
    Point([f32; 3]),
}

pub struct LineOfSightDebugWindow {
    pub from: LineOfSightKind,
    pub to: LineOfSightKind,
    pub mode: LineOfSightMode,
    pub result: Option<LineOfSightResult>,
    pub show_raycast: bool,
}

impl LineOfSightDebugWindow {
    pub fn new() -> Self {
        Self {
            from: LineOfSightKind::Point([0.0, 0.0, 0.0]),
            to: LineOfSightKind::Point([0.0, 0.0, 0.0]),
            mode: LineOfSightMode::Ray,
            result: None,
            show_raycast: true,
        }
    }
}

impl RenderableMutWithContext<&mut GameState> for LineOfSightDebugWindow {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        gui_state: &mut GuiState,
        game_state: &mut GameState,
    ) {
        let mut los_debug_open = *gui_state
            .settings
            .get::<bool>(state::parameters::RENDER_LINE_OF_SIGHT_DEBUG);

        if !los_debug_open {
            return;
        }

        gui_state.window_manager.render_window(
            ui,
            "Line of Sight Debug",
            &anchor::CENTER_RIGHT,
            AUTO_RESIZE,
            &mut los_debug_open,
            || {
                ui.separator_with_text("From");

                let width_token = ui.push_item_width(200.0);

                match &mut self.from {
                    LineOfSightKind::Entity(entity_option) => {
                        // TODO
                    }
                    LineOfSightKind::Point(point) => {
                        ui.input_float3("Point##From", point).build();
                    }
                }

                ui.separator_with_text("To");

                match &mut self.to {
                    LineOfSightKind::Entity(entity_option) => {
                        // TODO
                    }
                    LineOfSightKind::Point(point) => {
                        ui.input_float3("Point##To", point).build();
                    }
                }

                width_token.end();

                if ui.button("Compute Line of Sight") {
                    match (&self.from, &self.to) {
                        (
                            LineOfSightKind::Entity(from_entity_option),
                            LineOfSightKind::Entity(to_entity_option),
                        ) => {
                            if let (Some(from_entity), Some(to_entity)) =
                                (from_entity_option, to_entity_option)
                            {
                                self.result = Some(systems::geometry::line_of_sight_entity_entity(
                                    &game_state.world,
                                    &game_state.geometry,
                                    *from_entity,
                                    *to_entity,
                                    &self.mode,
                                ));
                            }
                        }

                        (LineOfSightKind::Entity(entity), LineOfSightKind::Point(_)) => todo!(),

                        (LineOfSightKind::Point(_), LineOfSightKind::Entity(entity)) => todo!(),

                        (LineOfSightKind::Point(_), LineOfSightKind::Point(_)) => {
                            if let (
                                LineOfSightKind::Point(from_point),
                                LineOfSightKind::Point(to_point),
                            ) = (self.from.clone(), self.to.clone())
                            {
                                self.result = Some(systems::geometry::line_of_sight_point_point(
                                    &game_state.world,
                                    &game_state.geometry,
                                    from_point.into(),
                                    to_point.into(),
                                    &self.mode,
                                    &systems::geometry::RaycastFilter::All,
                                ));
                            }
                        }
                    }
                }

                ui.checkbox("Show Raycast", &mut self.show_raycast);

                if let Some(result) = &self.result {
                    ui.text(format!("{:#?}", result));

                    if self.show_raycast
                        && let Some(raycast_result) = &result.raycast_result
                    {
                        match raycast_result.mode {
                            RaycastMode::Ray(ray) => {
                                let end_point = if let Some(closest) = raycast_result.closest() {
                                    ray.point_at(closest.toi)
                                } else {
                                    ray.point_at(f32::MAX)
                                };
                                gui_state.line_renderer.add_line(
                                    ray.origin.into(),
                                    end_point.into(),
                                    [1.0, 1.0, 1.0],
                                );
                            }
                            RaycastMode::Parabola {
                                start,
                                initial_velocity,
                                time_step,
                                ..
                            } => {
                                let toi = if let Some(closest) = raycast_result.closest() {
                                    closest.toi
                                } else {
                                    5.0 // Arbitrary fallback
                                };
                                gui_state.line_renderer.add_parabola(
                                    start.into(),
                                    initial_velocity.into(),
                                    ((toi / time_step).ceil() as usize).max(2),
                                    [1.0, 1.0, 1.0],
                                );
                            }
                        }
                    }
                }
            },
        );
    }
}
