use std::cmp::min;

use hecs::World;
use imgui::TreeNodeFlags;
use nat20_rs::{
    components::{
        actions::action::{Action, ActionContext},
        health::life_state::LifeState,
        id::Name,
    },
    engine::{
        encounter,
        event::{EncounterEvent, Event, EventKind, EventLog},
    },
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};
use strum::{Display, EnumIter};

use crate::render::{
    components::new_life_state_text,
    text::{TextKind, TextSegment, TextSegments, indent_text},
    utils::{ImguiRenderable, ImguiRenderableWithContext},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIter, Display)]
pub enum LogLevel {
    Info,
    Debug,
}

impl From<usize> for LogLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => LogLevel::Info,
            1 => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

pub fn event_log_level(event: &Event) -> LogLevel {
    match &event.kind {
        EventKind::Encounter(_) => LogLevel::Info,
        EventKind::ActionRequested { .. } => LogLevel::Info,
        EventKind::ActionPerformed { .. } => LogLevel::Info,
        EventKind::ReactionTriggered { .. } => LogLevel::Info,
        EventKind::LifeStateChanged { .. } => LogLevel::Info,
        EventKind::D20CheckPerformed(_, _, _) => LogLevel::Debug,
        EventKind::D20CheckResolved(_, result_kind, _) => match result_kind {
            D20ResultKind::SavingThrow { .. } | D20ResultKind::Skill { .. } => LogLevel::Info,
            systems::d20::D20ResultKind::AttackRoll { .. } => LogLevel::Debug,
        },
        EventKind::DamageRollPerformed(_, _) => LogLevel::Debug,
        EventKind::DamageRollResolved(_, _) => LogLevel::Debug,
    }
}

pub fn render_event_description(ui: &imgui::Ui, event: &Event, world: &World) {
    let event_description = match &event.kind {
        EventKind::ActionRequested { action } => vec![
            (
                format!(
                    "{}'s",
                    systems::helpers::get_component::<Name>(world, event.actor().unwrap()).as_str(),
                ),
                TextKind::Actor,
            ),
            (format!("{}", action.action_id), TextKind::Action),
        ],
        _ => vec![(format!("{:?}", event.kind.name()), TextKind::Details)],
    };

    TextSegments::new(event_description).render(ui);
}

pub fn events_match(event1: &Event, event2: &Event) -> bool {
    match (&event1.kind, &event2.kind) {
        (
            EventKind::ActionRequested { action: a1 },
            EventKind::ActionPerformed { action: a2, .. },
        ) => a1.actor == a2.actor && a1.action_id == a2.action_id && a1.targets == a2.targets,

        (EventKind::D20CheckPerformed(e1, _, _), EventKind::D20CheckResolved(e2, _, _)) => e1 == e2,

        (EventKind::DamageRollPerformed(e1, _), EventKind::DamageRollResolved(e2, _)) => e1 == e2,

        _ => false,
    }
}

impl ImguiRenderableWithContext<&(&World, &LogLevel)> for EventLog {
    fn render_with_context(&self, ui: &imgui::Ui, context: &(&World, &LogLevel)) {
        let (_, log_level) = context;

        let log_level_events = self
            .events
            .iter()
            .filter(|event| event_log_level(event) <= **log_level)
            .collect::<Vec<_>>();

        for (i, entry) in log_level_events.iter().enumerate() {
            // For visual clarity at 'Info' level, we don't need to see e.g. both
            // the 'ActionRequested' and 'ActionPerformed' events, so if two
            // consecutive events "match" then we only show the first one, e.g.
            // for an action we would only show the 'ActionPerformed' event.
            if **log_level == LogLevel::Info && i < log_level_events.len() - 1 {
                let next_entry = &log_level_events[i + 1];
                if events_match(entry, next_entry) {
                    continue;
                }
            }

            entry.render_with_context(ui, context);
        }
    }
}

