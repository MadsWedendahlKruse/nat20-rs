use serde::{Deserialize, Serialize};

use crate::{
    components::{
        id::{ScriptId, SpellId},
        resource::ResourceAmountMap,
        spells::spell::{MagicSchool, Spell},
    },
    registry::{
        registry_validation::{ReferenceCollector, RegistryReference, RegistryReferenceCollector},
        serialize::{action::ActionKindDefinition, targeting::TargetingDefinition},
    },
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

impl RegistryReferenceCollector for SpellDefinition {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        self.kind.collect_registry_references(collector);
        for resource in self.resource_cost.keys() {
            collector.add(RegistryReference::Resource(resource.clone()));
        }
        if let Some(script_id) = &self.reaction_trigger {
            collector.add(RegistryReference::Script(script_id.clone()));
        }
    }
}
