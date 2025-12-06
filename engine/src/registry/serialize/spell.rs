use serde::{Deserialize, Serialize};

use crate::{
    components::{
        id::SpellId,
        resource::ResourceAmountMap,
        spells::spell::{MagicSchool, Spell},
    },
    registry::serialize::{action::ActionKindDefinition, targeting::TargetingDefinition},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SpellDefinition {
    pub id: SpellId,
    pub description: String,
    pub base_level: u8,
    pub school: MagicSchool,
    pub kind: ActionKindDefinition,
    pub resource_cost: ResourceAmountMap,
    pub targeting: TargetingDefinition,
    // TODO: How to handle reaction triggers in serialization?
    // pub reaction_trigger: Option<Arc<dyn Fn(Entity, &Event) -> bool + Send + Sync>>,
}

impl From<SpellDefinition> for Spell {
    fn from(value: SpellDefinition) -> Self {
        Spell::new(
            value.id,
            value.description,
            value.base_level,
            value.school,
            value.kind.into(),
            value.resource_cost,
            value.targeting.function(),
            None,
        )
    }
}
