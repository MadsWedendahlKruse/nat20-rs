use crate::components::faction::{Attitude, Faction};
use crate::components::id::FactionId;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static FACTION_REGISTRY: LazyLock<HashMap<FactionId, Faction>> = LazyLock::new(|| {
    HashMap::from([
        (GOBLINS_ID.clone(), GOBLINS.to_owned()),
        (PLAYERS_ID.clone(), PLAYERS.to_owned()),
    ])
});

pub static GOBLINS_ID: LazyLock<FactionId> =
    LazyLock::new(|| FactionId::from_str("faction.goblins"));

static GOBLINS: LazyLock<Faction> = LazyLock::new(|| {
    Faction::new(
        GOBLINS_ID.clone(),
        "Goblins".to_string(),
        HashMap::from([]),
        Attitude::Hostile,
        Attitude::Friendly,
    )
});

pub static PLAYERS_ID: LazyLock<FactionId> =
    LazyLock::new(|| FactionId::from_str("faction.players"));

static PLAYERS: LazyLock<Faction> = LazyLock::new(|| {
    Faction::new(
        PLAYERS_ID.clone(),
        "Players".to_string(),
        HashMap::from([]),
        Attitude::Neutral,
        Attitude::Friendly,
    )
});
