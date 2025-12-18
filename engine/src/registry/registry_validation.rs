use std::collections::HashMap;

use crate::{
    components::{
        background::Background,
        faction::Faction,
        feat::Feat,
        id::{
            ActionId, BackgroundId, ClassId, EffectId, FactionId, FeatId, ItemId, ResourceId,
            ScriptId, SpeciesId, SpellId, SubclassId, SubspeciesId,
        },
        resource::Resource,
    },
    scripts::script::ScriptFunction,
};

#[derive(Debug, Clone)]
pub enum RegistryReference {
    Action(ActionId),
    Background(BackgroundId),
    Class(ClassId),
    Effect(EffectId),
    Faction(FactionId),
    Feat(FeatId),
    Item(ItemId),
    Resource(ResourceId),
    Script(ScriptId, ScriptFunction),
    Species(SpeciesId),
    Spell(SpellId),
    Subclass(SubclassId),
    Subspecies(SubspeciesId),
}

#[derive(Debug, Default)]
pub struct ReferenceCollector {
    pub references: Vec<RegistryReference>,
}

impl ReferenceCollector {
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
        }
    }

    pub fn into_references(self) -> Vec<RegistryReference> {
        self.references
    }
}

impl ReferenceCollector {
    pub fn add(&mut self, reference: RegistryReference) {
        self.references.push(reference);
    }
}

pub trait RegistryReferenceCollector {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector);
}

impl<T: RegistryReferenceCollector> RegistryReferenceCollector for Vec<T> {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        for item in self {
            item.collect_registry_references(collector);
        }
    }
}

impl<T: RegistryReferenceCollector> RegistryReferenceCollector for Option<T> {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        if let Some(value) = self {
            value.collect_registry_references(collector);
        }
    }
}

impl<K, V> RegistryReferenceCollector for HashMap<K, V>
where
    V: RegistryReferenceCollector,
{
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        for value in self.values() {
            value.collect_registry_references(collector);
        }
    }
}

// I don't know where to put these, so I'm hiding them in here :^)

impl RegistryReferenceCollector for Background {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        collector.add(RegistryReference::Feat(self.feat.clone()));
        self.equipment.collect_registry_references(collector);
    }
}

impl RegistryReferenceCollector for Faction {
    fn collect_registry_references(&self, _collector: &mut ReferenceCollector) {
        // Factions currently have no registry references
    }
}

impl RegistryReferenceCollector for Feat {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        for effect in self.effects() {
            collector.add(RegistryReference::Effect(effect.clone()));
        }
        for prompt in self.prompts() {
            prompt.collect_registry_references(collector);
        }
    }
}

impl RegistryReferenceCollector for Resource {
    fn collect_registry_references(&self, _collector: &mut ReferenceCollector) {
        // Resources currently have no registry references
    }
}
