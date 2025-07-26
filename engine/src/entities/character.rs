use std::{collections::HashMap, sync::LazyLock};

use hecs::{Bundle, Entity, Query, Ref, RefMut, With, World};

use crate::{
    components::{
        ability::AbilityScoreSet,
        actions::action::{ActionContext, ActionMap},
        class::ClassName,
        damage::DamageResistances,
        effects::effects::Effect,
        hit_points::HitPoints,
        id::{ActionId, CharacterId, ResourceId},
        items::equipment::{loadout::Loadout, weapon::WeaponProficiencyMap},
        level::CharacterLevels,
        resource::{RechargeRule, Resource, ResourceMap},
        saving_throw::{SavingThrowSet, create_saving_throw_set},
        skill::{SkillSet, create_skill_set},
        spells::spellbook::Spellbook,
    },
    registry::{self},
    systems,
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
        pub id: CharacterId,
        pub name: String,
        pub levels: CharacterLevels,
        pub hp: HitPoints,
        pub ability_scores: AbilityScoreSet,
        pub skills: SkillSet,
        pub saving_throws: SavingThrowSet,
        pub resistances: DamageResistances,
        pub weapon_proficiencies: WeaponProficiencyMap,
        pub loadout: Loadout,
        pub spellbook: Spellbook,
        pub resources: ResourceMap,
        pub effects: Vec<Effect>,
        pub actions: ActionMap,
        pub cooldowns: HashMap<ActionId, RechargeRule>,
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
            id: CharacterId::new_v4(),
            name: name.to_string(),
            levels: CharacterLevels::new(),
            hp: HitPoints::new(0),
            ability_scores: AbilityScoreSet::new(),
            skills: create_skill_set(),
            saving_throws: create_saving_throw_set(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: WeaponProficiencyMap::new(),
            loadout: Loadout::new(),
            spellbook: Spellbook::new(),
            resources,
            effects: Vec::new(),
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
