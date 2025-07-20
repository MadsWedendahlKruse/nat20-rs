use hecs::{Entity, World};

use crate::{
    components::{
        class::SpellcastingProgression, level::CharacterLevels, spells::spellbook::Spellbook,
    },
    registry,
};

pub fn spellcaster_levels(world: &World, entity: Entity) -> u8 {
    let mut spellcaster_levels = 0.0;
    if let Ok(class_levels) = world.get::<&CharacterLevels>(entity) {
        for (class_name, level_progression) in class_levels.all_classes() {
            if let Some(class) = registry::classes::CLASS_REGISTRY.get(&class_name) {
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

pub fn update_spell_slots(world: &mut World, entity: Entity) {
    if let Ok(mut spellbook) = world.get::<&mut Spellbook>(entity) {
        let spellcaster_levels = spellcaster_levels(world, entity);
        spellbook.update_spell_slots(spellcaster_levels);
    }
}
