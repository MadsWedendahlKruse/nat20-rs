use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use imgui::TreeNodeFlags;
use nat20_rs::{
    components::{
        ability::Ability,
        class::{ClassName, SubclassName},
        id::EffectId,
        level::CharacterLevels,
        level_up::LevelUpPrompt,
        skill::Skill,
    },
    entities::character::{Character, CharacterTag},
    registry,
    systems::{
        self,
        level_up::{LevelUpDecision, LevelUpSession},
    },
    test_utils::fixtures,
};
use strum::IntoEnumIterator;

use crate::{
    buttons,
    render::utils::{
        ImguiRenderable, ImguiRenderableMut, ImguiRenderableMutWithContext,
        render_button_disabled_conditionally, render_button_selectable, render_uniform_buttons_do,
        render_window_at_cursor,
    },
    table_with_columns,
};

#[derive(Debug, Clone, PartialEq)]
enum LevelUpDecisionProgress {
    Class(Option<ClassName>),
    Subclass(Option<SubclassName>),
    Effect(Option<EffectId>),
    SkillProficiency {
        selected: HashSet<Skill>,
        remaining_decisions: u8,
    },
    AbilityScores {
        assignments: HashMap<Ability, u8>,
        remaining_budget: u8,
        plus_2_bonus: Option<Ability>,
        plus_1_bonus: Option<Ability>,
    },
    // Add more as needed
}

