use hecs::Entity;
use nat20_rs::engine::game_state::GameState;

use crate::{
    render::ui::{
        entities::CreatureRenderMode,
        utils::{
            ImguiRenderableMutWithContext, ImguiRenderableWithContext, render_uniform_buttons,
        },
    },
    windows::creature_debug::CreatureDebugWindow,
};

pub enum CreatureRightClickState {
    MainMenu,
    InspectCreature,
    DebugCreature(CreatureDebugWindow),
}

pub struct CreatureRightClickWindow {
    pub state: CreatureRightClickState,
    pub entity: Entity,
}

impl CreatureRightClickWindow {
    pub fn new(entity: Entity) -> Self {
        Self {
            state: CreatureRightClickState::MainMenu,
            entity,
        }
    }
}
impl ImguiRenderableMutWithContext<&mut GameState> for CreatureRightClickWindow {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, game_state: &mut GameState) {
        match &mut self.state {
            CreatureRightClickState::MainMenu => {
                if let Some(index) = render_uniform_buttons(ui, &["Inspect", "Debug"]) {
                    match index {
                        0 => {
                            self.state = CreatureRightClickState::InspectCreature;
                        }
                        1 => {
                            self.state = CreatureRightClickState::DebugCreature(
                                CreatureDebugWindow::new(self.entity),
                            );
                        }
                        _ => {}
                    }
                }
            }
            CreatureRightClickState::InspectCreature => {
                self.entity
                    .render_with_context(ui, (&game_state.world, CreatureRenderMode::Full));
            }
            CreatureRightClickState::DebugCreature(debug_gui) => {
                debug_gui.render_mut_with_context(ui, game_state);
            }
        }
    }
}
