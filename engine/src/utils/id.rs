use std::fmt;

use uuid::Uuid;

pub type CharacterId = Uuid;

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

id_newtypes!(EffectId, ResourceId, ActionId, SpellId);
