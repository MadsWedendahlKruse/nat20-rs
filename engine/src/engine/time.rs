use std::{collections::HashMap, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::{
    engine::{
        encounter::EncounterId,
        event::{EncounterEvent, Event, EventKind},
        interaction::InteractionScopeId,
    },
    systems,
};

pub type TurnListenerId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnBoundary {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnKey {
    pub encounter_id: EncounterId,
    pub entity: Entity,
    pub boundary: TurnBoundary,
}

/// Callback invoked when the scheduled boundary triggers.
pub type TurnCallback = Arc<dyn Fn(&mut World) + Send + Sync + 'static>;

#[derive(Clone)]
struct ScheduledTurnCallback {
    pub id: TurnListenerId,
    pub remaining: u32,
    pub callback: TurnCallback,
}

/// Scheduler stored on GameState (singleton).
///
/// Design goals:
/// - O(k) per boundary: only touch listeners bound to that (encounter, entity, boundary)
/// - No coupling to effects; effects can register callbacks that remove effects, etc.
#[derive(Default)]
pub struct TurnScheduler {
    listeners: HashMap<TurnKey, Vec<ScheduledTurnCallback>>,
    // Optional reverse index for faster cancellations:
    // index_by_listener: HashMap<TurnListenerId, TurnKey>,
}

impl TurnScheduler {
    pub fn register(
        &mut self,
        key: TurnKey,
        remaining: u32,
        callback: TurnCallback,
    ) -> TurnListenerId {
        let id = Uuid::new_v4();
        let entry = self.listeners.entry(key).or_default();
        debug!(
            "Registering turn listener {:?} for key {:?} with {} remaining",
            id, key, remaining
        );
        entry.push(ScheduledTurnCallback {
            id,
            remaining: remaining.max(1),
            callback,
        });
        id
    }

    pub fn cancel(&mut self, listener_id: TurnListenerId) -> bool {
        let mut removed = false;
        for (_key, list) in self.listeners.iter_mut() {
            let before = list.len();
            list.retain(|s| s.id != listener_id);
            removed |= list.len() != before;
        }
        // Optional: prune empty lists
        self.listeners.retain(|_, v| !v.is_empty());
        removed
    }

    pub fn cancel_for_entity(&mut self, encounter_id: EncounterId, entity: Entity) {
        self.listeners
            .retain(|key, _| !(key.encounter_id == encounter_id && key.entity == entity));
    }

    pub fn cancel_for_encounter(&mut self, encounter_id: EncounterId) {
        self.listeners
            .retain(|key, _| key.encounter_id != encounter_id);
    }

    fn tick(&mut self, world: &mut World, key: TurnKey) {
        let Some(list) = self.listeners.get_mut(&key) else {
            debug!("No turn listeners for key {:?}", key);
            return;
        };

        // We want to run callbacks whose remaining reaches 0.
        // We also want to allow callbacks to register additional listeners safely.
        // So: gather fire-now callbacks in a separate vec.
        let mut fire_now: Vec<TurnCallback> = Vec::new();

        for scheduled in list.iter_mut() {
            if scheduled.remaining <= 1 {
                debug!("Firing turn listener {:?} for key {:?}", scheduled.id, key);
                fire_now.push(Arc::clone(&scheduled.callback));
            } else {
                scheduled.remaining -= 1;
            }
        }

        // Remove the fired callbacks from the list
        list.retain(|s| s.remaining > 1);

        // Prune empty bucket
        if list.is_empty() {
            self.listeners.remove(&key);
        }

        // Fire callbacks (after mutation is done)
        for callback in fire_now {
            (callback)(world);
        }
    }

    pub fn on_event(&mut self, world: &mut World, scope: InteractionScopeId, event: &Event) {
        let InteractionScopeId::Encounter(encounter_id) = scope else {
            return;
        };

        let EventKind::Encounter(encounter_event) = &event.kind else {
            return;
        };

        debug!("TurnScheduler received event: {:?}", encounter_event);

        match encounter_event {
            EncounterEvent::EncounterEnded(uuid, _) => {
                if *uuid == encounter_id {
                    self.cancel_for_encounter(*uuid);
                }
            }

            EncounterEvent::TurnBoundary {
                encounter_id,
                entity,
                boundary,
                ..
            } => {
                let key = TurnKey {
                    encounter_id: *encounter_id,
                    entity: *entity,
                    boundary: *boundary,
                };

                // TODO: Not sure what the best place to handle this is
                // It's sort of like a hard-coded callback for all entities
                match boundary {
                    TurnBoundary::Start => systems::time::on_turn_start(world, *entity),
                    TurnBoundary::End => systems::time::on_turn_end(world, *entity),
                }

                self.tick(world, key);
            }
            _ => return,
        }
    }
}
