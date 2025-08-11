use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::Ability,
        class::{ClassName, SubclassName},
        id::{BackgroundId, EffectId, FeatId, RaceId, SubraceId},
        modifier::ModifierSource,
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
    AbilityScores(HashMap<u8, u8>, u8),
    AbilityScoreImprovement {
        feat: FeatId,
        budget: u8,
        abilities: HashSet<Ability>,
        max_score: u8,
    },
    Background(Vec<BackgroundId>),
    Class(Vec<ClassName>),
    Effect(Vec<EffectId>),
    Feat(Vec<FeatId>),
    Race(Vec<RaceId>),
    SkillProficiency(HashSet<Skill>, u8, ModifierSource),
    Subclass(Vec<SubclassName>),
    Subrace(Vec<SubraceId>),
    // SpellSelection(SpellcastingClass, Vec<SpellOption>),
    // etc.
}

impl LevelUpPrompt {
    pub fn name(&self) -> &'static str {
        match self {
            LevelUpPrompt::AbilityScores(_, _) => "AbilityScores",
            LevelUpPrompt::AbilityScoreImprovement { .. } => "AbilityScoreImprovement",
            LevelUpPrompt::Background(_) => "Background",
            LevelUpPrompt::Class(_) => "Class",
            LevelUpPrompt::Effect(_) => "Effect",
            LevelUpPrompt::Feat(_) => "Feat",
            LevelUpPrompt::Race(_) => "Race",
            LevelUpPrompt::SkillProficiency(_, _, _) => "SkillProficiency",
            LevelUpPrompt::Subclass(_) => "Subclass",
            LevelUpPrompt::Subrace(_) => "Subrace",
        }
    }
}

impl LevelUpPrompt {
    pub fn ability_scores() -> Self {
        LevelUpPrompt::AbilityScores(ABILITY_SCORE_POINT_COST.clone(), ABILITY_SCORE_POINTS)
    }

    pub fn background() -> Self {
        LevelUpPrompt::Background(
            registry::backgrounds::BACKGROUND_REGISTRY
                .keys()
                .cloned()
                .collect(),
        )
    }

    pub fn class() -> Self {
        let classes = ClassName::iter().collect();
        LevelUpPrompt::Class(classes)
    }

    pub fn feats() -> Self {
        let mut feats: Vec<_> = registry::feats::FEAT_REGISTRY.keys().cloned().collect();
        // TODO: Bit of a dirty hack to remove fighting styles from the list of feats.
        feats.retain(|feat_id| !feat_id.to_string().starts_with("feat.fighting_style."));
        LevelUpPrompt::Feat(feats)
    }

    pub fn race() -> Self {
        LevelUpPrompt::Race(registry::races::RACE_REGISTRY.keys().cloned().collect())
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

    pub fn subrace(race: RaceId) -> Self {
        let subraces = registry::races::RACE_REGISTRY
            .get(&race)
            .map_or_else(Vec::new, |r| r.subraces.keys().cloned().collect());
        LevelUpPrompt::Subrace(subraces)
    }
}
