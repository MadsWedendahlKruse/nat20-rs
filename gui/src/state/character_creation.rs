use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use imgui::{Condition, TableFlags, TreeNodeFlags};
use nat20_rs::{
    components::{
        ability::Ability,
        class::{ClassName, SubclassName},
        id::EffectId,
        level_up::LevelUpChoice,
        skill::Skill,
    },
    entities::character::{Character, CharacterTag},
    registry,
    systems::{
        self,
        level_up::{LevelUpSelection, LevelUpSession},
    },
    test_utils::fixtures,
};
use strum::IntoEnumIterator;

use crate::{
    render::imgui_render::{ImguiRenderableMut, ImguiRenderableMutWithContext, table_column},
    table_columns,
};

#[derive(Debug, Clone)]
enum LevelUpSelectionProgress {
    Class(Option<ClassName>),
    Subclass(Option<SubclassName>),
    Effect(Option<EffectId>),
    SkillProficiency {
        selected: HashSet<Skill>,
        remaining_selections: u8,
    },
    AbilityScores {
        assignments: HashMap<Ability, u8>,
        remaining_budget: u8,
        plus_2_bonus: Option<Ability>,
        plus_1_bonus: Option<Ability>,
    },
    // Add more as needed
}

impl LevelUpSelectionProgress {
    // TODO: Might need more complex validation
    fn is_complete(&self) -> bool {
        match self {
            LevelUpSelectionProgress::Class(class) => class.is_some(),
            LevelUpSelectionProgress::Subclass(subclass) => subclass.is_some(),
            LevelUpSelectionProgress::Effect(effect) => effect.is_some(),
            LevelUpSelectionProgress::SkillProficiency {
                selected,
                remaining_selections,
            } => remaining_selections == &0 && selected.len() > 0,
            LevelUpSelectionProgress::AbilityScores {
                assignments,
                remaining_budget,
                plus_2_bonus,
                plus_1_bonus,
            } => {
                assignments.len() == Ability::iter().count()
                    && remaining_budget == &0
                    && plus_2_bonus.is_some()
                    && plus_1_bonus.is_some()
            }
        }
    }

    fn finalize(self) -> LevelUpSelection {
        match self {
            LevelUpSelectionProgress::Class(class) => LevelUpSelection::Class(class.unwrap()),
            LevelUpSelectionProgress::Subclass(subclass) => {
                LevelUpSelection::Subclass(subclass.unwrap())
            }
            LevelUpSelectionProgress::Effect(effect) => LevelUpSelection::Effect(effect.unwrap()),
            LevelUpSelectionProgress::SkillProficiency { selected, .. } => {
                LevelUpSelection::SkillProficiency(selected)
            }
            LevelUpSelectionProgress::AbilityScores {
                assignments,
                plus_2_bonus,
                plus_1_bonus,
                ..
            } => LevelUpSelection::AbilityScores {
                scores: assignments,
                plus_2_bonus: plus_2_bonus.unwrap(),
                plus_1_bonus: plus_1_bonus.unwrap(),
            },
        }
    }

