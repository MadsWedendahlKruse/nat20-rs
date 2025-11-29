use serde::{Deserialize, Serialize};

use crate::components::{
    ability::Ability,
    id::{BackgroundId, FeatId},
    level_up::ChoiceSpec,
    skill::Skill,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Background {
    pub id: BackgroundId,
    // TODO: Not sure if we want to use this?
    pub ability_scores: [Ability; 3],
    pub feat: FeatId,
    pub skill_proficiencies: [Skill; 2],
    // TODO: Not sure what to do with these yet
    // tool_proficiencies
    pub equipment: ChoiceSpec,
}
