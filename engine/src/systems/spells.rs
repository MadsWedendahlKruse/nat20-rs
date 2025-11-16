use std::{collections::HashMap, sync::LazyLock};

use hecs::{Entity, World};

use crate::{
    components::{class::SpellcastingProgression, level::CharacterLevels, resource::ResourceMap},
    registry,
};

pub fn spellcaster_levels(world: &World, entity: Entity) -> u8 {
    let mut spellcaster_levels = 0.0;
    if let Ok(class_levels) = world.get::<&CharacterLevels>(entity) {
        for (class_id, level_progression) in class_levels.all_classes() {
            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&class_id) {
                let spellcasting_progression = class.spellcasting_progression(
                    // TODO: Not entirely sure why it's necessary to do it like this
                    level_progression.subclass(),
                );

                let level = level_progression.level() as f32;

                spellcaster_levels += match spellcasting_progression {
                    SpellcastingProgression::None => 0.0,
                    SpellcastingProgression::Full => level,
                    SpellcastingProgression::Half => level / 2.0,
                    SpellcastingProgression::Third => level / 3.0,
                };
            }
        }
    }
    spellcaster_levels as u8
}

pub static MAX_SPELL_LEVEL: u8 = 9;

static SPELL_SLOTS_PER_LEVEL: LazyLock<HashMap<u8, Vec<u8>>> = LazyLock::new(|| {
    HashMap::from([
        (0, vec![]),
        (1, vec![2]),
        (2, vec![3]),
        (3, vec![4, 2]),
        (4, vec![4, 3]),
        (5, vec![4, 3, 2]),
        (6, vec![4, 3, 3]),
        (7, vec![4, 3, 3, 1]),
        (8, vec![4, 3, 3, 2]),
        (9, vec![4, 3, 3, 3, 1]),
        (10, vec![4, 3, 3, 3, 2]),
        (11, vec![4, 3, 3, 3, 2, 1]),
        (12, vec![4, 3, 3, 3, 2, 1]),
        (13, vec![4, 3, 3, 3, 2, 1, 1]),
        (14, vec![4, 3, 3, 3, 2, 1, 1]),
        (15, vec![4, 3, 3, 3, 2, 1, 1, 1]),
        (16, vec![4, 3, 3, 3, 2, 1, 1, 1]),
        (17, vec![4, 3, 3, 3, 2, 1, 1, 1, 1]),
        (18, vec![4, 3, 3, 3, 2, 1, 1, 1, 1]),
        (19, vec![4, 3, 3, 3, 3, 2, 1, 1, 1]),
        (20, vec![4, 3, 3, 3, 3, 2, 2, 1, 1]),
    ])
});

pub fn update_spell_slots(world: &mut World, entity: Entity) {
    if let Ok(mut resources) = world.get::<&mut ResourceMap>(entity) {
        let spellcaster_levels = spellcaster_levels(world, entity);
        let slots_vec = SPELL_SLOTS_PER_LEVEL.get(&spellcaster_levels).unwrap();
        for (level, &num_slots) in slots_vec.iter().enumerate() {
            let level = level as u8 + 1;
            resources.add(
                registry::resources::SPELL_SLOT.build_resource(level, num_slots),
                false,
            );
        }
    }
}
