use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::{
    components::{
        ability::{Ability, AbilityScore, AbilityScoreDistribution, AbilityScoreMap},
        actions::action::ActionMap,
        background::Background,
        class::{Class, ClassBase, ClassName, SubclassName},
        feat::Feat,
        hit_points::HitPoints,
        id::{ActionId, BackgroundId, EffectId, FeatId},
        level::CharacterLevels,
        level_up::LevelUpPrompt,
        modifier::ModifierSource,
        proficiency::{Proficiency, ProficiencyLevel},
        resource::{Resource, ResourceMap},
        saving_throw::SavingThrowSet,
        skill::{Skill, SkillSet},
    },
    registry, systems,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelUpDecision {
    Background(BackgroundId),
    Class(ClassName),
    Subclass(SubclassName),
    Effect(EffectId),
    SkillProficiency(HashSet<Skill>),
    AbilityScores(AbilityScoreDistribution),
    Feat(FeatId),
    AbilityScoreImprovement(HashMap<Ability, u8>),
    // Spell(SpellcastingClass, SpellOption),
    // etc.
}

impl LevelUpDecision {
    pub fn name(&self) -> &'static str {
        match self {
            LevelUpDecision::Background(_) => "Background",
            LevelUpDecision::Class(_) => "Class",
            LevelUpDecision::Subclass(_) => "Subclass",
            LevelUpDecision::Effect(_) => "Effect",
            LevelUpDecision::SkillProficiency(_) => "SkillProficiency",
            LevelUpDecision::AbilityScores { .. } => "AbilityScores",
            LevelUpDecision::Feat(_) => "Feat",
            LevelUpDecision::AbilityScoreImprovement(_) => "AbilityScoreImprovement",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LevelUpError {
    InvalidDecision {
        prompt: LevelUpPrompt,
        decision: LevelUpDecision,
    },
    PrompDecisionMismatch {
        prompt: LevelUpPrompt,
        decision: LevelUpDecision,
    },
    MissingChoiceForDecision {
        decision: LevelUpDecision,
    },
    RegistryMissing(String),
    // TODO: Add more error variants as needed
}

pub struct LevelUpSession {
    character: Entity,
    pending_prompts: Vec<LevelUpPrompt>,
    decisions: Vec<LevelUpDecision>,
}

impl LevelUpSession {
    pub fn new(world: &World, character: Entity) -> Self {
        let mut pending_prompts = Vec::new();

        let levels = systems::helpers::get_component::<CharacterLevels>(world, character);
        if levels.total_level() == 0 {
            pending_prompts.push(LevelUpPrompt::background());
        }

        pending_prompts.push(LevelUpPrompt::class());

        LevelUpSession {
            character,
            pending_prompts,
            decisions: Vec::new(),
        }
    }

    pub fn pending_prompts(&self) -> &Vec<LevelUpPrompt> {
        &self.pending_prompts
    }

    pub fn decisions(&self) -> &Vec<LevelUpDecision> {
        &self.decisions
    }

    pub fn is_complete(&self) -> bool {
        self.pending_prompts.is_empty()
    }

    pub fn advance(
        &mut self,
        world: &mut World,
        decision: &LevelUpDecision,
    ) -> Result<(), LevelUpError> {
        let mut new_prompts = Vec::new();

        let mut resolved_prompt = None;

        for prompt in self.pending_prompts.iter() {
            if prompt.name() != decision.name() {
                continue;
            }

            let next_prompts =
                resolve_level_up_prompt(world, self.character, prompt.clone(), decision.clone())?;
            new_prompts.extend(next_prompts);
            resolved_prompt = Some(prompt.clone());
            break;
        }

        if resolved_prompt.is_none() {
            return Err(LevelUpError::MissingChoiceForDecision {
                decision: decision.clone(),
            });
        }

        self.pending_prompts
            .retain(|c| c != resolved_prompt.as_ref().unwrap());

        self.decisions.push(decision.clone());

        self.pending_prompts.extend(new_prompts);
        Ok(())
    }

    pub fn chosen_class(&self) -> Option<ClassName> {
        self.decisions.iter().find_map(|d| match d {
            LevelUpDecision::Class(class_name) => Some(class_name.clone()),
            _ => None,
        })
    }
}

fn resolve_level_up_prompt(
    world: &mut World,
    entity: Entity,
    prompt: LevelUpPrompt,
    decision: LevelUpDecision,
) -> Result<Vec<LevelUpPrompt>, LevelUpError> {
    let mut prompts = Vec::new();

    match (&prompt, &decision) {
        (LevelUpPrompt::Background(backgrounds), LevelUpDecision::Background(background_id)) => {
            if !backgrounds.contains(background_id) {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            if let Some(background) = registry::backgrounds::BACKGROUND_REGISTRY.get(background_id)
            {
                prompts.extend(systems::backgrounds::set_background(
                    world, entity, background,
                ));
            } else {
                return Err(LevelUpError::RegistryMissing(background_id.to_string()));
            }
        }

        (LevelUpPrompt::Class(classes), LevelUpDecision::Class(class_name)) => {
            if !classes.contains(&class_name) {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            // Special prompt when creating a new character
            if systems::helpers::get_component::<CharacterLevels>(world, entity).total_level() == 0
            {
                prompts.push(LevelUpPrompt::ability_scores());
            }

            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&class_name) {
                prompts.extend(increment_class_level(world, entity, &class));
            } else {
                return Err(LevelUpError::RegistryMissing(class_name.to_string()));
            }
        }

        (LevelUpPrompt::Subclass(subclasses), LevelUpDecision::Subclass(subclass_name)) => {
            if !subclasses.contains(&subclass_name) {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&subclass_name.class) {
                if !class.subclasses.contains_key(&subclass_name) {
                    return Err(LevelUpError::InvalidDecision { prompt, decision });
                }

                prompts.extend(set_subclass(world, entity, class, subclass_name.clone()));
            } else {
                return Err(LevelUpError::RegistryMissing(
                    subclass_name.class.to_string(),
                ));
            }
        }

        (LevelUpPrompt::Effect(effects), LevelUpDecision::Effect(effect_id)) => {
            if !effects.contains(&effect_id) {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            // TODO: Unnecessary check?
            systems::effects::add_effect(world, entity, effect_id);
        }

        (
            LevelUpPrompt::SkillProficiency(skills, num_prompts, source),
            LevelUpDecision::SkillProficiency(selected_skills),
        ) => {
            if selected_skills.len() != *num_prompts as usize {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            for skill in selected_skills {
                if !skills.contains(&skill) {
                    return Err(LevelUpError::InvalidDecision { prompt, decision });
                }
                // TODO: Expertise handling
                systems::helpers::get_component_mut::<SkillSet>(world, entity).set_proficiency(
                    *skill,
                    Proficiency::new(ProficiencyLevel::Proficient, source.clone()),
                );
            }
        }

        (
            LevelUpPrompt::AbilityScores(score_point_cost, num_points),
            LevelUpDecision::AbilityScores(distribution),
        ) => {
            if distribution.scores.len() != Ability::iter().count() {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            if distribution
                .scores
                .values()
                .any(|&score| !score_point_cost.contains_key(&score))
            {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            let total_cost = distribution
                .scores
                .iter()
                .map(|(_, score)| {
                    score_point_cost
                        .get(score)
                        .expect("Invalid ability score")
                        .clone()
                })
                .sum::<u8>();
            if total_cost != *num_points {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            let mut ability_score_set =
                systems::helpers::get_component_mut::<AbilityScoreMap>(world, entity);
            for (ability, score) in &distribution.scores {
                let mut final_score = *score as i32;
                if *ability == distribution.plus_2_bonus {
                    final_score += 2;
                } else if *ability == distribution.plus_1_bonus {
                    final_score += 1;
                }
                ability_score_set.set(*ability, AbilityScore::new(*ability, final_score));
            }
        }

        (LevelUpPrompt::Feat(feats), LevelUpDecision::Feat(feat_id)) => {
            if !feats.contains(&feat_id) {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            let result = systems::feats::add_feat(world, entity, feat_id);
            if result.is_err() {
                eprintln!(
                    "Failed to add feat {}: {:?}",
                    feat_id,
                    result.as_ref().err().unwrap()
                );
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }
            prompts.extend(result.unwrap());
        }

        (
            LevelUpPrompt::AbilityScoreImprovement {
                feat,
                budget,
                abilities,
                max_score,
            },
            LevelUpDecision::AbilityScoreImprovement(decision_points),
        ) => {
            if decision_points.values().sum::<u8>() != *budget {
                return Err(LevelUpError::InvalidDecision { prompt, decision });
            }

            let mut ability_score_set =
                systems::helpers::get_component_mut::<AbilityScoreMap>(world, entity);

            for (ability, bonus) in decision_points {
                if !abilities.contains(ability) {
                    return Err(LevelUpError::InvalidDecision { prompt, decision });
                }
                let current_score = ability_score_set.get(*ability).total() as u8;
                if current_score + bonus > *max_score {
                    return Err(LevelUpError::InvalidDecision { prompt, decision });
                }

                // TODO: Not sure what the best way to apply the points is
                ability_score_set.add_modifier(
                    *ability,
                    // Since some feats are repeatable, we can't use the same source
                    // every time, so we'll have to make it unique
                    ModifierSource::Feat(format!("{}.{}", feat.to_string(), Uuid::new_v4())),
                    *bonus as i32,
                );
            }
        }

        _ => {
            // If the prompt and decision are called the same, and we made it here,
            // it's probably just because it hasn't been implemented yet
            if prompt.name() == decision.name() {
                todo!(
                    "Implement prompt: {:?} with decision: {:?}",
                    prompt,
                    decision
                );
            }

            return Err(LevelUpError::PrompDecisionMismatch { prompt, decision });
        }
    }

    Ok(prompts)
}

fn increment_class_level(world: &mut World, entity: Entity, class: &Class) -> Vec<LevelUpPrompt> {
    let (new_level, subclass) = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        let new_level = character_levels.level_up(class.name.clone());
        let subclass = if let Some(subclass_name) = character_levels.subclass(&class.name) {
            class.subclass(&subclass_name)
        } else {
            None
        };
        (new_level, subclass)
    };

    for ability in class.saving_throw_proficiencies.iter() {
        systems::helpers::get_component_mut::<SavingThrowSet>(world, entity).set_proficiency(
            *ability,
            Proficiency::new(
                ProficiencyLevel::Proficient,
                ModifierSource::ClassFeature(class.name.to_string().clone()),
            ),
        );
    }

    // TODO: If it's a level that triggers a feat prompt, and ability score improvement
    // is selected, then the Constitution modifier might increase, in which case we need to
    // recalculate hit points.
    systems::health::update_hit_points(world, entity);

    systems::spells::update_spell_slots(world, entity);

    let mut prompts = apply_class_base(world, entity, &class.base, new_level);
    if let Some(subclass) = subclass {
        prompts.extend(apply_class_base(world, entity, subclass.base(), new_level));
    }
    prompts
}

fn set_subclass(
    world: &mut World,
    entity: Entity,
    class: &Class,
    subclass_name: SubclassName,
) -> Vec<LevelUpPrompt> {
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
) -> Vec<LevelUpPrompt> {
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
                        .and_modify(|(action_context, action_resource_cost)| {
                            action_context.push(context.clone().unwrap());
                            action_resource_cost.extend(resource_cost.clone());
                        })
                        .or_insert((vec![context.clone().unwrap()], resource_cost.clone()));
                } else {
                    panic!("Action {} not found in registry", action_id);
                }
            }
        }
    }

    // Return any additional prompts that should be presented to the player
    let mut new_prompts = class_base
        .prompts_by_level
        .get(&level)
        .cloned()
        .unwrap_or_default();

    // Some prompts have to be filtered based on the current state of the character
    for prompt in new_prompts.iter_mut() {
        match prompt {
            // Feats need special handling since they can have prerequisites and
            // can (or can't) be repeatable.
            LevelUpPrompt::Feat(feats) => {
                feats.retain(|feat_id| {
                    let feat = registry::feats::FEAT_REGISTRY.get(feat_id).unwrap();
                    if !feat.meets_prerequisite(world, entity) {
                        return false;
                    }
                    if feat.is_repeatable() {
                        return true;
                    }
                    !systems::helpers::get_component::<Vec<FeatId>>(world, entity).contains(feat_id)
                });
            }

            _ => {}
        }
    }

    new_prompts
}

pub fn apply_level_up_decision(
    world: &mut World,
    entity: Entity,
    levels: u8,
    decisions: Vec<LevelUpDecision>,
) {
    let mut decisions = decisions;

    for level in 1..=levels {
        let name = systems::helpers::get_component_clone::<String>(world, entity);
        let mut level_up_session = LevelUpSession::new(world, entity);

        // Some of the responses are identical, e.g. selecting the same class
        // multiple times. Using retain would therefore remove all of them,
        // so we need to track the indices of the used responses.
        let mut used_indices = Vec::new();
        for (i, decision) in decisions.iter().enumerate() {
            let result = level_up_session.advance(world, decision);
            match result {
                Ok(_) | Err(LevelUpError::MissingChoiceForDecision { .. }) => {
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
            decisions.remove(*index);
        }

        if !level_up_session.is_complete() {
            panic!(
                "Level up session for {} at level {} did not complete. Pending prompts: {:?}",
                name,
                level,
                level_up_session.pending_prompts()
            );
        }
    }
}

pub struct LevelUpGains {
    pub hit_points: HitPoints,
    pub actions: Vec<ActionId>,
    pub effects: Vec<EffectId>,
    pub resources: Vec<Resource>,
}

pub fn level_up_gains(
    world: &World,
    entity: Entity,
    class_name: &ClassName,
    level: u8,
) -> LevelUpGains {
    let class = registry::classes::CLASS_REGISTRY
        .get(class_name)
        .expect("Class should exist in the registry");

    let hit_points = systems::helpers::get_component_clone::<HitPoints>(world, entity);
    let mut effects = class
        .base
        .effects_by_level
        .get(&level)
        .cloned()
        .unwrap_or_default();
    let mut resources = class
        .base
        .resources_by_level
        .get(&level)
        .cloned()
        .unwrap_or_default();
    let mut actions = class
        .base
        .actions_by_level
        .get(&level)
        .cloned()
        .unwrap_or_default();

    if let Some(subclass) =
        systems::helpers::get_component::<CharacterLevels>(world, entity).subclass(class_name)
    {
        if let Some(subclass) = class.subclass(&subclass) {
            if let Some(subclass_effects) = subclass.base.effects_by_level.get(&level) {
                effects.extend(subclass_effects.clone());
            }
            if let Some(subclass_resources) = subclass.base.resources_by_level.get(&level) {
                resources.extend(subclass_resources.clone());
            }
            if let Some(subclass_actions) = subclass.base.actions_by_level.get(&level) {
                actions.extend(subclass_actions.clone());
            }
        }
    }

    LevelUpGains {
        hit_points,
        actions,
        effects,
        resources,
    }
}
