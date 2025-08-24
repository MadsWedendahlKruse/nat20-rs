use std::collections::HashMap;

use hecs::Bundle;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        damage::DamageResistances,
        effects::effects::Effect,
        health::{hit_points::HitPoints, life_state::LifeState},
        id::{BackgroundId, FeatId, Name, RaceId, SubraceId},
        items::{
            equipment::{armor::ArmorTrainingSet, loadout::Loadout, weapon::WeaponProficiencyMap},
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
        pub race: RaceId,
        pub subrace: Option<SubraceId>,
        pub size: CreatureSize,
        pub creature_type: CreatureType,
        pub speed: Speed,
        pub background: BackgroundId,
        pub levels: CharacterLevels,
        pub hit_points: HitPoints,
        pub life_state: LifeState,
        pub ability_scores: AbilityScoreMap,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        pub weapon_proficiencies: WeaponProficiencyMap,
        pub armor_training: ArmorTrainingSet,
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
            race: RaceId::from_str(""),
            subrace: None,
            background: BackgroundId::from_str(""),
            size: CreatureSize::Medium,
            creature_type: CreatureType::Humanoid,
            speed: Speed(0),
            levels: CharacterLevels::new(),
            hit_points: HitPoints::new(1),
            life_state: LifeState::Normal,
            ability_scores: AbilityScoreMap::new(),
            skills: SkillSet::default(),
            saving_throws: SavingThrowSet::default(),
            resistances: DamageResistances::new(),
            armor_training: ArmorTrainingSet::new(),
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
