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

// TODO: I feel like this View thing is an anti-pattern
/// Macro to generate a struct and a corresponding view struct with references to its fields.
// macro_rules! struct_with_view {
//     (
//         $(#[$meta:meta])*
//         $vis:vis struct $name:ident {
//             $(
//                 $(#[$field_meta:meta])*
//                 $field_vis:vis $field:ident : $ty:ty,
//             )*
//         }
//         view $view_name:ident
//         view_mut $view_mut_name:ident
//     ) => {
//         $(#[$meta])*
//         $vis struct $name {
//             $(
//                 $(#[$field_meta])*
//                 $field_vis $field : $ty,
//             )*
//         }

//         $vis struct $view_name<'a> {
//             $(
//                 $(#[$field_meta])*
//                 $field_vis $field : Ref<'a, $ty>,
//             )*
//         }

//         impl<'a> $view_name<'a> {
//             pub fn from_world(world: &'a World, entity: Entity) -> Self {
//                 Self {
//                     $(
//                         $field: systems::helpers::get_component(world, entity),
//                     )*
//                 }
//             }
//         }

//         $vis struct $view_mut_name<'a> {
//             $(
//                 $(#[$field_meta])*
//                 $field_vis $field : &'a mut $ty,
//             )*
//         }

//         impl<'a> $view_mut_name<'a> {
//             pub fn from_world(world: &'a mut World, entity: Entity) -> Self {
//                 let (
//                     $($field,)*
//                 ) = world
//                     .query_one_mut::<(
//                         $( &mut $ty , )*
//                     )>(entity)
//                     .unwrap();

//                 Self {
//                     $($field,)*
//                 }
//             }
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct CharacterTag;

// struct_with_view! {
//     #[derive(Bundle)]
//     pub struct Character {
//         pub tag: CharacterTag,
//         pub id: CharacterId,
//         pub name: String,
//         pub levels: CharacterLevels,
//         pub hp: HitPoints,
//         pub ability_scores: AbilityScoreSet,
//         pub skills: SkillSet,
//         pub saving_throws: SavingThrowSet,
//         pub resistances: DamageResistances,
//         pub weapon_proficiencies: WeaponProficiencyMap,
//         pub loadout: Loadout,
//         pub spellbook: Spellbook,
//         pub resources: ResourceMap,
//         pub effects: Vec<Effect>,
//         pub actions: ActionMap,
//         pub cooldowns: HashMap<ActionId, RechargeRule>,
//     }
//     view CharacterView
//     view_mut CharacterViewMut
// }

#[derive(Bundle)]
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
