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
    scripts::script::ScriptFunction,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SpellDefinition {
    pub id: SpellId,
    pub description: String,
    pub base_level: u8,
    pub school: MagicSchool,
    #[serde(default)]
    pub concentration: bool,
    pub kind: ActionKindDefinition,
    pub resource_cost: ResourceAmountMap,
    pub targeting: TargetingDefinition,
    #[serde(default)]
    pub reaction_trigger: Option<ScriptId>,
    /// TODO: Is there a better way to represent this?
    ///
    /// Some spells like Hex or Hunter's Mark grant an alternative version of themselves
    /// which doesn't cost a spell slot, but can only be cast under certain conditions.
    /// It seems like easiest way is just to grant those alternative spells directly,
    /// though they won't be able to be used until the conditions are met, e.g. killing
    /// an enemy marked by the original spell.
    #[serde(default)]
    pub granted_spells: Vec<(SpellId, u8)>,
}

impl From<SpellDefinition> for Spell {
    fn from(value: SpellDefinition) -> Self {
        Spell::new(
            value.id,
            value.description,
            value.base_level,
            value.school,
            value.concentration,
            value.kind.into(),
            value.resource_cost,
            value.targeting.function(),
            value.reaction_trigger,
            value.granted_spells,
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
            collector.add(RegistryReference::Script(
                script_id.clone(),
                ScriptFunction::ReactionTrigger,
            ));
        }
    }
}
