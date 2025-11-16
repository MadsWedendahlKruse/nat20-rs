use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::LazyLock,
};

use crate::{
    components::{
        ability::Ability,
        id::{
            ActionId, BackgroundId, ClassId, EffectId, FeatId, ItemId, RaceId, SubclassId,
            SubraceId,
        },
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChoiceItem {
    Action(ActionId),
    Background(BackgroundId),
    Class(ClassId),
    Subclass(SubclassId),
    Effect(EffectId),
    Feat(FeatId),
    Race(RaceId),
    Subrace(SubraceId),
    Equipment {
        items: Vec<(u8, ItemId)>,
        money: String, // e.g., "10 GP"
    }, // SubPrompt(Box<LevelUpPrompt>), // cascade
       // Escape hatch if you need something truly custom
       // Custom(String),
}

impl ChoiceItem {
    pub fn id(&self) -> &'static str {
        match self {
            ChoiceItem::Action(_) => "choice.action",
            ChoiceItem::Background(_) => "choice.background",
            ChoiceItem::Class(_) => "choice.class",
            ChoiceItem::Subclass(_) => "choice.subclass",
            ChoiceItem::Effect(_) => "choice.effect",
            ChoiceItem::Feat(_) => "choice.feat",
            ChoiceItem::Race(_) => "choice.race",
            ChoiceItem::Subrace(_) => "choice.subrace",
            ChoiceItem::Equipment { .. } => "choice.equipment",
        }
    }

    /// Primarily used for visualization purposes.
    pub fn priority(&self) -> u8 {
        match self {
            ChoiceItem::Race(_) => 0,
            ChoiceItem::Subrace(_) => 1,
            ChoiceItem::Background(_) => 2,
            ChoiceItem::Class(_) => 3,
            ChoiceItem::Subclass(_) => 4,
            ChoiceItem::Equipment { .. } => 5,
            ChoiceItem::Action(_) => 6,
            ChoiceItem::Effect(_) => 7,
            ChoiceItem::Feat(_) => 8,
        }
    }
}

impl std::fmt::Display for ChoiceItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChoiceItem::Effect(id) => write!(f, "{}", id),
            ChoiceItem::Feat(id) => write!(f, "{}", id),
            ChoiceItem::Action(id) => write!(f, "{}", id),
            ChoiceItem::Background(id) => write!(f, "{}", id),
            ChoiceItem::Class(id) => write!(f, "{}", id),
            ChoiceItem::Subclass(id) => write!(f, "{}", id),
            ChoiceItem::Race(id) => write!(f, "{}", id),
            ChoiceItem::Subrace(id) => write!(f, "{}", id),
            ChoiceItem::Equipment { items, money } => {
                let mut lines: Vec<String> = items
                    .iter()
                    .map(|(count, id)| format!("{} x {}", count, id.to_string()))
                    .collect();
                if !money.is_empty() {
                    lines.push(money.to_string());
                }
                write!(f, "{}", lines.join("\n"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceSpec {
    pub id: String,
    pub label: String,
    pub options: Vec<ChoiceItem>,
    pub picks: u8,
    pub allow_duplicates: bool,
}

impl ChoiceSpec {
    pub fn single(label: impl Into<String>, options: Vec<ChoiceItem>) -> Self {
        if options.is_empty() {
            panic!("ChoiceSpec must have at least one option");
        }

        Self {
            // Assuming all the options have the same type we can just infer the
            // id from the first option.
            id: options.first().map(|item| item.id().to_string()).unwrap(),
            label: label.into(),
            options,
            picks: 1,
            allow_duplicates: false,
        }
    }

    pub fn with_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.id = id.into();
        self
    }

    pub fn priority(&self) -> u8 {
        self.options
            .iter()
            .map(ChoiceItem::priority)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelUpPrompt {
    Choice(ChoiceSpec),
    AbilityScores(HashMap<u8, u8>, u8),
    AbilityScoreImprovement {
        feat: FeatId,
        budget: u8,
        abilities: HashSet<Ability>,
        max_score: u8,
    },
    SkillProficiency(HashSet<Skill>, u8, ModifierSource),
    // SpellSelection(SpellcastingClass, Vec<SpellOption>),
    // etc.
}

impl LevelUpPrompt {
    pub fn priority(&self) -> u8 {
        match self {
            LevelUpPrompt::Choice(spec) => spec.priority(),
            LevelUpPrompt::AbilityScores(_, _) => 4,
            LevelUpPrompt::SkillProficiency(_, _, _) => 5,
            LevelUpPrompt::AbilityScoreImprovement { .. } => 8,
        }
    }

    pub fn ability_scores() -> Self {
        LevelUpPrompt::AbilityScores(ABILITY_SCORE_POINT_COST.clone(), ABILITY_SCORE_POINTS)
    }

    pub fn background() -> Self {
        LevelUpPrompt::Choice(ChoiceSpec::single(
            "Background",
            registry::backgrounds::BACKGROUND_REGISTRY
                .keys()
                .cloned()
                .map(ChoiceItem::Background)
                .collect(),
        ))
    }

    pub fn class() -> Self {
        LevelUpPrompt::Choice(ChoiceSpec::single(
            "Class",
            registry::classes::CLASS_REGISTRY
                .keys()
                .cloned()
                .map(ChoiceItem::Class)
                .collect(),
        ))
    }

    pub fn feats() -> Self {
        let mut feats: Vec<_> = registry::feats::FEAT_REGISTRY.keys().cloned().collect();
        // TODO: Bit of a dirty hack to remove fighting styles from the list of feats.
        feats.retain(|feat_id| !feat_id.to_string().starts_with("feat.fighting_style."));
        LevelUpPrompt::Choice(ChoiceSpec::single(
            "Feat",
            feats.into_iter().map(ChoiceItem::Feat).collect(),
        ))
    }

    pub fn race() -> Self {
        LevelUpPrompt::Choice(ChoiceSpec::single(
            "Race",
            registry::races::RACE_REGISTRY
                .keys()
                .cloned()
                .map(ChoiceItem::Race)
                .collect(),
        ))
    }

    pub fn subrace(race: RaceId) -> Self {
        let subraces = registry::races::RACE_REGISTRY
            .get(&race)
            .map_or_else(Vec::new, |r| r.subraces.keys().cloned().collect());
        LevelUpPrompt::Choice(ChoiceSpec::single(
            "Subrace",
            subraces.into_iter().map(ChoiceItem::Subrace).collect(),
        ))
    }
}

impl Display for LevelUpPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelUpPrompt::Choice(spec) => write!(f, "{}", spec.label),
            LevelUpPrompt::AbilityScores(_, _) => write!(f, "Ability Scores"),
            LevelUpPrompt::AbilityScoreImprovement { .. } => {
                write!(f, "Ability Score Improvement")
            }
            LevelUpPrompt::SkillProficiency(_, _, _) => {
                write!(f, "Skill Proficiency")
            }
        }
    }
}
