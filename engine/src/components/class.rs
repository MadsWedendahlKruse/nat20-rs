use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use strum::EnumIter;

use crate::components::{
    ability::{Ability, AbilityScoreDistribution},
    dice::DieSize,
    id::{ActionId, EffectId},
    items::equipment::{armor::ArmorType, weapon::WeaponCategory},
    level_up::LevelUpPrompt,
    resource::Resource,
    skill::Skill,
};

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
    pub skill_prompts: u8,

    pub armor_proficiencies: HashSet<ArmorType>,
    pub weapon_proficiencies: HashSet<WeaponCategory>,

    /// Spellcasting progression, defining how many spells slots are available per level, if any.
    /// This is usually defined by the class, but certain subclasses might want to override it.
    /// For example, a subclass of a Fighter might gain some spellcasting abilities.
    pub spellcasting: SpellcastingProgression,
    // TODO
    // pub features_by_level: HashMap<u8, Vec<ClassFeature>>,
    /// Passive effects that are always active for the class or subclass.
    pub effects_by_level: HashMap<u8, Vec<EffectId>>,
    pub resources_by_level: HashMap<u8, Vec<Resource>>,
    /// Class specific prompts that can be made at each level.
    /// For example, a Fighter might choose a fighting style at level 1.
    /// TODO: Include subclass prompts?
    pub prompts_by_level: HashMap<u8, Vec<LevelUpPrompt>>,
    /// Actions that are available at each level.
    pub actions_by_level: HashMap<u8, Vec<ActionId>>,
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

    pub default_abilities: AbilityScoreDistribution,

    /// Saving throw proficiencies granted at level 1 (e.g. STR + CON for Fighter)
    pub saving_throw_proficiencies: [Ability; 2],

    pub subclasses: HashMap<SubclassName, Subclass>,

    pub base: ClassBase,
}

impl Class {
    pub fn new(
        name: ClassName,
        hit_die: DieSize,
        hp_per_level: u8,
        default_abilities: AbilityScoreDistribution,
        saving_throw_proficiencies: [Ability; 2],
        subclass_level: u8,
        subclasses: HashMap<SubclassName, Subclass>,
        feat_levels: HashSet<u8>,
        skill_proficiencies: HashSet<Skill>,
        skill_prompts: u8,
        armor_proficiencies: HashSet<ArmorType>,
        weapon_proficiencies: HashSet<WeaponCategory>,
        spellcasting: SpellcastingProgression,
        effects_by_level: HashMap<u8, Vec<EffectId>>,
        resources_by_level: HashMap<u8, Vec<Resource>>,
        mut prompts_by_level: HashMap<u8, Vec<LevelUpPrompt>>,
        actions_by_level: HashMap<u8, Vec<ActionId>>,
    ) -> Self {
        // Add skill proficiencies
        prompts_by_level
            .entry(1)
            .or_default()
            .push(LevelUpPrompt::SkillProficiency(
                skill_proficiencies.clone(),
                skill_prompts,
            ));

        // Add subclass prompts
        prompts_by_level
            .entry(subclass_level)
            .or_default()
            .push(LevelUpPrompt::Subclass(
                subclasses.keys().cloned().collect(),
            ));

        // Add feat decisions
        for level in feat_levels.iter() {
            prompts_by_level
                .entry(*level)
                .or_default()
                // TODO: Don't use *all* feats in the future
                .push(LevelUpPrompt::feats());
        }

        Self {
            name,
            hit_die,
            hp_per_level,
            default_abilities,
            saving_throw_proficiencies,
            subclasses,
            base: ClassBase {
                skill_proficiencies,
                skill_prompts,
                armor_proficiencies,
                weapon_proficiencies,
                spellcasting,
                effects_by_level,
                resources_by_level,
                prompts_by_level,
                actions_by_level,
            },
        }
    }

    pub fn subclass(&self, subclass_name: &SubclassName) -> Option<&Subclass> {
        self.subclasses.get(subclass_name)
    }

    pub fn spellcasting_progression(
        &self,
        subclass_name: Option<&SubclassName>,
    ) -> SpellcastingProgression {
        if self.base.spellcasting != SpellcastingProgression::None {
            return self.base.spellcasting.clone();
        }
        if let Some(subclass) = subclass_name.and_then(|name| self.subclass(name)) {
            return subclass.base.spellcasting.clone();
        }
        SpellcastingProgression::None
    }

    pub fn effects_by_level(&self, level: u8, subclass_name: &SubclassName) -> Vec<EffectId> {
        let subclass_map = self
            .subclass(subclass_name)
            .map(|subclass| &subclass.base.effects_by_level);
        self.merge_by_level(level, &self.base.effects_by_level, subclass_map)
    }

    pub fn resources_by_level(&self, level: u8, subclass_name: &SubclassName) -> Vec<Resource> {
        let subclass_map = self
            .subclass(subclass_name)
            .map(|subclass| &subclass.base.resources_by_level);
        self.merge_by_level(level, &self.base.resources_by_level, subclass_map)
    }

    pub fn actions_by_level(&self, level: u8, subclass_name: &SubclassName) -> Vec<ActionId> {
        let subclass_map = self
            .subclass(subclass_name)
            .map(|subclass| &subclass.base.actions_by_level);
        self.merge_by_level(level, &self.base.actions_by_level, subclass_map)
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

    pub fn base(&self) -> &ClassBase {
        &self.base
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

impl Subclass {
    pub fn base(&self) -> &ClassBase {
        &self.base
    }
}