impl ImguiRenderableWithContext<&(&World, &LogLevel)> for Event {
    fn render_with_context(&self, ui: &imgui::Ui, context: &(&World, &LogLevel)) {
        let (world, log_level) = context;

        let group_token = ui.begin_group();

        match &self.kind {
            EventKind::Encounter(encounter_event) => match encounter_event {
                EncounterEvent::EncounterStarted(encounter_id) => {
                    ui.separator_with_text(&format!("Encounter {}", encounter_id));
                }

                EncounterEvent::EncounterEnded(encounter_id, combat_log) => {
                    if ui.collapsing_header(format!("Log##{}", encounter_id), TreeNodeFlags::FRAMED)
                    {
                        combat_log.render_with_context(ui, context);
                    }
                    ui.separator();
                }

                EncounterEvent::NewRound(encounter_id, round) => {
                    ui.separator_with_text(format!("Round {}", round));
                }
            },

            EventKind::ActionRequested { action } => {
                TextSegments::new(vec![
                    (
                        &systems::helpers::get_component::<Name>(world, action.actor).to_string(),
                        TextKind::Actor,
                    ),
                    (&"is using".to_string(), TextKind::Normal),
                    (&action.action_id.to_string(), TextKind::Action),
                ])
                .render(ui);

                let self_target = action.targets.len() == 1 && action.targets[0] == action.actor;

                if !self_target {
                    ui.same_line();
                    TextSegment::new("on", TextKind::Normal).render(ui);
                    action.targets.render_with_context(ui, &world);
                }

                match &action.context {
                    ActionContext::Reaction {
                        trigger_event,
                        resource_cost,
                        context,
                    } => {
                        ui.same_line();
                        TextSegment::new("as a response to".to_string(), TextKind::Normal)
                            .render(ui);
                        ui.same_line();
                        render_event_description(ui, trigger_event, world);
                    }

                    _ => {}
                }
            }

            EventKind::ActionPerformed { action, results } => {
                TextSegments::new(vec![
                    (
                        &systems::helpers::get_component::<Name>(world, action.actor).to_string(),
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
                    result.render_with_context(ui, (&world, 0));
                }
            }

            EventKind::ReactionTriggered {
                trigger_event,
                reactors,
            } => {
                render_event_description(ui, &trigger_event, world);

                ui.same_line();

                TextSegment::new("triggered a reaction from".to_string(), TextKind::Normal)
                    .render(ui);

                reactors
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .render_with_context(ui, &world);
            }

            EventKind::LifeStateChanged {
                entity,
                new_state,
                actor,
            } => {
                let entity_name =
                    systems::helpers::get_component::<Name>(world, *entity).to_string();
                let actor_name = actor.and_then(|a| {
                    Some(systems::helpers::get_component::<Name>(world, a).to_string())
                });

                let segments = new_life_state_text(&entity_name, new_state, actor_name.as_deref());
                TextSegments::new(segments).render(ui);
            }

            EventKind::D20CheckResolved(entity, result_kind, dc_kind)
            | EventKind::D20CheckPerformed(entity, result_kind, dc_kind) => {
                if let Some(dc_kind) = dc_kind {
                    let dc_text_segments = match dc_kind {
                        D20CheckDCKind::SavingThrow(dc) => {
                            vec![
                                (dc.key.to_string(), TextKind::Ability),
                                ("check".to_string(), TextKind::Normal),
                            ]
                        }
                        D20CheckDCKind::Skill(dc) => vec![
                            (dc.key.to_string(), TextKind::Ability),
                            ("check".to_string(), TextKind::Normal),
                        ],
                        D20CheckDCKind::AttackRoll(_, target, _) => {
                            let target_name =
                                systems::helpers::get_component::<Name>(world, *target);
                            vec![
                                ("attack roll against".to_string(), TextKind::Normal),
                                (target_name.to_string(), TextKind::Actor),
                            ]
                        }
                    };

                    let mut segments = vec![
                        (
                            systems::helpers::get_component::<Name>(world, *entity).to_string(),
                            TextKind::Actor,
                        ),
                        (
                            if result_kind.is_success(dc_kind) {
                                "succeeded a".to_string()
                            } else {
                                "failed a".to_string()
                            },
                            TextKind::Normal,
                        ),
                    ];

                    segments.extend(dc_text_segments);

                    TextSegments::new(segments).render(ui);

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("DC:");
                            ui.same_line();
                            dc_kind.render(ui);
                            ui.text("");
                            ui.text("D20 Check:");
                            ui.same_line();
                            result_kind.render(ui);
                        });
                    }
                }
            }
            EventKind::DamageRollPerformed(entity, damage_roll_result)
            | EventKind::DamageRollResolved(entity, damage_roll_result) => {
                TextSegments::new(vec![
                    (
                        systems::helpers::get_component::<Name>(world, *entity).to_string(),
                        TextKind::Details,
                    ),
                    ("rolled".to_string(), TextKind::Details),
                    (format!("{}", damage_roll_result.total), TextKind::Details),
                    ("damage".to_string(), TextKind::Details),
                ])
                .render(ui);

                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        damage_roll_result.render(ui);
                    });
                }
            }
        }

        group_token.end();

        if ui.is_item_hovered() && ui.is_key_pressed_no_repeat(imgui::Key::ModCtrl) {
            ui.open_popup(self.id.to_string());
        }

        ui.popup(self.id.to_string(), || {
            let debug_text = format!("{:#?}", self);
            let size = ui.calc_text_size(&debug_text);
            ui.child_window("Event Debug Info")
                .size([size[0] + 10.0, f32::min(size[1], 400.0)])
                .build(|| {
                    ui.text(debug_text);
                });
        });
    }
}