    fn from_choice(choice: &LevelUpChoice) -> Self {
        match choice {
            LevelUpChoice::Class(_) => LevelUpSelectionProgress::Class(None),
            LevelUpChoice::Subclass(_) => LevelUpSelectionProgress::Subclass(None),
            LevelUpChoice::Effect(_) => LevelUpSelectionProgress::Effect(None),
            LevelUpChoice::SkillProficiency(_, required) => {
                LevelUpSelectionProgress::SkillProficiency {
                    selected: HashSet::new(),
                    remaining_selections: *required,
                }
            }
            LevelUpChoice::AbilityScores(_, budget) => LevelUpSelectionProgress::AbilityScores {
                assignments: HashMap::new(),
                remaining_budget: *budget,
                plus_2_bonus: None,
                plus_1_bonus: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct LevelUpChoiceWithProgress {
    choice: LevelUpChoice,
    progress: LevelUpSelectionProgress,
}

impl LevelUpChoiceWithProgress {
    fn new(choice: LevelUpChoice) -> Self {
        let progress = LevelUpSelectionProgress::from_choice(&choice);
        Self { choice, progress }
    }
}

#[derive(Debug, PartialEq)]
pub enum CharacterCreationState {
    ChoosingMethod,
    FromPredefined,
    FromScratch,
}

pub struct CharacterCreation {
    /// World where the character creation takes place. Note that this is not the
    /// main game world, but rather a dummy world. Once the character is created,
    /// it will be added to the main game world.
    world: World,
    pub state: Option<CharacterCreationState>,
    current_character: Option<Entity>,
    level_up_session: Option<LevelUpSession>,
    pending_selections: Vec<LevelUpChoiceWithProgress>,
    creation_complete: bool,
}

impl CharacterCreation {
    pub fn new() -> Self {
        let mut world = World::new();

        let spawners = vec![
            fixtures::creatures::heroes::fighter,
            fixtures::creatures::heroes::wizard,
            fixtures::creatures::heroes::warlock,
            fixtures::creatures::monsters::goblin_warrior,
        ];

        for spawner in spawners {
            let (entity, _) = spawner(&mut world);
            // They spawn with zero health, so we heal them to full
            systems::health::heal_full(&mut world, entity);
        }

        Self {
            world,
            state: None,
            current_character: None,
            level_up_session: None,
            pending_selections: Vec::new(),
            creation_complete: false,
        }
    }

    fn reset(&mut self) {
        self.state = None;
        self.current_character = None;
        self.level_up_session = None;
        self.pending_selections.clear();
        self.creation_complete = false;
    }

    fn sync_pending_selections(&mut self) {
        let pending_choices = self.level_up_session.as_ref().unwrap().pending_choices();

        for choice in pending_choices {
            let already_present = self.pending_selections.iter().any(|p| p.choice == *choice);
            if !already_present {
                self.pending_selections
                    .push(LevelUpChoiceWithProgress::new(choice.clone()));
            }
        }
    }

    pub fn creation_complete(&self) -> bool {
        self.creation_complete
    }

    pub fn get_character(&mut self) -> Option<Character> {
        if let Some(entity) = self.current_character {
            self.reset();
            Some(Character::from_world(&self.world, entity))
        } else {
            None
        }
    }
}

impl ImguiRenderableMut for CharacterCreation {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        if self.state.is_none() {
            return;
        }
        ui.window("Character Creation")
            .size([500.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                match self.state {
                    Some(CharacterCreationState::ChoosingMethod) => {
                        if ui.button("From Predefined") {
                            self.state = Some(CharacterCreationState::FromPredefined);
                        }
                        if ui.button("From Scratch") {
                            self.state = Some(CharacterCreationState::FromScratch);
                        }
                        if ui.button("Cancel") {
                            self.state = None;
                        }
                    }

                    Some(CharacterCreationState::FromPredefined) => {
                        let mut characters = Vec::new();
                        for (entity, (name, tag)) in
                            self.world.query_mut::<(&String, &CharacterTag)>()
                        {
                            characters.push((entity, name.clone(), tag.clone()));
                        }
                        for (entity, name, tag) in characters {
                            if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                                if ui.button(format!("Add to World##{}", entity.id())) {
                                    self.current_character = Some(entity);
                                    self.creation_complete = true;
                                }
                                ui.separator();
                                (&mut self.world, entity, tag).render_mut(ui);
                            }
                        }
                        ui.separator();
                        if ui.button("Back") {
                            self.state = Some(CharacterCreationState::ChoosingMethod);
                        }
                    }

                    Some(CharacterCreationState::FromScratch) => {
                        // Logic for scratch character creation
                        if ui.button("Back") {
                            self.state = Some(CharacterCreationState::ChoosingMethod);
                            self.current_character = None;
                            self.level_up_session = None;
                        }
                        ui.separator();

                        if self.current_character.is_none() {
                            let character = Character::new("Johnny Hero");
                            self.current_character = Some(self.world.spawn(character));
                        }

                        if self.level_up_session.is_none() {
                            println!("Starting level up session for character");
                            self.level_up_session = Some(LevelUpSession::new(
                                &self.world,
                                self.current_character.unwrap(),
                            ));
                        }

                        let tag = systems::helpers::get_component_clone::<CharacterTag>(
                            &self.world,
                            self.current_character.unwrap(),
                        );

                        let mut name = systems::helpers::get_component_clone::<String>(
                            &self.world,
                            self.current_character.unwrap(),
                        );
                        ui.text("Name:");
                        if ui
                            .input_text("##", &mut name)
                            .enter_returns_true(true)
                            .build()
                        {
                            // User pressed Enter, maybe commit name change here
                            systems::helpers::set_component(
                                &mut self.world,
                                self.current_character.unwrap(),
                                name,
                            );
                        }

                        ui.separator();

                        let mut choice_selected = false;
                        // TODO: Not terribly efficient, but works for now
                        let choice_clones = self.pending_selections.clone();
                        for pending_selection in &mut self.pending_selections {
                            // ui.separator_with_text(format!("{}", pending_selection.choice.name()));
                            if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs")) {
                                if let Some(tab) =
                                    ui.tab_item(format!("{}", pending_selection.choice.name()))
                                {
                                    pending_selection.render_mut_with_context(ui, &choice_clones);
                                    tab.end();
                                }

                                tab_bar.end();

                                if pending_selection.progress.is_complete() {
                                    let finalized_selection =
                                        pending_selection.progress.clone().finalize();
                                    let result = self
                                        .level_up_session
                                        .as_mut()
                                        .unwrap()
                                        .advance(&mut self.world, &finalized_selection);
                                    if result.is_ok() {
                                        choice_selected = true;
                                    }
                                }
                            }
                        }

                        // Check if any new pending choices were triggered
                        // Or if there are no pending selections to choose from
                        if choice_selected || self.pending_selections.is_empty() {
                            self.sync_pending_selections();
                        }

                        ui.separator();

                        for (i, choice) in self
                            .level_up_session
                            .as_ref()
                            .unwrap()
                            .pending_choices()
                            .iter()
                            .enumerate()
                        {
                            ui.text(format!("Choice {}: {}", i + 1, choice.name()));
                        }

                        if self.level_up_session.as_ref().unwrap().is_complete() {
                            if ui.button("Finish Character Creation") {
                                self.creation_complete = true;
                            }
                        }
                    }
                    None => {}
                }
            });
    }
}

impl ImguiRenderableMutWithContext<Vec<LevelUpChoiceWithProgress>> for LevelUpChoiceWithProgress {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        context: &Vec<LevelUpChoiceWithProgress>,
    ) {
        match self.choice {
            LevelUpChoice::Class(ref classes) => {
                if let LevelUpSelectionProgress::Class(ref mut selection) = self.progress {
                    let button_size = [100.0, 30.0];
                    let buttons_per_row = 4;

                    for (i, class) in classes.iter().enumerate() {
                        let is_selected = selection.is_some_and(|s| s == *class);

                        let style_color = if is_selected {
                            ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.6, 0.2, 1.0])
                        } else {
                            ui.push_style_color(
                                imgui::StyleColor::Button,
                                ui.style_color(imgui::StyleColor::Button),
                            )
                        };

                        if ui.button_with_size(format!("{}", class), button_size) {
                            *selection = Some(class.clone());
                        }

                        style_color.pop();

                        if (i + 1) % buttons_per_row != 0 {
                            ui.same_line();
                        }
                    }
                } else {
                    ui.text("Mismatched progress type for Class choice");
                }
            }

