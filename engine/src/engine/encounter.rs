use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use hecs::{Entity, World};
use uuid::Uuid;

use crate::{
    components::{
        actions::{action::ReactionResult, targeting::TargetType},
        ai::{AIController, PlayerControlledTag},
        d20::{D20CheckDC, D20CheckResult},
        health::life_state::{DEATH_SAVING_THROW_DC, LifeState},
        id::{AIControllerId, ActionId},
        modifier::{ModifierSet, ModifierSource},
        resource::{RechargeRule, ResourceMap},
        saving_throw::SavingThrowKind,
        skill::{Skill, SkillSet},
    },
    engine::{
        event::{
            self, ActionData, ActionDecision, ActionError, ActionPrompt, CallbackResult,
            EncounterEvent, Event, EventId, EventKind, EventListener, EventLog, EventQueue,
            ReactionData,
        },
        game_state::{self, GameState},
    },
    entities::{character::CharacterTag, monster::MonsterTag},
    registry::{self, resources},
    systems,
};

pub type EncounterId = Uuid;

pub enum ParticipantsFilter {
    All,
    Characters,
    Monsters,
    Specific(HashSet<Entity>),
    LifeStates(HashSet<LifeState>),
    NotLifeStates(HashSet<LifeState>),
}

impl From<TargetType> for ParticipantsFilter {
    fn from(value: TargetType) -> Self {
        match value {
            TargetType::Entity {
                allowed_states,
                invert,
            } => {
                if invert {
                    ParticipantsFilter::NotLifeStates(allowed_states)
                } else {
                    ParticipantsFilter::LifeStates(allowed_states)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Encounter {
    id: EncounterId,
    participants: HashSet<Entity>,
    round: usize,
    turn_index: usize,
    initiative_order: Vec<(Entity, D20CheckResult)>,
    /// Pending prompts for the participants in this encounter
    pending_prompts: VecDeque<ActionPrompt>,
    event_log: EventLog,
}

impl Encounter {
    pub fn new(game_state: &mut GameState, participants: HashSet<Entity>, id: EncounterId) -> Self {
        let mut encounter = Self {
            id,
            participants,
            round: 1,
            turn_index: 0,
            initiative_order: Vec::new(),
            pending_prompts: VecDeque::new(),
            event_log: EventLog::new(),
        };
        encounter.roll_initiative(&game_state.world);
        encounter.start_turn(game_state);
        encounter
            .event_log
            .push(Event::encounter_event(EncounterEvent::NewRound(
                encounter.id.clone(),
                encounter.round(),
            )));
        encounter
    }

    fn roll_initiative(&mut self, world: &World) {
        let mut indexed_rolls: Vec<(Entity, D20CheckResult)> = self
            .participants
            .iter()
            .map(|entity| {
                let roll = systems::helpers::get_component::<SkillSet>(world, *entity).check(
                    Skill::Initiative,
                    world,
                    *entity,
                );
                (entity.clone(), roll)
            })
            .collect();

        indexed_rolls.sort_by_key(|(_, roll)| -(roll.total as i32));
        self.initiative_order = indexed_rolls
            .into_iter()
            .map(|(i, roll)| (i, roll))
            .collect();
    }

    pub fn initiative_order(&self) -> &Vec<(Entity, D20CheckResult)> {
        &self.initiative_order
    }

    pub fn current_entity(&self) -> Entity {
        let (idx, _) = self.initiative_order[self.turn_index];
        idx
    }

    pub fn pending_prompts(&self) -> &VecDeque<ActionPrompt> {
        &self.pending_prompts
    }

    pub(crate) fn queue_prompt(&mut self, prompt: ActionPrompt, at_front: bool) {
        if at_front {
            self.pending_prompts.push_front(prompt);
        } else {
            self.pending_prompts.push_back(prompt);
        }
    }

    pub fn next_pending_prompt(&self) -> Option<&ActionPrompt> {
        self.pending_prompts.front()
    }

    pub(crate) fn pop_prompt(&mut self) -> Option<ActionPrompt> {
        self.pending_prompts.pop_front()
    }

    pub fn participants(&self, world: &World, filter: ParticipantsFilter) -> Vec<Entity> {
        match filter {
            ParticipantsFilter::All => self.participants.iter().cloned().collect(),

            ParticipantsFilter::Characters => world
                .query::<&CharacterTag>()
                .iter()
                .map(|(e, _)| e)
                .collect(),

            ParticipantsFilter::Monsters => world
                .query::<&MonsterTag>()
                .iter()
                .map(|(e, _)| e)
                .collect(),

            ParticipantsFilter::Specific(entities) => {
                self.participants.intersection(&entities).cloned().collect()
            }

            ParticipantsFilter::LifeStates(life_states) => world
                .query::<&LifeState>()
                .iter()
                .filter_map(|(e, ls)| {
                    if life_states.contains(ls) {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect(),

            ParticipantsFilter::NotLifeStates(life_states) => world
                .query::<&LifeState>()
                .iter()
                .filter_map(|(e, ls)| {
                    if !life_states.contains(ls) {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    fn start_turn(&mut self, game_state: &mut GameState) {
        systems::time::pass_time(
            &mut game_state.world,
            self.current_entity(),
            &RechargeRule::OnTurn,
        );

        if self.should_skip_turn(game_state) {
            self.end_turn(game_state, self.current_entity());
            return;
        }

        self.pending_prompts.push_back(ActionPrompt::Action {
            actor: self.current_entity(),
        });
    }

    pub fn end_turn(&mut self, game_state: &mut GameState, entity: Entity) {
        if entity != self.current_entity() {
            panic!("Cannot end turn for entity that is not the current entity");
        }

        for prompt in self.pending_prompts.iter() {
            for respondent in prompt.actors() {
                if respondent != entity {
                    panic!(
                        "Attempted to end turn for {:?} but there is a pending prompt for {:?}",
                        entity, respondent
                    );
                }
            }
        }

        self.pending_prompts.clear();

        self.turn_index = (self.turn_index + 1) % self.participants.len();
        if self.turn_index == 0 {
            self.round += 1;
            self.event_log
                .push(Event::encounter_event(EncounterEvent::NewRound(
                    self.id.clone(),
                    self.round(),
                )));
        }

        self.start_turn(game_state);
    }

    // TODO: Some of this feels like it should belong somewhere else?
    fn should_skip_turn(&mut self, game_state: &mut GameState) -> bool {
        let current_entity = self.current_entity();

        // Borrow checker forces potential death saving throws to be handled in
        // a specific order

        // 1) Peek life state without taking a mutable borrow
        let is_unconscious = matches!(
            *systems::helpers::get_component::<LifeState>(&game_state.world, current_entity),
            LifeState::Unconscious(_)
        );

        if is_unconscious {
            // TODO: Re-implement death saving throws

            // 2) Do the d20 roll (needs only &World)
            // let check_dc = D20CheckDC {
            //     dc: ModifierSet::from_iter([(
            //         ModifierSource::Custom("Death Saving Throw".to_string()),
            //         DEATH_SAVING_THROW_DC as i32,
            //     )]),
            //     key: SavingThrowKind::Death,
            // };
            // let check_result = systems::d20::saving_throw_dc(world, current_entity, &check_dc);
            // self.event_log.push(Event::SavingThrow(
            //     current_entity,
            //     check_result.clone(),
            //     check_dc.clone(),
            // ));

            // // 3) Now take the mutable borrow and update
            // let mut life_state =
            //     systems::helpers::get_component_mut::<LifeState>(world, current_entity);
            // if let LifeState::Unconscious(ref mut death_saving_throws) = *life_state {
            //     death_saving_throws.update(check_result);

            //     let next_state = death_saving_throws.next_state();

            //     if next_state != *life_state {
            //         *life_state = next_state.clone();

            //         self.event_log.push(Event::new(EventKind::LifeStateChanged {
            //             entity: current_entity,
            //             new_state: next_state,
            //             actor: None,
            //         }));
            //     }
            // }

            return true;
        } else {
            // Normal / other states => decide if they can act
            return !matches!(
                *systems::helpers::get_component::<LifeState>(&game_state.world, current_entity),
                LifeState::Normal
            );
        };
    }

    pub fn round(&self) -> usize {
        self.round
    }

    pub(crate) fn log_event(&mut self, event: Event) {
        self.event_log.push(event);
    }

    pub fn combat_log(&self) -> &EventLog {
        &self.event_log
    }

    pub fn combat_log_move(&mut self) -> EventLog {
        std::mem::take(&mut self.event_log)
    }
}
