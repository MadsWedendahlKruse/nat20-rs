use std::collections::HashMap;

use crate::{
    components::id::{ActionId, EffectId, RaceId, SubraceId},
    systems,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

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
    pub speed: u8,
}

#[derive(Debug, Clone)]
pub struct Subrace {
    pub id: SubraceId,
    pub base: RaceBase,
}
