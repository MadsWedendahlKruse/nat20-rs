use std::collections::HashMap;

use hecs::Bundle;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionCooldownMap, ActionMap},
        damage::DamageResistances,
        effects::effects::Effect,
        hit_points::HitPoints,
        id::FeatId,
        items::equipment::{loadout::Loadout, weapon::WeaponProficiencyMap},
        level::CharacterLevels,
        resource::{RechargeRule, Resource, ResourceMap},
        saving_throw::{SavingThrowSet, create_saving_throw_set},
        skill::{SkillSet, create_skill_set},
        spells::spellbook::Spellbook,
    },
    registry::{self},
};

macro_rules! from_world {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty,
            )*
        }
    ) => {
        use hecs::{World, Entity};
        use crate::systems;

        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field : $ty,
            )*
        }

        impl $name {
            pub fn from_world(world: &World, entity: Entity) -> Self {
                Self {
                    $(
                        $field: systems::helpers::get_component_clone(world, entity),
                    )*
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharacterTag;

from_world!(
    #[derive(Bundle, Clone)]
    pub struct Character {
        pub tag: CharacterTag,
        pub name: String,
        pub levels: CharacterLevels,
        pub hp: HitPoints,
        pub ability_scores: AbilityScoreMap,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        pub weapon_proficiencies: WeaponProficiencyMap,
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
    pub fn new(name: &str) -> Self {
        // TODO: Not sure this is the best place to put this?
        // By default everyone has one action, bonus action and reaction
        let mut resources = ResourceMap::new();
        for resource in [
            registry::resources::ACTION.clone(),
            registry::resources::BONUS_ACTION.clone(),
            registry::resources::REACTION.clone(),
        ] {
            resources.add(
                Resource::new(resource, 1, RechargeRule::OnTurn).unwrap(),
                true,
            );
        }
        Self {
            tag: CharacterTag,
            name: name.to_string(),
            levels: CharacterLevels::new(),
            hp: HitPoints::new(0),
            ability_scores: AbilityScoreMap::new(),
            skills: create_skill_set(),
            saving_throws: create_saving_throw_set(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: WeaponProficiencyMap::new(),
            loadout: Loadout::new(),
            spellbook: Spellbook::new(),
            resources,
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
        Character::new("John Doe")
    }
}
