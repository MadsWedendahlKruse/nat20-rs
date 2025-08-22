use std::collections::HashMap;

use strum::Display;

use crate::components::id::{ActionId, EffectId, RaceId, SubraceId};

#[derive(Debug, Clone, Display)]
pub enum CreatureType {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,
}

#[derive(Debug, Clone, Display)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

// TODO: Do we need all these modes?
// pub struct Speed {
//     pub walk: u8,
//     pub burrow: Option<u8>,
//     pub climb: Option<u8>,
//     pub fly: Option<u8>,
//     pub swim: Option<u8>,
// }

#[derive(Debug, Clone)]
pub struct Speed(pub u8);

#[derive(Debug, Clone)]
pub struct RaceBase {
    pub effects_by_level: HashMap<u8, Vec<EffectId>>,
    pub actions_by_level: HashMap<u8, Vec<ActionId>>,
}

#[derive(Debug, Clone)]
pub struct Race {
    pub id: RaceId,
    pub base: RaceBase,
    pub subraces: HashMap<SubraceId, Subrace>,
    pub creature_type: CreatureType,
    pub size: CreatureSize,
    // TODO: Subraces can modify the speed using an effect?
    pub speed: Speed,
}

#[derive(Debug, Clone)]
pub struct Subrace {
    pub id: SubraceId,
    pub base: RaceBase,
}