impl LevelUpDecisionProgress {
    // TODO: Might need more complex validation
    fn is_complete(&self) -> bool {
        match self {
            LevelUpDecisionProgress::Class(class) => class.is_some(),
            LevelUpDecisionProgress::Subclass(subclass) => subclass.is_some(),
            LevelUpDecisionProgress::Effect(effect) => effect.is_some(),
            LevelUpDecisionProgress::SkillProficiency {
                selected,
                remaining_decisions,
            } => remaining_decisions == &0 && selected.len() > 0,
            LevelUpDecisionProgress::AbilityScores {
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

    fn finalize(self) -> LevelUpDecision {
        match self {
            LevelUpDecisionProgress::Class(class) => LevelUpDecision::Class(class.unwrap()),
            LevelUpDecisionProgress::Subclass(subclass) => {
                LevelUpDecision::Subclass(subclass.unwrap())
            }
            LevelUpDecisionProgress::Effect(effect) => LevelUpDecision::Effect(effect.unwrap()),
            LevelUpDecisionProgress::SkillProficiency { selected, .. } => {
                LevelUpDecision::SkillProficiency(selected)
            }
            LevelUpDecisionProgress::AbilityScores {
                assignments,
                plus_2_bonus,
                plus_1_bonus,
                ..
            } => LevelUpDecision::AbilityScores {
                scores: assignments,
                plus_2_bonus: plus_2_bonus.unwrap(),
                plus_1_bonus: plus_1_bonus.unwrap(),
            },
        }
    }

    fn from_prompt(prompt: &LevelUpPrompt) -> Self {
        match prompt {
            LevelUpPrompt::Class(_) => LevelUpDecisionProgress::Class(None),
            LevelUpPrompt::Subclass(_) => LevelUpDecisionProgress::Subclass(None),
            LevelUpPrompt::Effect(_) => LevelUpDecisionProgress::Effect(None),
            LevelUpPrompt::SkillProficiency(_, required) => {
                LevelUpDecisionProgress::SkillProficiency {
                    selected: HashSet::new(),
                    remaining_decisions: *required,
                }
            }
            LevelUpPrompt::AbilityScores(_, budget) => LevelUpDecisionProgress::AbilityScores {
                assignments: HashMap::new(),
                remaining_budget: *budget,
                plus_2_bonus: None,
                plus_1_bonus: None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LevelUpPromptWithProgress {
    prompt: LevelUpPrompt,
    progress: LevelUpDecisionProgress,
}

impl LevelUpPromptWithProgress {
    fn new(prompt: LevelUpPrompt) -> Self {
        let progress = LevelUpDecisionProgress::from_prompt(&prompt);
        Self { prompt, progress }
    }
}

#[derive(Debug, PartialEq)]
pub enum CharacterCreationState {
    ChoosingMethod,
    FromPredefined,
    FromScratch,
    CreationComplete,
}

pub struct CharacterCreation {
    /// World where the character creation takes place. Note that this is not the
    /// main game world, but rather a dummy world. Once the character is created,
    /// it will be added to the main game world.
    world: World,
    state: Option<CharacterCreationState>,
    current_character: Option<Entity>,
    /// The initial state of the character when the level-up session was first created.
    /// Whenever the user changes a decision, this is used to reset the character
    /// to the initial state, and then apply the new decisions.
    initial_character: Option<Character>,
    level_up_session: Option<LevelUpSession>,
    pending_decisions: Vec<LevelUpPromptWithProgress>,
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
            let entity = spawner(&mut world).id();
            // They spawn with zero health, so we heal them to full
            systems::health::heal_full(&mut world, entity);
        }

        Self {
            world,
            state: None,
            current_character: None,
            initial_character: None,
            level_up_session: None,
            pending_decisions: Vec::new(),
        }
    }

    pub fn set_state(&mut self, state: CharacterCreationState) {
        self.state = Some(state);
    }

    pub fn creation_complete(&self) -> bool {
        matches!(self.state, Some(CharacterCreationState::CreationComplete))
    }

    pub fn get_character(&mut self) -> Option<Character> {
        if let Some(entity) = self.current_character {
            let character = Some(Character::from_world(&self.world, entity));
            self.reset();
            character
        } else {
            None
        }
    }

    fn reset(&mut self) {
        if let Some(entity) = self.current_character {
            self.world.despawn(entity).unwrap();
        }

        self.state = None;
        self.current_character = None;
        self.level_up_session = None;
        self.pending_decisions.clear();
    }

    fn sync_pending_decisions(&mut self) {
        // Preserve the name
        let name = systems::helpers::get_component_clone::<String>(
            &self.world,
            self.current_character.unwrap(),
        );

        // Drop the current character and spawn a new one with the same name
        // This is to re-apply any changes made during the level-up session
        self.world.despawn(self.current_character.unwrap()).unwrap();
        self.current_character = Some(
            self.world
                .spawn(self.initial_character.as_ref().unwrap().clone()),
        );
        systems::helpers::set_component(&mut self.world, self.current_character.unwrap(), name);

        self.level_up_session = Some(LevelUpSession::new(
            &self.world,
            self.current_character.unwrap(),
        ));

        // Check if any of the current decisions are still valid
        let mut valid_in_progress = Vec::new();
        for prompt_progress in &self.pending_decisions {
            if prompt_progress.progress.is_complete() {
                let decision = prompt_progress.progress.clone().finalize();
                let result = self
                    .level_up_session
                    .as_mut()
                    .unwrap()
                    .advance(&mut self.world, &decision);
                if result.is_ok() {
                    valid_in_progress.push(prompt_progress.clone());
                }
            }
        }
        self.pending_decisions = valid_in_progress;

        let pending_prompts = self.level_up_session.as_ref().unwrap().pending_prompts();

        for prompt in pending_prompts {
            let already_present = self.pending_decisions.iter().any(|p| p.prompt == *prompt);
            if !already_present {
                self.pending_decisions
                    .push(LevelUpPromptWithProgress::new(prompt.clone()));
            }
        }
    }
}

impl ImguiRenderableMut for CharacterCreation {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        if self.state.is_none() {
            return;
        }
        render_window_at_cursor(ui, "Character Creation", true, || {
            match self.state {
                Some(CharacterCreationState::ChoosingMethod) => {
                    let mut buttons = buttons![
                        "From Predefined" => |s: &mut Self| {
                            s.state = Some(CharacterCreationState::FromPredefined);
                        },
                        "From Scratch" => |s: &mut Self| {
                            s.reset();
                            s.state = Some(CharacterCreationState::FromScratch);
                        },
                        "Cancel" => |s: &mut Self| {
                            s.state = None;
                        },
                    ];
                    let _ = render_uniform_buttons_do(ui, &mut buttons, self, [20.0, 5.0]);
                }

                Some(CharacterCreationState::FromPredefined) => {
                    // Avoid double borrow
                    let characters = self
                        .world
                        .query_mut::<(&String, &CharacterTag)>()
                        .into_iter()
                        .map(|(entity, (name, tag))| (entity, name.clone(), tag.clone()))
                        .collect::<Vec<_>>();
                    for (entity, name, tag) in characters {
                        if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                            if ui.button(format!("Add to World##{}", entity.id())) {
                                self.current_character = Some(entity);
                                self.state = Some(CharacterCreationState::CreationComplete);
                            }
                            ui.separator();
                            // TODO: Maybe they shouldn't be rendered as mutable?
                            (entity, tag).render_mut_with_context(ui, &mut self.world);
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
                    }
                    ui.separator();

                    if self.current_character.is_none() {
                        self.initial_character = Some(Character::new("Johnny Hero"));
                        self.current_character = Some(
                            self.world
                                .spawn(self.initial_character.as_ref().unwrap().clone()),
                        );
                    }

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

                    {
                        let level = systems::helpers::get_component::<CharacterLevels>(
                            &self.world,
                            self.current_character.unwrap(),
                        );
                        level.render(ui);
                    }

                    ui.separator();

                    // TODO: Not terribly efficient, but works for now
                    let prompt_clones = self.pending_decisions.clone();

                    let mut decision_updated = false;
                    for pending_decision in &mut self.pending_decisions {
                        if let Some(tab_bar) = ui.tab_bar(format!("CharacterTabs")) {
                            let style_tokens = if pending_decision.progress.is_complete() {
                                Some(
                                    [
                                        (imgui::StyleColor::Tab, [0.0, 0.6, 0.0, 1.0]),
                                        (imgui::StyleColor::TabHovered, [0.0, 0.75, 0.0, 1.0]),
                                        (imgui::StyleColor::TabSelected, [0.0, 0.75, 0.0, 1.0]),
                                    ]
                                    .iter()
                                    .map(|(style, color)| ui.push_style_color(*style, *color))
                                    .collect::<Vec<_>>(),
                                )
                            } else {
                                None
                            };

                            if let Some(tab) =
                                ui.tab_item(format!("{}", pending_decision.prompt.name()))
                            {
                                pending_decision.render_mut_with_context(ui, &prompt_clones);
                                tab.end();
                            }

                            tab_bar.end();

                            if let Some(level_up_session) = &mut self.level_up_session {
                                // Check if a decision has been revoked
                                if level_up_session.is_complete()
                                    && !pending_decision.progress.is_complete()
                                {
                                    decision_updated = true;
                                }

                                // Check if the decision is complete
                                if pending_decision.progress.is_complete() {
                                    let decision = pending_decision.progress.clone().finalize();
                                    if !level_up_session.decisions().contains(&decision) {
                                        println!("New decision: {:?}", decision);
                                        decision_updated = true;
                                    }
                                }
                            }
                        }
                    }

                    // Check if any new pending prompts were triggered
                    // Or if there are no pending decisions to choose from
                    if decision_updated || self.pending_decisions.is_empty() {
                        self.sync_pending_decisions();
                    }

                    let buttons_disabled = !self.level_up_session.as_ref().unwrap().is_complete();
                    let tooltip = "Please complete all required choices (*) before proceeding.";

                    ui.separator();
                    if render_button_disabled_conditionally(
                        ui,
                        "Level Up",
                        [200.0, 30.0],
                        buttons_disabled,
                        tooltip,
                    ) {
                        self.initial_character = Some(Character::from_world(
                            &self.world,
                            self.current_character.unwrap(),
                        ));
                        self.pending_decisions.clear();
                    }

                    ui.separator();
                    if render_button_disabled_conditionally(
                        ui,
                        "Finish Character Creation",
                        [200.0, 30.0],
                        buttons_disabled,
                        tooltip,
                    ) {
                        self.state = Some(CharacterCreationState::CreationComplete);
                    }
                }

                _ => {}
            }
        });
    }
}

impl ImguiRenderableMutWithContext<&Vec<LevelUpPromptWithProgress>> for LevelUpPromptWithProgress {
    fn render_mut_with_context(
        &mut self,
        ui: &imgui::Ui,
        all_prompts: &Vec<LevelUpPromptWithProgress>,
    ) {
        match self.prompt {
            LevelUpPrompt::Class(ref classes) => {
                if let LevelUpDecisionProgress::Class(ref mut decision) = self.progress {
                    let button_size = [100.0, 30.0];
                    let buttons_per_row = 4;

                    for (i, class) in classes.iter().enumerate() {
                        if render_button_selectable(
                            ui,
                            format!("{}", class),
                            button_size,
                            decision.is_some_and(|s| s == *class),
                        ) {
                            *decision = Some(class.clone());
                        }

                        if (i + 1) % buttons_per_row != 0 {
                            ui.same_line();
                        }
                    }
                } else {
                    ui.text("Mismatched progress type for Class prompt");
                }
            }

            LevelUpPrompt::AbilityScores(ref scores_cost, ref point_budget) => {
                if let LevelUpDecisionProgress::AbilityScores {
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

                    for prompt in all_prompts {
                        match &prompt.progress {
                            LevelUpDecisionProgress::Class(chosen_class) => {
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
                        table_with_columns!(ui, "Abilities", "Ability", "Score", "Mod", "+2", "+1",)
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
                    ui.text("Mismatched progress type for Ability Scores prompt");
                }
            }

            LevelUpPrompt::Effect(ref effects) => {
                if let LevelUpDecisionProgress::Effect(ref mut decision) = self.progress {
                    for effect in effects {
                        if render_button_selectable(
                            ui,
                            format!("{}", effect),
                            [0., 0.],
                            decision.as_ref().is_some_and(|s| *s == *effect),
                        ) {
                            *decision = Some(effect.clone());
                        }
                    }
                } else {
                    ui.text("Mismatched progress type for Effect prompt");
                }
            }

            LevelUpPrompt::SkillProficiency(ref skills, ref num_prompts) => {
                if let LevelUpDecisionProgress::SkillProficiency {
                    ref mut selected,
                    ref mut remaining_decisions,
                } = self.progress
                {
                    ui.text(format!(
                        "Select up to {} skills ({} selected):",
                        num_prompts,
                        selected.len()
                    ));

                    if ui.button("Reset##Skills") {
                        selected.clear();
                        *remaining_decisions = *num_prompts;
                    }

                    if let Some(table) = table_with_columns!(ui, "Skills", "Skill", "") {
                        for skill in skills {
                            ui.table_next_column();
                            ui.text(skill.to_string());

                            ui.table_next_column();
                            let is_selected = selected.contains(skill);
                            let mut checked = is_selected;

                            if ui.checkbox(format!("##{}", skill), &mut checked) {
                                if checked {
                                    // Only add if we have room and it's not already selected
                                    if selected.len() < *num_prompts as usize {
                                        selected.insert(*skill);
                                        *remaining_decisions -= 1;
                                    }
                                } else {
                                    selected.remove(skill);
                                    *remaining_decisions += 1;
                                }
                            }
                        }

                        table.end();
                    }
                } else {
                    ui.text("Mismatched progress type for Skill Proficiency prompt");
                }
            }

            _ => {
                ui.text(format!("Not implemented yet :^)",));
            }
        }
    }
}
