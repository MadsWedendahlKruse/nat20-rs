use hecs::Entity;
use nat20_rs::{
    components::{
        ai::PlayerControlledTag,
        d20::D20CheckDC,
        modifier::{ModifierSet, ModifierSource},
        resource::RechargeRule,
        saving_throw::SavingThrowKind,
        skill::Skill,
    },
    engine::game_state::GameState,
    systems::{self, d20::D20CheckDCKind},
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
    PassTime,
    TogglePlayerControl,
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
                    [
                        "Despawn",
                        "Heal Full",
                        "Pass Time (Rest etc.)",
                        "Toggle Player Control",
                        "Saving Throw",
                        "Skill Check",
                    ],
                    [20.0, 5.0],
                ) {
                    match index {
                        0 => {
                            game_state.world.despawn(self.creature).ok();
                            ui.close_current_popup();
                        }
                        1 => {
                            systems::health::heal_full(&mut game_state.world, self.creature);
                            ui.close_current_popup();
                        }
                        2 => {
                            self.state = CreatureDebugState::PassTime;
                        }
                        3 => {
                            self.state = CreatureDebugState::TogglePlayerControl;
                        }
                        4 => {
                            self.state = CreatureDebugState::Check {
                                kind: CheckKind::SavingThrow,
                                dc_value: 10,
                            };
                        }
                        5 => {
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
                        SavingThrowKind::iter().map(|ability| ability.to_string()),
                        [20.0, 5.0],
                    );

                    if let Some(index) = choice {
                        let kind = SavingThrowKind::iter()
                            .nth(index)
                            .expect("Invalid ability index");
                        let dc = D20CheckDCKind::SavingThrow(D20CheckDC {
                            dc: ModifierSet::from_iter([(
                                ModifierSource::Custom("Saving Throw DC".to_string()),
                                *dc_value,
                            )]),
                            key: kind,
                        });
                        let event = systems::d20::check(game_state, self.creature, &dc);
                        game_state.process_event(event);
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
                        let dc = D20CheckDCKind::Skill(D20CheckDC {
                            dc: ModifierSet::from_iter([(
                                ModifierSource::Custom("Skill Check DC".to_string()),
                                *dc_value,
                            )]),
                            key: skill,
                        });
                        let event = systems::d20::check(game_state, self.creature, &dc);
                        game_state.process_event(event);
                        ui.close_current_popup();
                    }
                }
            },

            CreatureDebugState::PassTime => {
                if let Some(index) =
                    render_uniform_buttons(ui, ["New Turn", "Short Rest", "Long Rest"], [20.0, 5.0])
                {
                    let passed_time = match index {
                        0 => RechargeRule::Turn,
                        1 => RechargeRule::ShortRest,
                        2 => RechargeRule::LongRest,
                        _ => unreachable!(),
                    };

                    systems::time::pass_time(&mut game_state.world, self.creature, &passed_time);
                    ui.close_current_popup();
                }
            }

            CreatureDebugState::TogglePlayerControl => {
                if let Some(index) = render_uniform_buttons(
                    ui,
                    ["Set Player Controlled", "Set AI Controlled"],
                    [20.0, 5.0],
                ) {
                    match index {
                        0 => {
                            game_state
                                .world
                                .insert_one(self.creature, PlayerControlledTag)
                                .ok();
                        }
                        1 => {
                            game_state
                                .world
                                .remove_one::<PlayerControlledTag>(self.creature)
                                .ok();
                        }
                        _ => unreachable!(),
                    }
                    ui.close_current_popup();
                }
            }
        });
    }
}
