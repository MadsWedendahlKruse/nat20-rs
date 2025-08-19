use std::{collections::HashMap, sync::LazyLock};

use crate::{
    components::{
        id::{EffectId, RaceId, SubraceId},
        level_up::LevelUpPrompt,
        race::{CreatureSize, CreatureType, Race, RaceBase, Subrace},
    },
    registry,
};

pub static RACE_REGISTRY: LazyLock<HashMap<RaceId, Race>> =
    LazyLock::new(|| HashMap::from([(DRAGONBORN_ID.clone(), DRAGONBORN.to_owned())]));

pub static DRAGONBORN_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.dragonborn"));

static DRAGONBORN: LazyLock<Race> = LazyLock::new(|| Race {
    id: DRAGONBORN_ID.clone(),
    base: RaceBase {
        // TODO: Darkvision
        effects_by_level: HashMap::new(),
        // TODO: Draconic flight
        actions_by_level: HashMap::new(),
    },
    subraces: HashMap::from([
        (DRAGONBORN_BLACK.id.clone(), DRAGONBORN_BLACK.to_owned()),
        (DRAGONBORN_BLUE.id.clone(), DRAGONBORN_BLUE.to_owned()),
        (DRAGONBORN_BRASS.id.clone(), DRAGONBORN_BRASS.to_owned()),
        (DRAGONBORN_BRONZE.id.clone(), DRAGONBORN_BRONZE.to_owned()),
        (DRAGONBORN_COPPER.id.clone(), DRAGONBORN_COPPER.to_owned()),
        (DRAGONBORN_GOLD.id.clone(), DRAGONBORN_GOLD.to_owned()),
        (DRAGONBORN_GREEN.id.clone(), DRAGONBORN_GREEN.to_owned()),
        (DRAGONBORN_RED.id.clone(), DRAGONBORN_RED.to_owned()),
        (DRAGONBORN_SILVER.id.clone(), DRAGONBORN_SILVER.to_owned()),
        (DRAGONBORN_WHITE.id.clone(), DRAGONBORN_WHITE.to_owned()),
    ]),
    creature_type: CreatureType::Humanoid,
    size: CreatureSize::Medium,
    speed: 30,
});

macro_rules! dragonborn_subraces {
    ($( $Name:ident => $slug:literal => [ $( $effect_id:path ),+ $(,)? ] ),+ $(,)?) => {
        use paste::paste;
        paste! {
            $(
                pub static [<$Name _ID>]: LazyLock<SubraceId> =
                    LazyLock::new(|| SubraceId::from_str(concat!("race.dragonborn.", $slug)));

                static $Name: LazyLock<Subrace> = LazyLock::new(|| Subrace {
                    id: [<$Name _ID>].clone(),
                    base: RaceBase {
                        effects_by_level: {
                            let mut m: HashMap<u8, Vec<EffectId>> = HashMap::new();
                            m.insert(1, vec![ $( $effect_id.clone() ),+ ]);
                            m
                        },
                        actions_by_level: HashMap::new(),
                    },
                });
            )+
        }
    }
}

dragonborn_subraces!(
    DRAGONBORN_BLACK  => "black"  => [registry::effects::DRACONIC_ANCESTRY_BLACK_ID],
    DRAGONBORN_BLUE   => "blue"   => [registry::effects::DRACONIC_ANCESTRY_BLUE_ID],
    DRAGONBORN_BRASS  => "brass"  => [registry::effects::DRACONIC_ANCESTRY_BRASS_ID],
    DRAGONBORN_BRONZE => "bronze" => [registry::effects::DRACONIC_ANCESTRY_BRONZE_ID],
    DRAGONBORN_COPPER => "copper" => [registry::effects::DRACONIC_ANCESTRY_COPPER_ID],
    DRAGONBORN_GOLD   => "gold"   => [registry::effects::DRACONIC_ANCESTRY_GOLD_ID],
    DRAGONBORN_GREEN  => "green"  => [registry::effects::DRACONIC_ANCESTRY_GREEN_ID],
    DRAGONBORN_RED    => "red"    => [registry::effects::DRACONIC_ANCESTRY_RED_ID],
    DRAGONBORN_SILVER => "silver" => [registry::effects::DRACONIC_ANCESTRY_SILVER_ID],
    DRAGONBORN_WHITE  => "white"  => [registry::effects::DRACONIC_ANCESTRY_WHITE_ID],
);

pub static DWARF_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.dwarf"));

pub static ELF_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.elf"));

pub static GNOME_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.gnome"));

pub static GOLIATH_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.goliath"));

pub static HALFLING_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.halfling"));

pub static HUMAN_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.human"));

pub static ORC_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.orc"));

pub static TIEFLING_ID: LazyLock<RaceId> = LazyLock::new(|| RaceId::from_str("race.tiefling"));
