use serde::{Deserialize, Serialize};

use crate::{
    components::{
        id::{ScriptId, SpellId},
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
    #[serde(default)]
    pub reaction_trigger: Option<ScriptId>,
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
            value.reaction_trigger,
        )
    }
}
