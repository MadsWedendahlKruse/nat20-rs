use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::{Ability, AbilityScore, AbilityScoreSet},
        actions::action::{ActionContext, ActionMap},
        class::{Class, ClassBase, ClassName, SubclassName},
        id::EffectId,
        level::CharacterLevels,
        level_up::LevelUpChoice,
        proficiency::Proficiency,
        resource::{ResourceCostMap, ResourceMap},
        saving_throw::SavingThrowSet,
        skill::{Skill, SkillSet},
    },
    registry, systems,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelUpSelection {
    Class(ClassName),
    Subclass(SubclassName),
    Effect(EffectId),
    SkillProficiency(HashSet<Skill>),
    AbilityScores {
        scores: HashMap<Ability, u8>,
        plus_2_bonus: Ability,
        plus_1_bonus: Ability,
    },
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
            LevelUpSelection::AbilityScores { .. } => "AbilityScores",
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
    MissingChoiceForSelection {
        selection: LevelUpSelection,
    },
    RegistryMissing(String),
    // TODO: Add more error variants as needed
}

pub struct LevelUpSession {
    character: Entity,
    pending_choices: Vec<LevelUpChoice>,
    chosen_selections: Vec<LevelUpSelection>,
}

impl LevelUpSession {
    pub fn new(world: &World, character: Entity) -> Self {
        let mut pending = Vec::new();
        pending.push(LevelUpChoice::class());

        // Special level up choices when creating a new character
        if systems::helpers::get_component::<CharacterLevels>(world, character).total_level() == 0 {
            pending.push(LevelUpChoice::ability_scores());
        }

        LevelUpSession {
            character,
            pending_choices: pending,
            chosen_selections: Vec::new(),
        }
    }

    pub fn pending_choices(&self) -> &Vec<LevelUpChoice> {
        &self.pending_choices
    }

    pub fn chosen_selections(&self) -> &Vec<LevelUpSelection> {
        &self.chosen_selections
    }

    pub fn is_complete(&self) -> bool {
        self.pending_choices.is_empty()
    }

    pub fn advance(
        &mut self,
        world: &mut World,
        selection: &LevelUpSelection,
    ) -> Result<(), LevelUpError> {
        let mut new_choices = Vec::new();

        let mut resolved_choice = None;

        for choice in self.pending_choices.iter() {
            if choice.name() != selection.name() {
                continue;
            }

            let next_choices =
                resolve_level_up_choice(world, self.character, choice.clone(), selection.clone())?;
            new_choices.extend(next_choices);
            resolved_choice = Some(choice.clone());
            break;
        }

        if resolved_choice.is_none() {
            return Err(LevelUpError::MissingChoiceForSelection {
                selection: selection.clone(),
            });
        }

        self.pending_choices
            .retain(|c| c != resolved_choice.as_ref().unwrap());

        self.chosen_selections.push(selection.clone());

        self.pending_choices.extend(new_choices);
        Ok(())
    }
}

