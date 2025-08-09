use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::Ability,
        class::{ClassName, SubclassName},
        id::{EffectId, FeatId},
        skill::Skill,
    },
    registry,
};

static ABILITY_SCORE_POINT_COST: LazyLock<HashMap<u8, u8>> = LazyLock::new(|| {
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

static ABILITY_SCORE_POINTS: u8 = 27;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelUpPrompt {
    Class(Vec<ClassName>),
    Subclass(Vec<SubclassName>),
    Effect(Vec<EffectId>),
    SkillProficiency(HashSet<Skill>, u8),
    AbilityScores(HashMap<u8, u8>, u8),
    Feat(Vec<FeatId>),
    AbilityScoreImprovement {
        // TODO: Does it ever *not* come from a feat?
        feat: Option<FeatId>,
        budget: u8,
        abilities: HashSet<Ability>,
        max_score: u8,
    },
    // SpellSelection(SpellcastingClass, Vec<SpellOption>),
    // etc.
}

impl LevelUpPrompt {
    pub fn name(&self) -> &'static str {
        match self {
            LevelUpPrompt::Class(_) => "Class",
            LevelUpPrompt::Subclass(_) => "Subclass",
            LevelUpPrompt::Effect(_) => "Effect",
            LevelUpPrompt::SkillProficiency(_, _) => "SkillProficiency",
            LevelUpPrompt::AbilityScores(_, _) => "AbilityScores",
            LevelUpPrompt::Feat(_) => "Feat",
            LevelUpPrompt::AbilityScoreImprovement { .. } => "AbilityScoreImprovement",
        }
    }
}

impl LevelUpPrompt {
    pub fn class() -> Self {
        let classes = ClassName::iter().collect();
        LevelUpPrompt::Class(classes)
    }

    pub fn subclass(class_name: ClassName) -> Self {
        let subclasses = registry::classes::CLASS_REGISTRY
            .get(&class_name)
            .map_or_else(Vec::new, |class| class.subclasses.keys().cloned().collect());
        if subclasses.is_empty() {
            panic!("No subclasses found for class: {:?}", class_name);
        }
        LevelUpPrompt::Subclass(subclasses)
    }

    pub fn ability_scores() -> Self {
        LevelUpPrompt::AbilityScores(ABILITY_SCORE_POINT_COST.clone(), ABILITY_SCORE_POINTS)
    }

    pub fn feats() -> Self {
        let feats = registry::feats::FEAT_REGISTRY.keys().cloned().collect();
        LevelUpPrompt::Feat(feats)
    }
}
