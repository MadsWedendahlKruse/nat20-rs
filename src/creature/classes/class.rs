use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use strum::EnumIter;

use crate::{
    creature::level_up::LevelUpChoice,
    dice::dice::DieSize,
    effects::effects::Effect,
    items::equipment::{armor::ArmorType, weapon::WeaponCategory},
    stats::{ability::Ability, skill::Skill},
};

use super::resources::Resource;

// TODO: Better name
// TODO: Classes are an enum, but subclasses are just a string?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ClassName {
    Fighter,
    Wizard,
    Rogue,
    Cleric,
    Druid,
    Paladin,
    Ranger,
    Bard,
    Sorcerer,
    Warlock,
    Monk,
    Barbarian,
}

impl fmt::Display for ClassName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Classes and subclasses share a lot of common properties, so we define a base struct
// TODO: Better name
#[derive(Debug, Clone)]
pub struct ClassBase {
    /// Skills that can be chosen from when gaining the (sub)class
    pub skill_proficiencies: HashSet<Skill>,
    /// The number of skill proficiencies the character can choose
    pub skill_choices: u8,

    pub armor_proficiencies: HashSet<ArmorType>,
    pub weapon_proficiencies: HashSet<WeaponCategory>,

    /// Spellcasting progression, defining how many spells slots are available per level, if any.
    /// This is usually defined by the class, but certain subclasses might want to override it.
    /// For example, a subclass of a Fighter might gain some spellcasting abilities.
    pub spellcasting: SpellcastingProgression,
    // TODO
    // pub features_by_level: HashMap<u8, Vec<ClassFeature>>,
    /// Passive effects that are always active for the class or subclass.
    pub effects_by_level: HashMap<u8, Vec<Effect>>,
    pub resources_by_level: HashMap<u8, Vec<Resource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellcastingProgression {
    /// Full spellcasting progression, e.g. Wizard.
    Full,
    /// Half spellcasting progression, e.g. Cleric.
    Half,
    /// Third spellcasting progression, e.g. Bard.
    Third,
    /// No spellcasting progression, e.g. Fighter.
    None,
}

#[derive(Debug, Clone)]
pub struct Class {
    // TODO: Can also be a string
    pub name: ClassName,
    pub hit_die: DieSize,
    pub hp_per_level: u8,

    /// Saving throw proficiencies granted at level 1 (e.g. STR + CON for Fighter)
    pub saving_throw_proficiencies: [Ability; 2],

    /// At which level the character picks a subclass (typically level 3).
    pub subclass_level: u8,
    pub subclasses: HashMap<SubclassName, Subclass>,

    pub feat_levels: HashSet<u8>,

    pub base: ClassBase,
}

impl Class {
    pub fn subclass(&self, subclass_name: &str) -> Option<&Subclass> {
        self.subclasses.get(&SubclassName {
            class: self.name,
            name: subclass_name.to_string(),
        })
    }

    pub fn spellcasting_progression(&self, subclass_name: &str) -> SpellcastingProgression {
        if self.base.spellcasting != SpellcastingProgression::None {
            return self.base.spellcasting.clone();
        }
        if let Some(subclass) = self.subclass(subclass_name) {
            return subclass.base.spellcasting.clone();
        }
        SpellcastingProgression::None
    }

    pub fn effects_by_level(&self, level: u8, subclass_name: &str) -> Vec<Effect> {
        let subclass_map = self
            .subclass(subclass_name)
            .map(|subclass| &subclass.base.effects_by_level);
        self.merge_by_level(level, &self.base.effects_by_level, subclass_map)
    }

    pub fn resources_by_level(&self, level: u8, subclass_name: &str) -> Vec<Resource> {
        let subclass_map = self
            .subclass(subclass_name)
            .map(|subclass| &subclass.base.resources_by_level);
        self.merge_by_level(level, &self.base.resources_by_level, subclass_map)
    }

    fn merge_by_level<T: Clone>(
        &self,
        level: u8,
        base_map: &HashMap<u8, Vec<T>>,
        subclass_map: Option<&HashMap<u8, Vec<T>>>,
    ) -> Vec<T> {
        let mut items = base_map.get(&level).cloned().unwrap_or_default();
        if let Some(subclass_map) = subclass_map {
            if let Some(subclass_items) = subclass_map.get(&level) {
                items.extend(subclass_items.clone());
            }
        }
        items
    }

    pub fn level_up_choices(&self, level: u8) -> Vec<LevelUpChoice> {
        let mut choices = Vec::new();

        if level == self.subclass_level {
            choices.push(LevelUpChoice::subclass(self.name));
        }

        if self.feat_levels.contains(&level) {
            // choices.push(LevelUpChoice::feat_selection());
        }

        // TODO: Pick new spells etc.

        choices
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubclassName {
    /// Validation logic becomes easier if the subclass knows what class it belongs to
    pub class: ClassName,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Subclass {
    pub name: SubclassName,
    pub base: ClassBase,
}
