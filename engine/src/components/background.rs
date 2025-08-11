use crate::components::{
    ability::Ability,
    id::{BackgroundId, FeatId},
    skill::Skill,
};

#[derive(Debug, Clone)]
pub struct Background {
    id: BackgroundId,
    // TODO: Not sure if we want to use this?
    ability_scores: [Ability; 3],
    feat: FeatId,
    skill_proficiencies: [Skill; 2],
    // TODO: Not sure what to do with these yet
    // tool_proficiencies
    // TODO: Include equipment here or should it be based on the class?
    // equipment:
}

impl Background {
    pub fn new(
        id: BackgroundId,
        ability_scores: [Ability; 3],
        feat: FeatId,
        skill_proficiencies: [Skill; 2],
    ) -> Self {
        Self {
            id,
            ability_scores,
            feat,
            skill_proficiencies,
        }
    }

    pub fn id(&self) -> &BackgroundId {
        &self.id
    }

    pub fn ability_scores(&self) -> &[Ability; 3] {
        &self.ability_scores
    }

    pub fn feat(&self) -> &FeatId {
        &self.feat
    }

    pub fn skill_proficiencies(&self) -> &[Skill; 2] {
        &self.skill_proficiencies
    }
}
