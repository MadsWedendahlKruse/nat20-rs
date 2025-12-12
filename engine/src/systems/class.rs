use crate::{
    components::{
        class::ClassBase,
        id::{ClassId, FeatId, SubclassId},
        items::equipment::{armor::ArmorTrainingSet, weapon::WeaponProficiencyMap},
        level_up::ChoiceItem,
        proficiency::ProficiencyLevel,
        resource::ResourceMap,
        saving_throw::SavingThrowKind,
    },
    registry::{self, registry::ClassesRegistry},
};
use hecs::{Entity, World};

use crate::{
    components::{
        level::CharacterLevels, level_up::LevelUpPrompt, modifier::ModifierSource,
        proficiency::Proficiency, saving_throw::SavingThrowSet,
    },
    systems,
};

enum ClassIdentifier {
    Class(ClassId),
    Subclass(SubclassId),
}

impl ClassIdentifier {
    pub fn modifier_source(&self) -> ModifierSource {
        match self {
            ClassIdentifier::Class(class_id) => ModifierSource::ClassFeature(class_id.clone()),
            ClassIdentifier::Subclass(subclass_id) => {
                ModifierSource::SubclassFeature(subclass_id.clone())
            }
        }
    }
}

pub fn class_level(world: &World, entity: Entity, class_id: &ClassId) -> u8 {
    if let Ok(character_levels) = world.get::<&CharacterLevels>(entity) {
        if let Some(class_level) = character_levels.class_level(class_id) {
            return class_level.level();
        }
    }
    0
}

pub fn increment_class_level(
    world: &mut World,
    entity: Entity,
    class_id: &ClassId,
) -> Vec<LevelUpPrompt> {
    let class = ClassesRegistry::get(class_id).expect(&format!(
        "Class with name `{}` not found in the registry",
        class_id
    ));

    let (new_level, subclass) = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        let new_level = character_levels.level_up(class_id.clone());
        let subclass = if let Some(subclass_id) = character_levels.subclass(&class_id) {
            class.subclass(&subclass_id)
        } else {
            None
        };
        (new_level, subclass)
    };

    for ability in class.saving_throw_proficiencies.iter() {
        systems::helpers::get_component_mut::<SavingThrowSet>(world, entity).set_proficiency(
            SavingThrowKind::Ability(*ability),
            Proficiency::new(
                ProficiencyLevel::Proficient,
                ModifierSource::ClassFeature(class_id.clone()),
            ),
        );
    }

    // TODO: If it's a level that triggers a feat prompt, and ability score improvement
    // is selected, then the Constitution modifier might increase, in which case we need to
    // recalculate hit points.
    systems::health::update_hit_points(world, entity);

    systems::spells::update_spell_slots(world, entity);

    let mut prompts = apply_class_base(
        world,
        entity,
        &class.base,
        ClassIdentifier::Class(class_id.clone()),
        new_level,
    );
    if let Some(subclass) = subclass {
        prompts.extend(apply_class_base(
            world,
            entity,
            subclass.base(),
            ClassIdentifier::Subclass(subclass.id.clone()),
            new_level,
        ));
    }
    prompts
}

pub fn set_subclass(
    world: &mut World,
    entity: Entity,
    subclass_id: &SubclassId,
) -> Vec<LevelUpPrompt> {
    let class_name = subclass_id.as_str().split(".").collect::<Vec<_>>()[1];
    let class_id = &ClassId::from_str(format!("class.{}", class_name));

    let class = ClassesRegistry::get(class_id).expect(&format!(
        "Class with name `{}` not found in the registry",
        class_id
    ));

    let (subclass, level) = {
        let mut character_levels =
            systems::helpers::get_component_mut::<CharacterLevels>(world, entity);
        character_levels.set_subclass(class_id, &subclass_id);

        let subclass = class
            .subclass(&subclass_id)
            .expect("Subclass should exist in the class registry");
        let level = character_levels.class_level(class_id).unwrap().level();

        (subclass, level)
    };

    apply_class_base(
        world,
        entity,
        subclass.base(),
        ClassIdentifier::Subclass(subclass_id.clone()),
        level,
    )
}

fn apply_class_base(
    world: &mut World,
    entity: Entity,
    class_base: &ClassBase,
    id: ClassIdentifier,
    level: u8,
) -> Vec<LevelUpPrompt> {
    // Effect
    if let Some(effects_for_level) = class_base.effects_by_level.get(&level) {
        for effect in effects_for_level {
            systems::effects::add_effect(world, entity, effect, &id.modifier_source());
        }
    }

    // Resources
    {
        let mut resources = systems::helpers::get_component_mut::<ResourceMap>(world, entity);
        if let Some(resources_for_level) = class_base.resources_by_level.get(&level) {
            for (resource, amount) in resources_for_level {
                resources.add(resource.clone(), amount.clone().into(), false);
            }
        }
    }

    // Actions
    {
        if let Some(actions_for_level) = class_base.actions_by_level.get(&level) {
            systems::actions::add_actions(world, entity, actions_for_level);
        }
    }

    // Weapons proficiencies
    {
        let mut weapon_proficiencies =
            systems::helpers::get_component_mut::<WeaponProficiencyMap>(world, entity);
        for proficiency in class_base.weapon_proficiencies.iter() {
            weapon_proficiencies.set_proficiency(
                proficiency.clone(),
                Proficiency::new(ProficiencyLevel::Proficient, id.modifier_source()),
            );
        }
    }

    // Armor training
    {
        let mut armor_training =
            systems::helpers::get_component_mut::<ArmorTrainingSet>(world, entity);
        for armor_type in class_base.armor_proficiencies.iter() {
            armor_training.insert(armor_type.clone());
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
            LevelUpPrompt::Choice(choice_spec) => {
                choice_spec.options.retain(|item| match item {
                    ChoiceItem::Feat(feat_id) => {
                        let feat = registry::feats::FEAT_REGISTRY.get(feat_id).unwrap();
                        if !feat.meets_prerequisite(world, entity) {
                            return false;
                        }
                        if feat.is_repeatable() {
                            return true;
                        }
                        !systems::helpers::get_component::<Vec<FeatId>>(world, entity)
                            .contains(feat_id)
                    }
                    _ => true,
                });
            }

            _ => {}
        }
    }

    new_prompts
}
