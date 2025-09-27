use std::collections::HashMap;

use hecs::Bundle;
use parry3d::na::Isometry3;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        ai::PlayerControlledTag,
        damage::DamageResistances,
        effects::effects::Effect,
        faction::FactionSet,
        health::{hit_points::HitPoints, life_state::LifeState},
        id::{AIControllerId, BackgroundId, FeatId, Name, RaceId, SubraceId},
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
    from_world, registry,
    systems::geometry::CreaturePose,
};

#[derive(Debug, Clone)]
pub struct CharacterTag;

from_world!(
    #[derive(Bundle, Clone)]
    pub struct Character {
        pub character_tag: CharacterTag,
        /// By default, characters are player controlled. In case the player gets
        /// possessed or mind controlled, this component can be removed from the
        /// entity to make it AI controlled.
        pub player_controlled: PlayerControlledTag,
        /// AI controller for this character. Ignored if `player_controlled` is present.
        pub brain: AIControllerId,
        pub pose: CreaturePose,
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
        pub factions: FactionSet,
    }
);

impl Character {
    pub fn new(name: Name) -> Self {
        Self {
            character_tag: CharacterTag,
            player_controlled: PlayerControlledTag,
            // TODO: Update to an actual ID
            brain: registry::ai::RANDOM_CONTROLLER_ID.clone(),
            pose: CreaturePose::identity(),
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
            factions: FactionSet::from([registry::factions::PLAYERS_ID.clone()]),
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new(Name::new("John Doe"))
    }
}
