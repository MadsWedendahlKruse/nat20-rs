use std::collections::HashMap;

use hecs::Bundle;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        damage::DamageResistances,
        effects::effects::Effect,
        hit_points::HitPoints,
        id::{BackgroundId, FeatId, Name, RaceId, SubraceId},
        items::{
            equipment::{loadout::Loadout, weapon::WeaponProficiencyMap},
            inventory::Inventory,
        },
        level::CharacterLevels,
        race::{CreatureSize, CreatureType, Speed},
        resource::ResourceMap,
        saving_throw::SavingThrowSet,
        skill::SkillSet,
        spells::spellbook::Spellbook,
    },
    from_world,
};

#[derive(Debug, Clone)]
pub struct CharacterTag;

from_world!(
    #[derive(Bundle, Clone)]
    pub struct Character {
        pub tag: CharacterTag,
        pub name: Name,
        // TODO: Not sure if Option makes sense here, it would only be at the
        // very beginning of character creation where they are not set
        pub race: Option<RaceId>,
        pub subrace: Option<SubraceId>,
        pub size: Option<CreatureSize>,
        pub creature_type: Option<CreatureType>,
        pub speed: Option<Speed>,
        pub background: Option<BackgroundId>,
        pub levels: CharacterLevels,
        pub hit_points: HitPoints,
        pub ability_scores: AbilityScoreMap,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        pub weapon_proficiencies: WeaponProficiencyMap,
        pub inventory: Inventory,
        pub loadout: Loadout,
        pub spellbook: Spellbook,
        pub resources: ResourceMap,
        pub effects: Vec<Effect>,
        pub feats: Vec<FeatId>,
        pub actions: ActionMap,
        pub cooldowns: ActionCooldownMap,
    }
);

impl Character {
    pub fn new(name: Name) -> Self {
        Self {
            tag: CharacterTag,
            name,
            race: None,
            subrace: None,
            background: None,
            size: None,
            creature_type: None,
            speed: None,
            levels: CharacterLevels::new(),
            hit_points: HitPoints::new(0),
            ability_scores: AbilityScoreMap::new(),
            skills: SkillSet::default(),
            saving_throws: SavingThrowSet::default(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: WeaponProficiencyMap::new(),
            loadout: Loadout::new(),
            inventory: Inventory::new(),
            spellbook: Spellbook::new(),
            resources: ResourceMap::default(),
            effects: Vec::new(),
            feats: Vec::new(),
            // TODO: Default actions like jump, dash, help, etc.
            actions: ActionMap::new(),
            cooldowns: HashMap::new(),
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new(Name::new("John Doe"))
    }
}
