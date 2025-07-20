use std::collections::HashMap;

use hecs::Bundle;

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
};

#[derive(Bundle)]
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub levels: CharacterLevels,
    pub latest_class: Option<ClassName>, // The class that was most recently leveled up
    pub hp: HitPoints,
    pub ability_scores: AbilityScoreSet,
    pub skills: SkillSet,
    pub saving_throws: SavingThrowSet,
    pub resistances: DamageResistances,
    // TODO: Might have to make this more granular later (not just martial/simple)
    // TODO: Should it just be a bool (or a set even)? Not sure if you can have expertise in a weapon
    pub weapon_proficiencies: WeaponProficiencyMap,
    /// Equipped items
    pub loadout: Loadout,
    pub spellbook: Spellbook,
    pub resources: ResourceMap,
    pub effects: Vec<Effect>,
    pub actions: ActionMap,
    /// Actions that are currently on cooldown
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
            id: CharacterId::new_v4(),
            name: name.to_string(),
            levels: CharacterLevels::new(),
            latest_class: None,
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
