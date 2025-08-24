use hecs::Bundle;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        damage::DamageResistances,
        effects::effects::Effect,
        health::{hit_points::HitPoints, life_state::LifeState},
        id::Name,
        items::equipment::{
            armor::ArmorTrainingSet, loadout::Loadout, weapon::WeaponProficiencyMap,
        },
        level::ChallengeRating,
        race::{CreatureSize, CreatureType, Speed},
        resource::ResourceMap,
        saving_throw::SavingThrowSet,
        skill::SkillSet,
        spells::spellbook::Spellbook,
    },
    from_world,
};

#[derive(Debug, Clone)]
pub struct MonsterTag;

from_world!(
    #[derive(Bundle, Clone)]
    pub struct Monster {
        pub tag: MonsterTag,
        pub name: Name,
        pub challenge_rating: ChallengeRating,
        pub hit_points: HitPoints,
        pub life_state: LifeState,
        pub size: CreatureSize,
        pub creature_type: CreatureType,
        pub speed: Speed,
        pub abilities: AbilityScoreMap,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        // TODO: alignment?
        // TODO: ArmorClass or just Loadout?
        pub loadout: Loadout,
        pub spellbook: Spellbook,
        pub resources: ResourceMap,
        pub effects: Vec<Effect>,
        pub actions: ActionMap,
        pub cooldowns: ActionCooldownMap,
        pub weapon_proficiencies: WeaponProficiencyMap,
        pub armor_training: ArmorTrainingSet,
    }
);

impl Monster {
    pub fn new(
        name: Name,
        challenge_rating: ChallengeRating,
        hit_points: HitPoints,
        size: CreatureSize,
        creature_type: CreatureType,
        speed: Speed,
        abilities: AbilityScoreMap,
    ) -> Self {
        Self {
            tag: MonsterTag,
            name,
            challenge_rating,
            hit_points,
            life_state: LifeState::Normal,
            size,
            creature_type,
            speed,
            abilities,
            skills: SkillSet::default(),
            saving_throws: SavingThrowSet::default(),
            resistances: DamageResistances::default(),
            loadout: Loadout::default(),
            spellbook: Spellbook::new(),
            resources: ResourceMap::default(),
            effects: Vec::new(),
            actions: ActionMap::default(),
            cooldowns: ActionCooldownMap::default(),
            weapon_proficiencies: WeaponProficiencyMap::new(),
            armor_training: ArmorTrainingSet::default(),
        }
    }
}
