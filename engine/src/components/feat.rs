use std::sync::Arc;

use hecs::{Entity, World};

use crate::components::{effects::effects::Effect, id::FeatId, level_up::LevelUpPrompt};

#[derive(Clone)]
pub struct Feat {
    id: FeatId,
    pre_requisites: Option<Arc<dyn Fn(&World, Entity) -> bool + Send + Sync>>,
    effects: Vec<Effect>,
    /// Some feats might require a choice to be made when selected.
    /// In most cases this will be some kind of ability score increase, but could
    /// also be a choice between learning a new spell etc.
    // TODO: Is it ever more than one?
    prompts: Vec<LevelUpPrompt>,
    /// Most feats are single-use, but some can be taken multiple times.
    /// This mostly applies to Ability Score Improvement.
    repeatable: bool,
}

impl Feat {
    pub fn new(
        id: FeatId,
        pre_requisites: Option<Arc<dyn Fn(&World, Entity) -> bool + Send + Sync>>,
        effects: Vec<Effect>,
        prompts: Vec<LevelUpPrompt>,
        repeatable: bool,
    ) -> Self {
        Self {
            id,
            pre_requisites,
            effects,
            prompts,
            repeatable,
        }
    }

    pub fn id(&self) -> &FeatId {
        &self.id
    }

    pub fn meets_pre_requisites(&self, world: &World, entity: Entity) -> bool {
        if let Some(pre_requisites) = &self.pre_requisites {
            pre_requisites(world, entity)
        } else {
            true
        }
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn prompts(&self) -> &[LevelUpPrompt] {
        &self.prompts
    }

    pub fn is_repeatable(&self) -> bool {
        self.repeatable
    }
}
