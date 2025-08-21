use std::{
    fmt::{self, Display},
    ops::Deref,
};

use hecs::Entity;
use uuid::Uuid;

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

                pub fn as_str(&self) -> &str {
                    &self.0
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

id_newtypes!(
    ItemId,
    EffectId,
    ResourceId,
    ActionId,
    SpellId,
    FeatId,
    BackgroundId,
    RaceId,
    SubraceId
);

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

// TODO: Not sure if this is the best place for this
/// Name is a simple wrapper around a String to provide a type-safe way to
/// handle names when querying entities in the game world. The alternative is to
/// use a String directly, but a String can be ambiguous in terms of what it
/// represents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn to_string_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Identifier for an entity in the game world.
/// This is used to uniquely identify entities, such as characters or creatures.
/// In most cases the id (`Entity`) is meaningless outside the context of the
/// world, so for convenience we also store the name of the entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityIdentifier {
    id: Entity,
    name: Name,
}

impl EntityIdentifier {
    pub fn new(id: Entity, name: Name) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> Entity {
        self.id
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn from_world(world: &hecs::World, entity: Entity) -> Self {
        let name = world
            .get::<&Name>(entity)
            .map(|name| name.deref().clone())
            .unwrap_or_else(|_| Name::new("Unnamed Entity"));
        Self::new(entity, name)
    }
}
