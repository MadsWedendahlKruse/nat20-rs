use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    components::{
        ability::Ability, feat::Feat, id::FeatId, level::CharacterLevels, level_up::LevelUpPrompt,
    },
    registry, systems,
};

pub static FEAT_REGISTRY: LazyLock<HashMap<FeatId, Feat>> = LazyLock::new(|| {
    HashMap::from([
        (
            ABILITY_SCORE_IMPROVEMENT_ID.to_owned(),
            ABILITY_SCORE_IMPROVEMENT.to_owned(),
        ),
        (
            FIGHTING_STYLE_ARCHERY_ID.to_owned(),
            FIGHTING_STYLE_ARCHERY.to_owned(),
        ),
        (
            FIGHTING_STYLE_DEFENSE_ID.to_owned(),
            FIGHTING_STYLE_DEFENSE.to_owned(),
        ),
        (
            FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.to_owned(),
            FIGHTING_STYLE_GREAT_WEAPON_FIGHTING.to_owned(),
        ),
    ])
});

pub static ABILITY_SCORE_IMPROVEMENT_ID: LazyLock<FeatId> =
    LazyLock::new(|| FeatId::from_str("feat.ability_score_improvement"));

pub static ABILITY_SCORE_IMPROVEMENT: LazyLock<Feat> = LazyLock::new(|| {
    Feat::new(
        ABILITY_SCORE_IMPROVEMENT_ID.clone(),
        Some(Arc::new(|world, entity| {
            systems::helpers::get_component::<CharacterLevels>(world, entity).total_level() >= 4
        })),
        vec![],
        vec![LevelUpPrompt::AbilityScoreImprovement {
            feat: ABILITY_SCORE_IMPROVEMENT_ID.clone(),
            budget: 2,
            abilities: Ability::set(),
            max_score: 20,
        }],
        true,
    )
});

pub static FIGHTING_STYLE_ARCHERY_ID: LazyLock<FeatId> =
    LazyLock::new(|| FeatId::from_str("feat.fighting_style.archery"));

pub static FIGHTING_STYLE_ARCHERY: LazyLock<Feat> = LazyLock::new(|| {
    Feat::new(
        FIGHTING_STYLE_ARCHERY_ID.clone(),
        None,
        vec![registry::effects::FIGHTING_STYLE_ARCHERY_ID.clone()],
        vec![],
        false,
    )
});

pub static FIGHTING_STYLE_DEFENSE_ID: LazyLock<FeatId> =
    LazyLock::new(|| FeatId::from_str("feat.fighting_style.defense"));

pub static FIGHTING_STYLE_DEFENSE: LazyLock<Feat> = LazyLock::new(|| {
    Feat::new(
        FIGHTING_STYLE_DEFENSE_ID.clone(),
        None,
        vec![registry::effects::FIGHTING_STYLE_DEFENSE_ID.clone()],
        vec![],
        false,
    )
});

pub static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID: LazyLock<FeatId> =
    LazyLock::new(|| FeatId::from_str("feat.fighting_style.great_weapon_fighting"));

pub static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING: LazyLock<Feat> = LazyLock::new(|| {
    Feat::new(
        FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
        None,
        vec![registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone()],
        vec![],
        false,
    )
});
