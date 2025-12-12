use hecs::{Entity, World};

use crate::{
    components::{
        id::{EffectId, RaceId, SubraceId},
        level_up::{ChoiceItem, ChoiceSpec, LevelUpPrompt},
        modifier::ModifierSource,
        race::{CreatureSize, CreatureType, RaceBase},
        speed::Speed,
    },
    registry, systems,
};

pub enum RaceIdentifier {
    Race(RaceId),
    Subrace(SubraceId),
}

impl RaceIdentifier {
    pub fn modifier_source(&self) -> ModifierSource {
        match self {
            RaceIdentifier::Race(id) => ModifierSource::Race(id.clone()),
            RaceIdentifier::Subrace(id) => ModifierSource::Subrace(id.clone()),
        }
    }
}

pub fn set_race(world: &mut World, entity: Entity, race: &RaceId) -> Vec<LevelUpPrompt> {
    let mut prompts = Vec::new();

    let race = registry::races::RACE_REGISTRY.get(&race).expect(&format!(
        "Race with ID `{}` not found in the registry",
        race
    ));

    systems::helpers::set_component::<RaceId>(world, entity, race.id.clone());

    // TODO: The race is presumably always set at level 1?
    apply_race_base(
        world,
        entity,
        &race.base,
        RaceIdentifier::Race(race.id.clone()),
        1,
    );

    if !race.subraces.is_empty() {
        prompts.push(LevelUpPrompt::Choice(ChoiceSpec::single(
            "Subrace",
            race.subraces
                .keys()
                .cloned()
                .map(ChoiceItem::Subrace)
                .collect(),
        )));
    }

    systems::helpers::set_component::<CreatureSize>(world, entity, race.size.clone());
    systems::helpers::set_component::<CreatureType>(world, entity, race.creature_type.clone());
    systems::helpers::set_component::<Speed>(world, entity, race.speed.clone());

    prompts
}

pub fn set_subrace(world: &mut World, entity: Entity, subrace: &SubraceId) {
    let race_id = systems::helpers::get_component_clone::<RaceId>(world, entity);

    let race = registry::races::RACE_REGISTRY
        .get(&race_id)
        .expect(&format!(
            "Race with ID `{}` not found in the registry",
            race_id
        ));

    let subrace = race.subraces.get(&subrace).expect(&format!(
        "Race `{}` does not have subrace `{}`",
        race_id, subrace
    ));

    systems::helpers::set_component::<Option<SubraceId>>(world, entity, Some(subrace.id.clone()));

    // TODO: Always level 1?
    apply_race_base(
        world,
        entity,
        &subrace.base,
        RaceIdentifier::Subrace(subrace.id.clone()),
        1,
    );
}

fn apply_race_base(
    world: &mut World,
    entity: Entity,
    base: &RaceBase,
    id: RaceIdentifier,
    level: u8,
) {
    if let Some(effects) = base.effects_by_level.get(&level) {
        systems::effects::add_effects(world, entity, effects, &id.modifier_source());
    }
    if let Some(actions) = base.actions_by_level.get(&level) {
        systems::actions::add_actions(world, entity, actions);
    }
}
