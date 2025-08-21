use hecs::Entity;
use nat20_rs::{
    components::{
        ability::Ability,
        d20_check::D20CheckDC,
        modifier::{ModifierSet, ModifierSource},
        skill::Skill,
    },
    engine::game_state::{GameEvent, GameState},
    systems,
};
use strum::IntoEnumIterator;

use crate::render::utils::{ImguiRenderableMutWithContext, render_uniform_buttons};

pub enum CheckKind {
    SavingThrow,
    SkillCheck,
}

pub enum CreatureDebugState {
    MainMenu,
    Check { kind: CheckKind, dc_value: i32 },
}

pub struct CreatureDebugWindow {
    pub state: CreatureDebugState,
    pub creature: Entity,
}

impl CreatureDebugWindow {
    pub fn new(creature: Entity) -> Self {
        Self {
            state: CreatureDebugState::MainMenu,
            creature,
        }
    }
}

impl ImguiRenderableMutWithContext<&mut GameState> for CreatureDebugWindow {
    fn render_mut_with_context(&mut self, ui: &imgui::Ui, game_state: &mut GameState) {
        ui.popup("Debug", || match &mut self.state {
            CreatureDebugState::MainMenu => {
                if let Some(index) = render_uniform_buttons(
                    ui,
                    ["Heal Full", "Saving Throw", "Skill Check"],
                    [20.0, 5.0],
                ) {
                    match index {
                        0 => {
                            systems::health::heal_full(&mut game_state.world, self.creature);
                            ui.close_current_popup();
                        }
                        1 => {
                            self.state = CreatureDebugState::Check {
                                kind: CheckKind::SavingThrow,
                                dc_value: 10,
                            };
                        }
                        2 => {
                            self.state = CreatureDebugState::Check {
                                kind: CheckKind::SkillCheck,
                                dc_value: 10,
                            };
                        }
                        _ => unreachable!(),
                    }
                }
            }

            CreatureDebugState::Check { kind, dc_value } => match kind {
                CheckKind::SavingThrow => {
                    ui.separator_with_text("Saving Throw");

                    let width_token = ui.push_item_width(50.0);
                    ui.input_int("DC", dc_value)
                        .auto_select_all(true)
                        .enter_returns_true(true)
                        .build();
                    width_token.end();
                    ui.separator();
                    let choice = render_uniform_buttons(
                        ui,
                        Ability::iter().map(|ability| ability.to_string()),
                        [20.0, 5.0],
                    );

                    if let Some(index) = choice {
                        let ability = Ability::iter().nth(index).expect("Invalid ability index");
                        let dc = D20CheckDC {
                            dc: ModifierSet::from_iter([(
                                ModifierSource::Custom("Saving Throw DC".to_string()),
                                *dc_value,
                            )]),
                            key: ability,
                        };
                        game_state.log_event(GameEvent::SavingThrow(
                            self.creature,
                            systems::d20_check::saving_throw_dc(
                                &game_state.world,
                                self.creature,
                                &dc,
                            ),
                            dc,
                        ));
                        ui.close_current_popup();
                    }
                }

                CheckKind::SkillCheck => {
                    ui.separator_with_text("Skill Check");

                    let width_token = ui.push_item_width(50.0);
                    ui.input_int("DC", dc_value)
                        .auto_select_all(true)
                        .enter_returns_true(true)
                        .build();
                    width_token.end();
                    ui.separator();
                    let choice = render_uniform_buttons(
                        ui,
                        Skill::iter().map(|skill| skill.to_string()),
                        [20.0, 5.0],
                    );

                    if let Some(index) = choice {
                        let skill = Skill::iter().nth(index).expect("Invalid skill index");
                        let dc = D20CheckDC {
                            dc: ModifierSet::from_iter([(
                                ModifierSource::Custom("Saving Throw DC".to_string()),
                                *dc_value,
                            )]),
                            key: skill,
                        };
                        game_state.log_event(GameEvent::SkillCheck(
                            self.creature,
                            systems::d20_check::skill_check_dc(
                                &game_state.world,
                                self.creature,
                                &dc,
                            ),
                            dc,
                        ));
                        ui.close_current_popup();
                    }
                }
            },
        });
    }
}
