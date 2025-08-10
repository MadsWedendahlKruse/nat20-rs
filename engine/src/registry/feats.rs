use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::{
    components::{
        ability::Ability, feat::Feat, id::FeatId, level::CharacterLevels, level_up::LevelUpPrompt,
    },
    systems,
};

pub static FEAT_REGISTRY: LazyLock<HashMap<FeatId, Feat>> = LazyLock::new(|| {
    HashMap::from([(
        ABILITY_SCORE_IMPROVEMENT_ID.to_owned(),
        ABILITY_SCORE_IMPROVEMENT.to_owned(),
    )])
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
