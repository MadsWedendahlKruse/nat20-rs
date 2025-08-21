use hecs::Bundle;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        damage::DamageResistances,
        effects::effects::Effect,
        hit_points::HitPoints,
        id::Name,
        items::equipment::loadout::Loadout,
        level::ChallengeRating,
        race::{CreatureSize, CreatureType},
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
        pub size: CreatureSize,
        pub creature_type: CreatureType,
        pub abilities: AbilityScoreMap,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        // TODO: alignment?
        // TODO: ArmorClass or just Loadout?
        pub loadout: Loadout,
        // TODO: Speed
        pub spellbook: Spellbook,
        pub resources: ResourceMap,
        pub effects: Vec<Effect>,
        pub actions: ActionMap,
        pub cooldowns: ActionCooldownMap,
    }
);

impl Monster {
    pub fn new(
        name: Name,
        challenge_rating: ChallengeRating,
        hit_points: HitPoints,
        size: CreatureSize,
        creature_type: CreatureType,
        abilities: AbilityScoreMap,
    ) -> Self {
        Self {
            tag: MonsterTag,
            name,
            challenge_rating,
            hit_points,
            size,
            creature_type,
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
        }
    }
}
