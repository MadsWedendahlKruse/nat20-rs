// TODO: Not sure if it's the best way to store this in a separate file, but character is already too big

use std::{
    collections::{HashSet, VecDeque},
    io::{self, Write},
};

use strum::IntoEnumIterator;

use crate::{
    creature::{character::Character, classes::class::SubclassName},
    registry::classes::CLASS_REGISTRY,
    stats::skill::Skill,
    test_utils::cli::CliChoiceProvider,
    utils::id::EffectId,
};

use super::classes::class::ClassName;

#[derive(Debug, Clone)]
pub enum LevelUpChoice {
    Class(Vec<ClassName>),
    Subclass(Vec<SubclassName>),
    Effect(Vec<EffectId>),
    SkillProficiency(HashSet<Skill>, u8),
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
        let subclasses = CLASS_REGISTRY
            .get(&class_name)
            .map_or_else(Vec::new, |class| class.subclasses.keys().cloned().collect());
        if subclasses.is_empty() {
            panic!("No subclasses found for class: {:?}", class_name);
        }
        LevelUpChoice::Subclass(subclasses)
    }
}

#[derive(Debug, Clone)]
pub enum LevelUpSelection {
    Class(ClassName),
    Subclass(SubclassName),
    Effect(EffectId),
    SkillProficiency(HashSet<Skill>),
    // Feat(FeatOption),
    // AbilityScoreImprovement(u8), // u8 = number of points to distribute
    // AbilityPoint(Ability),
    // Spell(SpellcastingClass, SpellOption),
    // etc.
}

impl LevelUpSelection {
    pub fn name(&self) -> &'static str {
        match self {
            LevelUpSelection::Class(_) => "Class",
            LevelUpSelection::Subclass(_) => "Subclass",
            LevelUpSelection::Effect(_) => "Effect",
            LevelUpSelection::SkillProficiency(_) => "SkillProficiency",
            // LevelUpSelection::Feat(_) => "Feat",
            // LevelUpSelection::AbilityScoreImprovement(_) => "AbilityScoreImprovement",
            // LevelUpSelection::AbilityPoint(_) => "AbilityPoint",
            // LevelUpSelection::Spell(_, _) => "Spell",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LevelUpError {
    InvalidSelection {
        choice: LevelUpChoice,
        selection: LevelUpSelection,
    },
    ChoiceSelectionMismatch {
        choice: LevelUpChoice,
        selection: LevelUpSelection,
    },
    RegistryMissing(String),
    // TODO: Add more error variants as needed
}
pub struct LevelUpSession<'a> {
    character: &'a mut Character,
    pending: VecDeque<LevelUpChoice>,
}

impl<'a> LevelUpSession<'a> {
    pub fn new(character: &'a mut Character) -> Self {
        let mut pending = VecDeque::new();
        pending.push_back(LevelUpChoice::class());
        LevelUpSession { character, pending }
    }

    pub fn advance(&mut self, provider: &mut impl ChoiceProvider) -> Result<(), LevelUpError> {
        while let Some(choice) = self.pending.pop_front() {
            let selection = provider.provide(&choice);
            let next = self.character.resolve_level_up_choice(choice, selection)?;
            for c in next {
                self.pending.push_back(c)
            }
        }
        self.character.apply_latest_level();
        Ok(())
    }
}
pub trait ChoiceProvider {
    fn provide(&mut self, choice: &LevelUpChoice) -> LevelUpSelection;
}

impl ChoiceProvider for CliChoiceProvider {
    fn provide(&mut self, choice: &LevelUpChoice) -> LevelUpSelection {
        match choice {
            LevelUpChoice::Class(classes) => {
                let idx = Self::select_from_list("Choose a class:", classes, |class| {
                    format!("{}", class)
                });
                LevelUpSelection::Class(classes[idx].clone())
            }

            LevelUpChoice::Subclass(subclasses) => {
                let idx = Self::select_from_list("Choose a subclass:", subclasses, |sub| {
                    format!("{} ({})", sub.name, sub.class)
                });
                LevelUpSelection::Subclass(subclasses[idx].clone())
            }

            LevelUpChoice::Effect(effects) => {
                let idx = Self::select_from_list("Choose an effect:", effects, |effect| {
                    format!("{}", effect)
                });
                LevelUpSelection::Effect(effects[idx].clone())
            }

            LevelUpChoice::SkillProficiency(skills, num_choices) => {
                let selected = Self::select_multiple(
                    &format!("Select {} skill(s) to gain proficiency in:", num_choices),
                    skills,
                    *num_choices,
                    |skill| format!("{:?}", skill),
                );
                LevelUpSelection::SkillProficiency(selected)
            }

            _ => {
                todo!("Implement CLI choice provider for other LevelUpChoice variants");
            }
        }
    }
}

/// A provider that hands out selections from a predefined list.
/// Useful for testing or when you want to simulate a specific sequence of choices.
pub struct PredefinedChoiceProvider {
    character: String,
    responses: VecDeque<LevelUpSelection>,
}

impl PredefinedChoiceProvider {
    pub fn new(character: String, responses: Vec<LevelUpSelection>) -> Self {
        Self {
            character,
            responses: responses.into(),
        }
    }
}

impl ChoiceProvider for PredefinedChoiceProvider {
    fn provide(&mut self, _choice: &LevelUpChoice) -> LevelUpSelection {
        self.responses.pop_front().expect(&format!(
            "Ran out of predefined responses when leveling up {}",
            self.character
        ))
    }
}
