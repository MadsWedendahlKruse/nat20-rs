use hecs::{Entity, World};

use crate::{
    components::{
        background::Background,
        id::BackgroundId,
        level_up::LevelUpPrompt,
        modifier::ModifierSource,
        proficiency::{Proficiency, ProficiencyLevel},
        skill::SkillSet,
    },
    systems,
};

pub fn background(world: &World, entity: Entity) -> hecs::Ref<'_, Option<BackgroundId>> {
    systems::helpers::get_component::<Option<BackgroundId>>(world, entity)
}

pub fn background_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Option<BackgroundId>> {
    systems::helpers::get_component_mut::<Option<BackgroundId>>(world, entity)
}

pub fn set_background(
    world: &mut World,
    entity: Entity,
    background: &Background,
) -> Vec<LevelUpPrompt> {
    *background_mut(world, entity) = Some(background.id().clone());

    let feat_result = systems::feats::add_feat(world, entity, background.feat());
    if let Err(e) = feat_result {
        // TODO: Not sure what to do here
        panic!("Error adding background feat: {:?}", e);
    }
    let prompts = feat_result.unwrap();

    let mut skill_set = systems::helpers::get_component_mut::<SkillSet>(world, entity);
    for skill in background.skill_proficiencies() {
        skill_set.set_proficiency(
            *skill,
            Proficiency::new(
                ProficiencyLevel::Proficient,
                ModifierSource::Background(background.id().clone()),
            ),
        );
    }

    // Set tool proficiencies

    // Set languages

    prompts
}