pub fn resolve_level_up_choice(
    world: &mut World,
    entity: Entity,
    choice: LevelUpChoice,
    selection: LevelUpSelection,
) -> Result<Vec<LevelUpChoice>, LevelUpError> {
    let mut choices = Vec::new();

    match (&choice, &selection) {
        (LevelUpChoice::Class(classes), LevelUpSelection::Class(class_name)) => {
            if !classes.contains(&class_name) {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&class_name) {
                choices.extend(increment_class_level(world, entity, &class));
            } else {
                return Err(LevelUpError::RegistryMissing(class_name.to_string()));
            }
        }

        (LevelUpChoice::Subclass(subclasses), LevelUpSelection::Subclass(subclass_name)) => {
            if !subclasses.contains(&subclass_name) {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&subclass_name.class) {
                if !class.subclasses.contains_key(&subclass_name) {
                    return Err(LevelUpError::InvalidSelection { choice, selection });
                }

                choices.extend(set_subclass(world, entity, class, subclass_name.clone()));
            } else {
                return Err(LevelUpError::RegistryMissing(
                    subclass_name.class.to_string(),
                ));
            }
        }

        (LevelUpChoice::Effect(effects), LevelUpSelection::Effect(effect_id)) => {
            if !effects.contains(&effect_id) {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            // TODO: Unnecessary check?
            systems::effects::add_effect(world, entity, effect_id);
        }

        (
            LevelUpChoice::SkillProficiency(skills, num_choices),
            LevelUpSelection::SkillProficiency(selected_skills),
        ) => {
            if selected_skills.len() != *num_choices as usize {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            for skill in selected_skills {
                if !skills.contains(&skill) {
                    return Err(LevelUpError::InvalidSelection { choice, selection });
                }
                // TODO: Expertise handling
                systems::helpers::get_component_mut::<SkillSet>(world, entity)
                    .set_proficiency(*skill, Proficiency::Proficient);
            }
        }

        (
            LevelUpChoice::AbilityScores(score_point_cost, num_points),
            LevelUpSelection::AbilityScores {
                scores,
                plus_2_bonus,
                plus_1_bonus,
            },
        ) => {
            if scores.len() != Ability::iter().count() {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            if scores
                .values()
                .any(|&score| !score_point_cost.contains_key(&score))
            {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            let total_cost = scores
                .iter()
                .map(|(_, score)| {
                    score_point_cost
                        .get(score)
                        .expect("Invalid ability score")
                        .clone()
                })
                .sum::<u8>();
            if total_cost != *num_points {
                return Err(LevelUpError::InvalidSelection { choice, selection });
            }

            let mut ability_score_set =
                systems::helpers::get_component_mut::<AbilityScoreSet>(world, entity);
            for (ability, score) in scores {
                let mut final_score = *score as i32;
                if ability == plus_2_bonus {
                    final_score += 2;
                } else if ability == plus_1_bonus {
                    final_score += 1;
                }
                ability_score_set.set(*ability, AbilityScore::new(*ability, final_score));
            }
        }

        _ => {
            // If the choice and selection are called the same, and we made it here,
            // it's probably just because it hasn't been implemented yet
            if choice.name() == selection.name() {
                todo!(
                    "Implement choice: {:?} with selection: {:?}",
                    choice,
                    selection
                );
            }

            return Err(LevelUpError::ChoiceSelectionMismatch { choice, selection });
        }
    }

    Ok(choices)
}

fn increment_class_level(world: &mut World, entity: Entity, class: &Class) -> Vec<LevelUpChoice> {
    let new_level = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        character_levels.level_up(class.name.clone())
    };

    for ability in class.saving_throw_proficiencies.iter() {
        systems::helpers::get_component_mut::<SavingThrowSet>(world, entity)
            .set_proficiency(*ability, Proficiency::Proficient);
    }

    // TODO: If it's a level that triggers a feat choice, and ability score improvement
    // is selected, then the Constitution modifier might increase, in which case we need to
    // recalculate hit points.
    systems::health::update_hit_points(world, entity);

    systems::spells::update_spell_slots(world, entity);

    apply_class_base(world, entity, &class.base, new_level)
}

fn set_subclass(
    world: &mut World,
    entity: Entity,
    class: &Class,
    subclass_name: SubclassName,
) -> Vec<LevelUpChoice> {
    let (subclass, level) = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        character_levels.set_subclass(subclass_name.class, subclass_name.clone());

        let subclass = class
            .subclass(&subclass_name)
            .expect("Subclass should exist in the class registry");
        let level = character_levels.class_level(&class.name).unwrap().level();

        (subclass, level)
    };

    apply_class_base(world, entity, subclass.base(), level)
}

fn apply_class_base(
    world: &mut World,
    entity: Entity,
    class_base: &ClassBase,
    level: u8,
) -> Vec<LevelUpChoice> {
    // Effect
    if let Some(effects_for_level) = class_base.effects_by_level.get(&level) {
        for effect in effects_for_level {
            systems::effects::add_effect(world, entity, effect);
        }
    }

    // Resources
    {
        let mut resources = systems::helpers::get_component_mut::<ResourceMap>(world, entity);
        if let Some(resources_for_level) = class_base.resources_by_level.get(&level) {
            for resource in resources_for_level {
                resources.add(resource.clone(), false);
            }
        }
    }

    // Actions
    {
        let mut actions = systems::helpers::get_component_mut::<ActionMap>(world, entity);
        if let Some(actions_for_level) = class_base.actions_by_level.get(&level) {
            for action_id in actions_for_level {
                if let Some((action, context)) = registry::actions::ACTION_REGISTRY.get(action_id) {
                    let resource_cost = &action.resource_cost().clone();
                    actions
                        .entry(action_id.clone())
                        .and_modify(|a: &mut (Vec<ActionContext>, ResourceCostMap)| {
                            a.0.push(context.clone().unwrap());
                            a.1.extend(resource_cost.clone());
                        })
                        .or_insert((vec![context.clone().unwrap()], resource_cost.clone()));
                } else {
                    panic!("Action {} not found in registry", action_id);
                }
            }
        }
    }

    // Return any additional choices that should be presented to the player
    class_base
        .choices_by_level
        .get(&level)
        .cloned()
        .unwrap_or_default()
}

pub fn apply_level_up_selection(
    world: &mut World,
    entity: Entity,
    levels: u8,
    responses: Vec<LevelUpSelection>,
) {
    let mut responses = responses;

    for level in 1..=levels {
        let name = systems::helpers::get_component_clone::<String>(world, entity);
        let mut level_up_session = LevelUpSession::new(world, entity);

        // Some of the responses are identical, e.g. selecting the same class
        // multiple times. Using retain would therefore remove all of them,
        // so we need to track the indices of the used responses.
        let mut used_indices = Vec::new();
        for (i, response) in responses.iter().enumerate() {
            let result = level_up_session.advance(world, response);
            match result {
                Ok(_) | Err(LevelUpError::MissingChoiceForSelection { .. }) => {
                    // This is expected to happen since the responses cover all
                    // levels, but the session only advances one level at a time.
                    used_indices.push(i);
                    if level_up_session.is_complete() {
                        break;
                    }
                }
                _ => {
                    panic!(
                        "Failed to apply level up response for {} at level {}: {:?}",
                        name, level, result
                    );
                }
            }
        }

        // Remove the used responses from the list
        for index in used_indices.iter().rev() {
            responses.remove(*index);
        }

        if !level_up_session.is_complete() {
            panic!(
                "Level up session for {} at level {} did not complete: {:?}",
                name,
                level,
                level_up_session.pending_choices()
            );
        }
    }
}