            LevelUpChoice::AbilityScores(ref scores_cost, ref point_budget) => {
                if let LevelUpSelectionProgress::AbilityScores {
                    ref mut assignments,
                    ref mut remaining_budget,
                    ref mut plus_2_bonus,
                    ref mut plus_1_bonus,
                } = self.progress
                {
                    if assignments.is_empty() {
                        for ability in Ability::iter() {
                            assignments.insert(ability, 8);
                        }
                    }

                    ui.text(format!("Remaining Budget: {}", remaining_budget));

                    // Reset button
                    if ui.button("Reset##Abilities") {
                        for ability in Ability::iter() {
                            assignments.insert(ability, 8);
                        }
                        *remaining_budget = *point_budget;
                        *plus_2_bonus = None;
                        *plus_1_bonus = None;
                    }

                    for choice in context {
                        match &choice.progress {
                            LevelUpSelectionProgress::Class(chosen_class) => {
                                if let Some(class) = chosen_class {
                                    ui.same_line();
                                    if ui.button(format!("Default ({})", class)) {
                                        if let Some(class) =
                                            registry::classes::CLASS_REGISTRY.get(class)
                                        {
                                            // Reset assignments to class defaults
                                            for (ability, score) in class.default_abilities.iter() {
                                                assignments.insert(*ability, *score);
                                            }
                                            *remaining_budget = *point_budget;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(table) =
                        table_columns!(ui, "Abilities", "Ability", "Score", "Mod", "+2", "+1",)
                    {
                        for ability in Ability::iter() {
                            // Ability name
                            ui.table_next_column();
                            ui.text(ability.to_string());

                            // Ability score
                            ui.table_next_column();

                            // Button for decreasing ability score
                            ui.same_line();
                            if ui.button_with_size(format!("-##{}", ability), [30.0, 0.0]) {
                                if let Some(score) = assignments.get_mut(&ability) {
                                    if *score > 8 {
                                        *score -= 1;
                                    }
                                }
                            }

                            let ability_score = assignments.get(&ability).unwrap().clone();
                            ui.same_line();
                            // Fixed width format: centered in a 2-character field (e.g., " 8", "10", "14")
                            let mut final_score = ability_score;
                            if let Some(plus_2) = plus_2_bonus {
                                if *plus_2 == ability {
                                    final_score += 2;
                                }
                            }
                            if let Some(plus_1) = plus_1_bonus {
                                if *plus_1 == ability {
                                    final_score += 1;
                                }
                            }

                            let formatted_score = format!("{:^2}", final_score);
                            ui.text(formatted_score);

                            // Button for increasing ability score
                            ui.same_line();
                            if ui.button_with_size(format!("+##{}", ability), [30.0, 0.0]) {
                                if let Some(score) = assignments.get_mut(&ability) {
                                    if *score < 15 {
                                        let price_of_current = scores_cost.get(score).unwrap();
                                        let price_of_next = scores_cost.get(&(*score + 1)).unwrap();
                                        let price_of_increase = price_of_next - price_of_current;
                                        if price_of_increase <= *remaining_budget {
                                            *score += 1;
                                        }
                                    }
                                }
                            }

                            // Recalculate remaining budget
                            *remaining_budget = *point_budget
                                - assignments
                                    .values()
                                    .map(|v| scores_cost.get(v).unwrap())
                                    .sum::<u8>();

                            // Ability modifier
                            // TODO: Do it manually for now
                            ui.table_next_column();
                            let ability_modifier = (final_score as i8 - 10) / 2;
                            let total = if ability_modifier >= 0 {
                                format!("+{}", ability_modifier)
                            } else {
                                format!("{}", ability_modifier)
                            };
                            ui.text(total);

                            // +2 Bonus column
                            ui.table_next_column();
                            let is_plus_2 = plus_2_bonus.map_or(false, |a| a == ability);
                            let mut checkbox_plus_2 = is_plus_2;
                            if ui.checkbox(format!("##plus2_{}", ability), &mut checkbox_plus_2) {
                                if checkbox_plus_2 {
                                    // Deselect +1 if it was the same ability
                                    if plus_1_bonus.map_or(false, |a| a == ability) {
                                        *plus_1_bonus = None;
                                    }
                                    *plus_2_bonus = Some(ability);
                                } else if is_plus_2 {
                                    *plus_2_bonus = None;
                                }
                            }

                            // +1 Bonus column
                            ui.table_next_column();
                            let is_plus_1 = plus_1_bonus.map_or(false, |a| a == ability);
                            let mut checkbox_plus_1 = is_plus_1;
                            if ui.checkbox(format!("##plus1_{}", ability), &mut checkbox_plus_1) {
                                if checkbox_plus_1 {
                                    // Deselect +2 if it was the same ability
                                    if plus_2_bonus.map_or(false, |a| a == ability) {
                                        *plus_2_bonus = None;
                                    }
                                    *plus_1_bonus = Some(ability);
                                } else if is_plus_1 {
                                    *plus_1_bonus = None;
                                }
                            }
                        }

                        table.end();
                    }
                } else {
                    ui.text("Mismatched progress type for Ability Scores choice");
                }
            }

            LevelUpChoice::Effect(ref effects) => {
                if let LevelUpSelectionProgress::Effect(ref mut selection) = self.progress {
                    for effect in effects {
                        let is_selected = selection.as_ref().is_some_and(|s| *s == *effect);

                        let style_color = if is_selected {
                            ui.push_style_color(imgui::StyleColor::Button, [0.2, 0.6, 0.2, 1.0])
                        } else {
                            ui.push_style_color(
                                imgui::StyleColor::Button,
                                ui.style_color(imgui::StyleColor::Button),
                            )
                        };

                        if ui.button(format!("{}", effect)) {
                            *selection = Some(effect.clone());
                        }

                        style_color.pop();
                    }
                } else {
                    ui.text("Mismatched progress type for Effect choice");
                }
            }

            LevelUpChoice::SkillProficiency(ref skills, ref num_choices) => {
                if let LevelUpSelectionProgress::SkillProficiency {
                    ref mut selected,
                    ref mut remaining_selections,
                } = self.progress
                {
                    ui.text(format!(
                        "Select up to {} skills ({} selected):",
                        num_choices,
                        selected.len()
                    ));

                    if ui.button("Reset##Skills") {
                        selected.clear();
                        *remaining_selections = *num_choices;
                    }

                    if let Some(table) = table_columns!(ui, "Skills", "Skill", "") {
                        for skill in skills {
                            ui.table_next_column();
                            ui.text(skill.to_string());

                            ui.table_next_column();
                            let is_selected = selected.contains(skill);
                            let mut checked = is_selected;

                            if ui.checkbox(format!("##{}", skill), &mut checked) {
                                if checked {
                                    // Only add if we have room and it's not already selected
                                    if selected.len() < *num_choices as usize {
                                        selected.insert(*skill);
                                        *remaining_selections -= 1;
                                    }
                                } else {
                                    selected.remove(skill);
                                    *remaining_selections += 1;
                                }
                            }
                        }

                        table.end();
                    }
                } else {
                    ui.text("Mismatched progress type for Skill Proficiency choice");
                }
            }

            _ => {
                ui.text(format!("Not implemented yet :^)",));
            }
        }
    }
}
