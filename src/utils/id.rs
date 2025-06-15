use std::fmt;

use uuid::Uuid;

pub type CharacterId = Uuid;

pub type ItemId = Uuid;

// pub type SpellId = String;

macro_rules! id_newtype {
    ($name:ident) => {
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
    };
}

id_newtype!(EffectId);
id_newtype!(ResourceId);
id_newtype!(ActionId);
id_newtype!(SpellId);
