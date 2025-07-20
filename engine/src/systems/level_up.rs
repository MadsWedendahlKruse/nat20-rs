use std::{
    collections::{HashSet, VecDeque},
    iter::FromIterator,
};

use hecs::{Entity, World};

use crate::{
    components::{
        ability::{AbilityScore, AbilityScoreSet},
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
    test_utils::cli::CliChoiceProvider,
};

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

pub struct LevelUpSession {
    character: Entity,
    pending: VecDeque<LevelUpChoice>,
}

impl LevelUpSession {
    pub fn new(character: Entity) -> Self {
        let mut pending = VecDeque::new();
        pending.push_back(LevelUpChoice::class());
        LevelUpSession { character, pending }
    }

    pub fn advance(
        &mut self,
        world: &mut World,
        provider: &mut impl ChoiceProvider,
    ) -> Result<(), LevelUpError> {
        while let Some(choice) = self.pending.pop_front() {
            let selection = provider.provide(&choice);
            let next = resolve_level_up_choice(world, self.character, choice, selection)?;
            for c in next {
                self.pending.push_back(c)
            }
        }
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
                let skills_vec: Vec<_> = skills.iter().cloned().collect();
                let selected = Self::select_multiple(
                    &format!("Select {} skill(s) to gain proficiency in:", num_choices),
                    &skills_vec,
                    *num_choices,
                    |skill| format!("{:?}", skill),
                    true, // Ensure unique selections
                );
                LevelUpSelection::SkillProficiency(HashSet::from_iter(selected))
            }

            #[allow(unreachable_patterns)]
            _ => {
                todo!("Implement CLI choice provider for other LevelUpChoice variants");
            }
        }
    }
}

/// A provider that hands out selections from a predefined list.
/// Useful for testing or when you want to simulate a specific sequence of choices.
pub struct PredefinedChoiceProvider {
    name: String,
    responses: VecDeque<LevelUpSelection>,
}

impl PredefinedChoiceProvider {
    pub fn new(name: String, responses: Vec<LevelUpSelection>) -> Self {
        Self {
            name,
            responses: responses.into(),
        }
    }
}

impl ChoiceProvider for PredefinedChoiceProvider {
    fn provide(&mut self, _choice: &LevelUpChoice) -> LevelUpSelection {
        self.responses.pop_front().expect(&format!(
            "Ran out of predefined responses when leveling up {}",
            self.name
        ))
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
            if let Some(effect) = registry::effects::EFFECT_REGISTRY.get(effect_id) {
                systems::effects::add_effect(world, entity, effect_id);
            } else {
                return Err(LevelUpError::RegistryMissing(effect_id.to_string()));
            }
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
    let (new_level, total_level) = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        let new_level = character_levels.level_up(class.name.clone());
        (new_level, character_levels.total_level())
    };

    // If it's the first total level set default ability scores
    if total_level == 1 {
        for (ability, score) in class.default_abilities.iter() {
            systems::helpers::get_component_mut::<AbilityScoreSet>(world, entity)
                .set(*ability, AbilityScore::new(*ability, *score));
        }
    }

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
