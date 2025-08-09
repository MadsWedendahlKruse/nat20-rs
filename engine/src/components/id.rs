use std::fmt;

use hecs::Entity;
use uuid::Uuid;

pub type ItemId = Uuid;

pub type EncounterId = Uuid;

macro_rules! id_newtypes {
    ($($name:ident),+) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name(String);

            impl $name {
                pub fn from_str(s: impl Into<String>) -> Self {
                    $name(s.into())
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
        )+
    };
}

id_newtypes!(EffectId, ResourceId, ActionId, SpellId, FeatId);

impl SpellId {
    pub fn to_action_id(&self) -> ActionId {
        ActionId::from_str(&self.0)
    }
}

impl ActionId {
    pub fn to_spell_id(&self) -> SpellId {
        SpellId::from_str(&self.0)
    }
}

/// Identifier for an entity in the game world.
/// This is used to uniquely identify entities, such as characters or creatures.
/// In most cases the id (`Entity` ) is meaningless outside the context of the
/// world, so for convenience we also store the name of the entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityIdentifier {
    id: Entity,
    name: String,
}

impl EntityIdentifier {
    pub fn new(id: Entity, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn id(&self) -> Entity {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn from_world(world: &hecs::World, entity: Entity) -> Self {
        let name = world
            .get::<&String>(entity)
            .map(|name| name.to_string())
            .unwrap_or_else(|_| "Unnamed Entity".to_string());
        Self::new(entity, name)
    }
}
