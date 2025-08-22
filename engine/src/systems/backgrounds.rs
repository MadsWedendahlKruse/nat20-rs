use hecs::{Entity, World};

use crate::{
    components::{
        background::{self, Background},
        id::BackgroundId,
        level_up::LevelUpPrompt,
        modifier::ModifierSource,
        proficiency::{Proficiency, ProficiencyLevel},
        skill::SkillSet,
    },
    registry, systems,
};

pub fn background(world: &World, entity: Entity) -> hecs::Ref<'_, BackgroundId> {
    systems::helpers::get_component::<BackgroundId>(world, entity)
}

pub fn background_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, BackgroundId> {
    systems::helpers::get_component_mut::<BackgroundId>(world, entity)
}

pub fn set_background(
    world: &mut World,
    entity: Entity,
    background_id: &BackgroundId,
) -> Vec<LevelUpPrompt> {
    let background = registry::backgrounds::BACKGROUND_REGISTRY
        .get(background_id)
        .expect(&format!(
            "Background with ID `{}` not found in the registry",
            background_id
        ));

    *background_mut(world, entity) = background_id.clone();

    let feat_result = systems::feats::add_feat(world, entity, background.feat());
    if let Err(e) = feat_result {
        // TODO: Not sure what to do here
        panic!("Error adding background feat: {:?}", e);
    }
    let mut prompts = feat_result.unwrap();

    let mut skill_set = systems::helpers::get_component_mut::<SkillSet>(world, entity);
    for skill in background.skill_proficiencies() {
        skill_set.set_proficiency(
            *skill,
            Proficiency::new(
                ProficiencyLevel::Proficient,
                ModifierSource::Background(background_id.clone()),
            ),
        );
    }

    // Set tool proficiencies

    // Set languages

    prompts.push(LevelUpPrompt::Choice(background.equipment().clone()));

    prompts
}
