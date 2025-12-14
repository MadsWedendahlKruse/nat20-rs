use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    components::{
        ability::{Ability, AbilityScoreDistribution},
        dice::DieSize,
        id::{ActionId, ClassId, EffectId, IdProvider, ResourceId, SubclassId},
        items::equipment::{armor::ArmorType, weapon::WeaponCategory},
        level_up::{ChoiceItem, ChoiceSpec, LevelUpPrompt},
        modifier::ModifierSource,
        resource::ResourceBudgetKind,
        skill::Skill,
    },
    registry::{registry::SubclassesRegistry, serialize::class::ClassDefinition},
};

/// Classes and subclasses share a lot of common properties, so we define a base struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassBase {
    /// Skills that can be chosen from when gaining the (sub)class
    #[serde(default)]
    pub skill_proficiencies: HashSet<Skill>,
    /// The number of skill proficiencies the character can choose
    #[serde(default)]
    pub skill_prompts: u8,

    #[serde(default)]
    pub armor_proficiencies: HashSet<ArmorType>,
    #[serde(default)]
    pub weapon_proficiencies: HashSet<WeaponCategory>,

    /// Spellcasting progression, defining how many spells slots are available per level, if any.
    /// This is usually defined by the class, but certain subclasses might want to override it.
    /// For example, a subclass of a Fighter might gain some spellcasting abilities.
    #[serde(default)]
    pub spellcasting: SpellcastingProgression,
    /// Passive effects that are always active for the class or subclass.
    #[serde(default)]
    pub effects_by_level: HashMap<u8, Vec<EffectId>>,
    #[serde(default)]
    pub resources_by_level: HashMap<u8, Vec<(ResourceId, ResourceBudgetKind)>>,
    /// Class specific prompts that can be made at each level.
    /// For example, a Fighter might choose a fighting style at level 1.
    /// TODO: Include subclass prompts?
    #[serde(default)]
    pub prompts_by_level: HashMap<u8, Vec<LevelUpPrompt>>,
    /// Actions that are available at each level.
    #[serde(default)]
    pub actions_by_level: HashMap<u8, Vec<ActionId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl Default for SpellcastingProgression {
    fn default() -> Self {
        SpellcastingProgression::None
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ClassDefinition")]
pub struct Class {
    pub id: ClassId,
    pub hit_die: DieSize,
    pub hp_per_level: u8,

    pub default_abilities: AbilityScoreDistribution,

    /// Saving throw proficiencies granted at level 1 (e.g. STR + CON for Fighter)
    pub saving_throw_proficiencies: [Ability; 2],

    pub subclasses: HashSet<SubclassId>,

    /// The levels at which the class can pick a new feat.
    pub feat_levels: HashSet<u8>,

    pub base: ClassBase,
}

impl Class {
    pub fn new(
        id: ClassId,
        hit_die: DieSize,
        hp_per_level: u8,
        default_abilities: AbilityScoreDistribution,
        saving_throw_proficiencies: [Ability; 2],
        subclass_level: u8,
        subclasses: HashSet<SubclassId>,
        feat_levels: HashSet<u8>,
        skill_proficiencies: HashSet<Skill>,
        skill_prompts: u8,
        armor_proficiencies: HashSet<ArmorType>,
        weapon_proficiencies: HashSet<WeaponCategory>,
        spellcasting: SpellcastingProgression,
        effects_by_level: HashMap<u8, Vec<EffectId>>,
        resources_by_level: HashMap<u8, Vec<(ResourceId, ResourceBudgetKind)>>,
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
                ModifierSource::ClassFeature(id.clone()),
            ));

        // Add subclass prompt
        // NOTE: *DON'T* make a helper method in LevelUpPrompt for subclass prompts.
        // you've done it twice, and every time it creates a lookup in the class
        // registry while it's being initialized, so it just creates an infinite loop.
        prompts_by_level
            .entry(subclass_level)
            .or_default()
            .push(LevelUpPrompt::Choice(ChoiceSpec::single(
                "Subclass",
                subclasses
                    .clone()
                    .into_iter()
                    .map(ChoiceItem::Subclass)
                    .collect(),
            )));

        // TODO: What if the subclass triggers its own prompts?

        Self {
            id,
            hit_die,
            hp_per_level,
            default_abilities,
            saving_throw_proficiencies,
            subclasses,
            feat_levels,
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

    pub fn subclass(&self, subclass_id: &SubclassId) -> Option<&Subclass> {
        if !self.subclasses.contains(subclass_id) {
            return None;
        }
        SubclassesRegistry::get(subclass_id)
    }

    pub fn spellcasting_progression(
        &self,
        subclass_id: Option<&SubclassId>,
    ) -> SpellcastingProgression {
        if self.base.spellcasting != SpellcastingProgression::None {
            return self.base.spellcasting.clone();
        }
        if let Some(subclass) = subclass_id.and_then(|name| self.subclass(name)) {
            return subclass.base.spellcasting.clone();
        }
        SpellcastingProgression::None
    }

    pub fn base(&self) -> &ClassBase {
        &self.base
    }
}

impl IdProvider for Class {
    type Id = ClassId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subclass {
    pub id: SubclassId,
    pub base: ClassBase,
}

impl Subclass {
    pub fn base(&self) -> &ClassBase {
        &self.base
    }
}

impl IdProvider for Subclass {
    type Id = SubclassId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
