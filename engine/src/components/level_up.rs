use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use strum::IntoEnumIterator;

use crate::{
    components::{
        class::{ClassName, SubclassName},
        id::EffectId,
        skill::Skill,
    },
    registry,
};

pub static ABILITY_SCORE_POINT_COST: LazyLock<HashMap<u8, u8>> = LazyLock::new(|| {
    HashMap::from([
        (8, 0),
        (9, 1),
        (10, 2),
        (11, 3),
        (12, 4),
        (13, 5),
        (14, 7),
        (15, 9),
    ])
});

pub static ABILITY_SCORE_POINTS: u8 = 27;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelUpChoice {
    Class(Vec<ClassName>),
    Subclass(Vec<SubclassName>),
    Effect(Vec<EffectId>),
    SkillProficiency(HashSet<Skill>, u8),
    AbilityScores(HashMap<u8, u8>, u8),
    // FeatSelection(Vec<FeatOption>),
    // AbilityScoreImprovement(u8), // u8 = number of points to distribute
    // AbilityPointSelection(Vec<Ability>),
    // SpellSelection(SpellcastingClass, Vec<SpellOption>),
    // etc.
}

impl LevelUpChoice {
    pub fn name(&self) -> &'static str {
        match self {
            LevelUpChoice::Class(_) => "Class",
            LevelUpChoice::Subclass(_) => "Subclass",
            LevelUpChoice::Effect(_) => "Effect",
            LevelUpChoice::SkillProficiency(_, _) => "SkillProficiency",
            LevelUpChoice::AbilityScores(_, _) => "AbilityScores",
            // LevelUpChoice::FeatSelection(_) => "FeatSelection",
            // LevelUpChoice::AbilityScoreImprovement(_) => "AbilityScoreImprovement",
            // LevelUpChoice::AbilityPointSelection(_) => "AbilityPointSelection",
            // LevelUpChoice::SpellSelection(_, _) => "SpellSelection",
        }
    }
}

impl LevelUpChoice {
    pub fn class() -> Self {
        let classes = ClassName::iter().collect();
        LevelUpChoice::Class(classes)
    }

    pub fn subclass(class_name: ClassName) -> Self {
        let subclasses = registry::classes::CLASS_REGISTRY
            .get(&class_name)
            .map_or_else(Vec::new, |class| class.subclasses.keys().cloned().collect());
        if subclasses.is_empty() {
            panic!("No subclasses found for class: {:?}", class_name);
        }
        LevelUpChoice::Subclass(subclasses)
    }

    pub fn ability_scores() -> Self {
        LevelUpChoice::AbilityScores(ABILITY_SCORE_POINT_COST.clone(), ABILITY_SCORE_POINTS)
    }
}
