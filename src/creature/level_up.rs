// TODO: Not sure if it's the best way to store this in a separate file, but character is already too big

use std::{
    collections::VecDeque,
    io::{self, Write},
};

use strum::IntoEnumIterator;

use crate::{
    creature::{character::Character, classes::class::SubclassName},
    registry::classes::CLASS_REGISTRY,
};

use super::classes::class::ClassName;

#[derive(Debug, Clone)]
pub enum LevelUpChoice {
    Class(Vec<ClassName>),
    Subclass(Vec<SubclassName>),
    // FeatSelection(Vec<FeatOption>),
    // AbilityScoreImprovement(u8), // u8 = number of points to distribute
    // AbilityPointSelection(Vec<Ability>),
    // SpellSelection(SpellcastingClass, Vec<SpellOption>),
    // etc.
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
    // Feat(FeatOption),
    // AbilityScoreImprovement(u8), // u8 = number of points to distribute
    // AbilityPoint(Ability),
    // Spell(SpellcastingClass, SpellOption),
    // etc.
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
    RegistryMissing(ClassName),
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

    pub fn finished(&self) -> bool {
        // TODO: Might not be relevant since advance uses a while loop
        self.pending.is_empty()
    }
}

pub trait ChoiceProvider {
    fn provide(&mut self, choice: &LevelUpChoice) -> LevelUpSelection;
}

pub struct CliChoiceProvider;

impl ChoiceProvider for CliChoiceProvider {
    fn provide(&mut self, choice: &LevelUpChoice) -> LevelUpSelection {
        match choice {
            LevelUpChoice::Class(classes) => {
                println!("\nChoose a class:");
                for (i, class) in classes.iter().enumerate() {
                    println!("  [{:>2}] {}", i + 1, class);
                }
                let idx = Self::read_index(classes.len());
                LevelUpSelection::Class(classes[idx].clone())
            }

            LevelUpChoice::Subclass(subclasses) => {
                println!("\nChoose a subclass:");
                for (i, sub) in subclasses.iter().enumerate() {
                    println!("  [{:>2}] {} ({})", i + 1, sub.name, sub.class);
                }
                let idx = Self::read_index(subclasses.len());
                LevelUpSelection::Subclass(subclasses[idx].clone())
            } // …
        }
    }
}

impl CliChoiceProvider {
    fn read_index(max: usize) -> usize {
        loop {
            print!("Enter choice (1-{}): ", max);
            io::stdout().flush().unwrap();

            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                println!("Error reading input, try again.");
                continue;
            }

            match line.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= max => return n - 1,
                _ => println!("Invalid number, please enter between 1 and {}.", max),
            }
        }
    }
}

/// A provider that hands out selections from a predefined list.
/// Useful for testing or when you want to simulate a specific sequence of choices.
pub struct PredefinedChoiceProvider {
    responses: VecDeque<LevelUpSelection>,
}

impl PredefinedChoiceProvider {
    pub fn new(responses: Vec<LevelUpSelection>) -> Self {
        Self {
            responses: responses.into(),
        }
    }
}

impl ChoiceProvider for PredefinedChoiceProvider {
    fn provide(&mut self, _choice: &LevelUpChoice) -> LevelUpSelection {
        // We have to mutate, so wrap in RefCell or make `provide` take &mut self:
        self.responses
            .pop_front()
            .expect("ran out of mock responses")
    }
}
