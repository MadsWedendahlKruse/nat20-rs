use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use imgui::TreeNodeFlags;
use nat20_rs::{
    components::{
        ability::{Ability, AbilityScoreSet},
        class::{ClassName, SubclassName},
        feat,
        id::{EffectId, FeatId},
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
        render_button_disabled_conditionally, render_button_selectable, render_uniform_buttons,
        render_uniform_buttons_do, render_window_at_cursor,
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
    Feat(Option<FeatId>),
    AbilityScoreImprovement {
        base_scores: HashMap<Ability, u8>,
        assignments: HashMap<Ability, u8>,
        remaining_points: u8,
    },
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
            LevelUpDecisionProgress::Feat(feat) => feat.is_some(),
            LevelUpDecisionProgress::AbilityScoreImprovement {
                assignments,
                remaining_points,
                ..
            } => remaining_points == &0 && !assignments.is_empty(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            LevelUpDecisionProgress::Class(class) => class.is_none(),
            LevelUpDecisionProgress::Subclass(subclass) => subclass.is_none(),
            LevelUpDecisionProgress::Effect(effect) => effect.is_none(),
            LevelUpDecisionProgress::SkillProficiency { selected, .. } => selected.is_empty(),
            LevelUpDecisionProgress::AbilityScores { assignments, .. } => assignments.is_empty(),
            LevelUpDecisionProgress::Feat(feat) => feat.is_none(),
            LevelUpDecisionProgress::AbilityScoreImprovement { assignments, .. } => {
                assignments.is_empty()
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
            LevelUpDecisionProgress::Feat(feat) => LevelUpDecision::Feat(feat.unwrap()),
            LevelUpDecisionProgress::AbilityScoreImprovement { assignments, .. } => {
                LevelUpDecision::AbilityScoreImprovement(assignments)
            }
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
            LevelUpPrompt::Feat(_) => LevelUpDecisionProgress::Feat(None),
            LevelUpPrompt::AbilityScoreImprovement { budget, .. } => {
                LevelUpDecisionProgress::AbilityScoreImprovement {
                    base_scores: HashMap::new(),
                    assignments: HashMap::new(),
                    remaining_points: *budget,
                }
            }
        }
    }

    fn default_from_prompt_and_character(
        prompt: &LevelUpPrompt,
        world: &World,
        entity: Entity,
    ) -> Self {
        if let Ok(levels) = world.get::<&CharacterLevels>(entity) {
            if levels.total_level() > 0 {
                match prompt {
                    LevelUpPrompt::Class(_) => {
                        return LevelUpDecisionProgress::Class(Some(
                            levels.latest_class().unwrap().clone(),
                        ));
                    }

                    LevelUpPrompt::AbilityScores(_, _) => {
                        let mut assignments = HashMap::new();
                        if let Some(class) =
                            registry::classes::CLASS_REGISTRY.get(levels.latest_class().unwrap())
                        {
                            // Reset assignments to class defaults
                            for (ability, score) in class.default_abilities.iter() {
                                assignments.insert(*ability, *score);
                            }
                        }
                        return LevelUpDecisionProgress::AbilityScores {
                            assignments,
                            remaining_budget: 0,
                            plus_2_bonus: None,
                            plus_1_bonus: None,
                        };
                    }

                    LevelUpPrompt::AbilityScoreImprovement { budget, .. } => {
                        let base_scores =
                            systems::helpers::get_component::<AbilityScoreSet>(world, entity)
                                .scores
                                .iter()
                                .map(|(ability, score)| (*ability, score.total() as u8))
                                .collect();
                        return LevelUpDecisionProgress::AbilityScoreImprovement {
                            base_scores,
                            assignments: HashMap::new(),
                            remaining_points: *budget,
                        };
                    }

                    _ => {}
                }
            }
        }
        Self::from_prompt(prompt)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LevelUpPromptWithProgress {
    prompt: LevelUpPrompt,
    progress: LevelUpDecisionProgress,
    initial_value: LevelUpDecisionProgress,
}

impl LevelUpPromptWithProgress {
    fn new(prompt: LevelUpPrompt) -> Self {
        let progress = LevelUpDecisionProgress::from_prompt(&prompt);
        Self {
            prompt,
            progress: progress.clone(),
            initial_value: progress,
        }
    }

    fn new_from_prompt_and_character(prompt: LevelUpPrompt, world: &World, entity: Entity) -> Self {
        let progress =
            LevelUpDecisionProgress::default_from_prompt_and_character(&prompt, world, entity);
        Self {
            prompt: prompt,
            progress: progress.clone(),
            initial_value: progress,
        }
    }

    fn reset(&mut self) {
        self.progress = self.initial_value.clone();
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

        // TODO: Naming seems all over the place here (both variables and structs)
        // Check if any of the current decisions are still valid
        let mut valid_decisions = Vec::new();

        // Split decisions by whether they are completed or still pending
        let (completed_decisions, pending_decisions) = self
            .pending_decisions
            .iter()
            .partition::<Vec<_>, _>(|p| p.progress.is_complete());

        // Keep all the decisions which are still valid for the new level-up session
        for prompt_progress in completed_decisions {
            let decision = prompt_progress.progress.clone().finalize();
            let result = self
                .level_up_session
                .as_mut()
                .unwrap()
                .advance(&mut self.world, &decision);
            if result.is_ok() {
                valid_decisions.push(prompt_progress.clone());
            }
        }

        let pending_prompts = self.level_up_session.as_ref().unwrap().pending_prompts();

        // Keep all the decisions in progress which are still valid for the new level-up session
        for promp_progress in pending_decisions {
            if pending_prompts.iter().any(|p| *p == promp_progress.prompt)
                && !promp_progress.progress.is_empty()
            {
                valid_decisions.push(promp_progress.clone());
            }
        }

        self.pending_decisions = valid_decisions;

        // Add any new pending prompts that were triggered by the level-up session
        for prompt in pending_prompts {
            let already_present = self.pending_decisions.iter().any(|p| p.prompt == *prompt);
            if !already_present {
                self.pending_decisions.push(
                    LevelUpPromptWithProgress::new_from_prompt_and_character(
                        prompt.clone(),
                        &self.world,
                        self.current_character.unwrap(),
                    ),
                );
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

                    let mut decision_updated = None;
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
                                pending_decision.render_mut(ui);
                                tab.end();
                            }

                            tab_bar.end();

                            if let Some(level_up_session) = &mut self.level_up_session {
                                // Check if a decision has been revoked
                                if level_up_session.is_complete()
                                    && !pending_decision.progress.is_complete()
                                {
                                    decision_updated = Some(pending_decision.clone());
                                }

                                // Check if the decision is complete
                                if pending_decision.progress.is_complete() {
                                    let decision = pending_decision.progress.clone().finalize();
                                    if !level_up_session.decisions().contains(&decision) {
                                        println!("New decision: {:?}", decision);
                                        decision_updated = Some(pending_decision.clone());
                                    }
                                }
                            }
                        }
                    }

                    // All the other decisions arise from what class is chosen,
                    // so if the class decision is updated, we can reset the
                    // other decisions (little bit of a hack)
                    if let Some(decision) = decision_updated.as_ref() {
                        if matches!(decision.prompt, LevelUpPrompt::Class(_)) {
                            self.pending_decisions = vec![decision.clone()];
                        }
                    }

                    // Check if any new pending prompts were triggered
                    // Or if there are no pending decisions to choose from
                    if decision_updated.is_some() || self.pending_decisions.is_empty() {
                        self.sync_pending_decisions();
                    }

                    let buttons_disabled = !self.level_up_session.as_ref().unwrap().is_complete();
                    let tooltip = "Please complete all required choices before proceeding.";

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

impl ImguiRenderableMut for LevelUpPromptWithProgress {
    fn render_mut(&mut self, ui: &imgui::Ui) {
        match &self.prompt {
            LevelUpPrompt::Class(classes) => {
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

            LevelUpPrompt::AbilityScores(scores_cost, point_budget) => {
                let mut reset = false;
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

                    if ui.button("Clear##Abilities") {
                        for ability in Ability::iter() {
                            assignments.insert(ability, 8);
                        }
                        *remaining_budget = *point_budget;
                        *plus_2_bonus = None;
                        *plus_1_bonus = None;
                    }

                    ui.same_line();

                    if ui.button("Recommended##Abilities") {
                        reset = true;
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Click to reset to recommended abilities for your class.\n\
                             This will clear any custom assignments.",
                        );
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

                if reset {
                    self.reset();
                }
            }

            LevelUpPrompt::Effect(effects) => {
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

            LevelUpPrompt::SkillProficiency(skills, num_prompts) => {
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

            LevelUpPrompt::Feat(feats) => {
                if let LevelUpDecisionProgress::Feat(ref mut decision) = self.progress {
                    for feat in feats {
                        if render_button_selectable(
                            ui,
                            format!("{}", feat),
                            [0., 0.],
                            decision.as_ref().is_some_and(|s| s == feat),
                        ) {
                            *decision = Some(feat.clone());
                        }
                    }
                } else {
                    ui.text("Mismatched progress type for Feat prompt");
                }
            }

            LevelUpPrompt::AbilityScoreImprovement {
                feat,
                budget,
                abilities,
                max_score,
            } => {
                if let LevelUpDecisionProgress::AbilityScoreImprovement {
                    ref base_scores,
                    ref mut assignments,
                    ref mut remaining_points,
                } = self.progress
                {
                    ui.separator_with_text(feat.to_string());

                    ui.text(format!("Remaining Points: {}", remaining_points));

                    if ui.button("Reset##ASI") {
                        assignments.clear();
                        *remaining_points = *budget;
                    }

                    if let Some(table) = table_with_columns!(ui, "Abilities", "Ability", "Score") {
                        for ability in Ability::iter() {
                            if !abilities.contains(&ability) {
                                continue; // Skip abilities not in the set
                            }
                            // Ability name
                            ui.table_next_column();
                            ui.text(ability.to_string());

                            // Ability score
                            ui.table_next_column();

                            if assignments.get(&ability).is_none() {
                                assignments.insert(ability, 0);
                            }

                            // Button for decreasing ability score
                            ui.same_line();
                            let can_decrease = assignments.get(&ability).unwrap() > &0;
                            let disabled_token_decrease = ui.begin_disabled(!can_decrease);
                            if ui.button_with_size(format!("-##{}", ability), [30.0, 0.0]) {
                                if can_decrease {
                                    assignments.get_mut(&ability).map(|score| {
                                        *score -= 1;
                                        *remaining_points += 1;
                                    });
                                }
                            }
                            disabled_token_decrease.end();

                            let total_score = base_scores.get(&ability).unwrap()
                                + assignments.get(&ability).unwrap();
                            ui.same_line();
                            ui.text(format!("{:^2}", total_score));

                            // Button for increasing ability score
                            ui.same_line();
                            let can_increase = total_score < *max_score && *remaining_points > 0;
                            let disabled_token_increase = ui.begin_disabled(!can_increase);
                            if ui.button_with_size(format!("+##{}", ability), [30.0, 0.0]) {
                                if can_increase {
                                    assignments.get_mut(&ability).map(|score| {
                                        *score += 1;
                                        *remaining_points -= 1;
                                    });
                                }
                            }
                            disabled_token_increase.end();
                        }
                        table.end();
                    }
                } else {
                    ui.text("Mismatched progress type for Ability Score Improvement prompt");
                }
            }

            LevelUpPrompt::Subclass(subclasses) => {
                if let LevelUpDecisionProgress::Subclass(ref mut decision) = self.progress {
                    for subclass in subclasses {
                        if render_button_selectable(
                            ui,
                            format!("{}", subclass.name),
                            [0., 0.],
                            decision.as_ref().is_some_and(|s| s == subclass),
                        ) {
                            *decision = Some(subclass.clone());
                        }
                    }
                } else {
                    ui.text("Mismatched progress type for Subclass prompt");
                }
            }

            _ => {
                ui.text(format!("Not implemented yet :^)",));
            }
        }
    }
}
