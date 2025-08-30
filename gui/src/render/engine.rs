use hecs::World;
use imgui::TreeNodeFlags;
use nat20_rs::{
    components::{health::life_state::LifeState, id::Name},
    engine::game_state::{EventLog, GameEvent},
    systems,
};

use crate::render::{
    text::{TextKind, TextSegments},
    utils::{ImguiRenderable, ImguiRenderableWithContext},
};

impl ImguiRenderableWithContext<&World> for EventLog {
    fn render_with_context(&self, ui: &imgui::Ui, world: &World) {
        for entry in self {
            match entry {
                GameEvent::EncounterStarted(encounter_id) => {
                    ui.separator_with_text(&format!("Encounter {}", encounter_id));
                }

                GameEvent::EncounterEnded(encounter_id, combat_log) => {
                    if ui.collapsing_header(
                        format!("Combat log##{}", encounter_id),
                        TreeNodeFlags::FRAMED,
                    ) {
                        combat_log.render_with_context(ui, world);
                    }
                    ui.separator();
                }

                GameEvent::ActionPerformed { action, results } => {
                    TextSegments::new(vec![
                        (
                            &systems::helpers::get_component::<Name>(world, action.actor)
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
                                systems::helpers::get_component::<Name>(world, action.targets[0])
                                    .as_str(),
                                TextKind::Target,
                            ),
                        ])
                        .render(ui);
                    }

                    for result in results {
                        result.render_with_context(ui, 0);
                    }
                }

                GameEvent::ReactionTriggered { reactor, action } => {
                    let mut segments = vec![
                        (
                            systems::helpers::get_component_clone::<Name>(world, action.actor)
                                .to_string(),
                            TextKind::Actor,
                        ),
                        ("used".to_string(), TextKind::Normal),
                        (action.action_id.to_string(), TextKind::Action),
                    ];
                    for (i, action_target) in action.targets.iter().enumerate() {
                        if i == 0 {
                            segments.push(("on".to_string(), TextKind::Normal));
                        } else {
                            segments.push((", ".to_string(), TextKind::Normal));
                        }
                        segments.push((
                            systems::helpers::get_component_clone::<Name>(world, *action_target)
                                .to_string(),
                            TextKind::Target,
                        ));
                    }
                    TextSegments::new(segments).render(ui);

                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *reactor).to_string(),
                            TextKind::Actor,
                        ),
                        ("is reacting to".to_string(), TextKind::Normal),
                        (
                            format!(
                                "{}'s",
                                systems::helpers::get_component::<Name>(world, action.actor)
                                    .as_str(),
                            ),
                            TextKind::Actor,
                        ),
                        (action.action_id.to_string(), TextKind::Action),
                    ])
                    .render(ui);
                }

                GameEvent::ActionCancelled {
                    reactor,
                    reaction,
                    action,
                } => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *reactor).to_string(),
                            TextKind::Actor,
                        ),
                        ("cancelled".to_string(), TextKind::Normal),
                        (
                            format!(
                                "{}'s",
                                systems::helpers::get_component::<Name>(world, action.actor)
                                    .as_str(),
                            ),
                            TextKind::Actor,
                        ),
                        (action.action_id.to_string(), TextKind::Action),
                        ("using".to_string(), TextKind::Normal),
                        (reaction.reaction_id.to_string(), TextKind::Action),
                    ])
                    .render(ui);
                }

                GameEvent::NoReactionTaken { reactor, action } => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *reactor).to_string(),
                            TextKind::Actor,
                        ),
                        ("did not react to".to_string(), TextKind::Normal),
                        (
                            format!(
                                "{}'s",
                                systems::helpers::get_component::<Name>(world, action.actor)
                                    .as_str(),
                            ),
                            TextKind::Actor,
                        ),
                        (action.action_id.to_string(), TextKind::Action),
                    ])
                    .render(ui);
                }

                GameEvent::NewRound(entity_id, round) => {
                    ui.separator_with_text(format!("Round {}", round));
                }
                GameEvent::SavingThrow(entity, result, dc) => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *entity).to_string(),
                            TextKind::Actor,
                        ),
                        (
                            // TODO: a vs an
                            if result.success {
                                "succeeded a".to_string()
                            } else {
                                "failed a".to_string()
                            },
                            TextKind::Normal,
                        ),
                        (dc.key.to_string(), TextKind::Ability),
                        ("saving throw".to_string(), TextKind::Normal),
                    ])
                    .render(ui);

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("DC:");
                            ui.same_line();
                            dc.render(ui);
                            ui.text("");
                            ui.text("Saving Throw:");
                            ui.same_line();
                            result.render(ui);
                        });
                    }
                }

                GameEvent::SkillCheck(entity, result, dc) => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *entity).to_string(),
                            TextKind::Actor,
                        ),
                        (
                            if result.success {
                                "succeeded a".to_string()
                            } else {
                                "failed a".to_string()
                            },
                            TextKind::Normal,
                        ),
                        (dc.key.to_string(), TextKind::Skill),
                        ("skill check".to_string(), TextKind::Normal),
                    ])
                    .render(ui);

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("DC:");
                            ui.same_line();
                            dc.render(ui);
                            ui.text("");
                            ui.text("Skill Check:");
                            ui.same_line();
                            result.render(ui);
                        });
                    }
                }

                GameEvent::LifeStateChanged {
                    entity,
                    new_state,
                    actor,
                } => {
                    let entity_name =
                        systems::helpers::get_component::<Name>(world, *entity).to_string();
                    let actor_name = actor.and_then(|a| {
                        Some(systems::helpers::get_component::<Name>(world, a).to_string())
                    });

                    match new_state {
                        LifeState::Normal => {
                            if let Some(actor_name) = actor_name {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was revived by".to_string(), TextKind::Normal),
                                    (actor_name, TextKind::Actor),
                                ])
                                .render(ui);
                            } else {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was revived".to_string(), TextKind::Normal),
                                ])
                                .render(ui);
                            }
                        }

                        LifeState::Unconscious(_) => {
                            if let Some(actor_name) = actor_name {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was knocked unconscious by".to_string(), TextKind::Normal),
                                    (actor_name, TextKind::Actor),
                                ])
                                .render(ui);
                            } else {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("fell unconscious".to_string(), TextKind::Normal),
                                ])
                                .render(ui);
                            }
                        }

                        LifeState::Stable => todo!(),

                        LifeState::Dead => {
                            if let Some(actor_name) = actor_name {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was killed by".to_string(), TextKind::Normal),
                                    (actor_name, TextKind::Actor),
                                ])
                                .render(ui);
                            } else {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("died".to_string(), TextKind::Normal),
                                ])
                                .render(ui);
                            }
                        }

                        LifeState::Defeated => {
                            if let Some(actor_name) = actor_name {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was defeated by".to_string(), TextKind::Normal),
                                    (actor_name, TextKind::Actor),
                                ])
                                .render(ui);
                            } else {
                                TextSegments::new(vec![
                                    (entity_name, TextKind::Actor),
                                    ("was defeated".to_string(), TextKind::Normal),
                                ])
                                .render(ui);
                            }
                        }
                    }
                }
            }
        }
    }
}
