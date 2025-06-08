use std::fmt;

use uuid::Uuid;

pub type CharacterId = Uuid;

pub type ItemId = Uuid;

pub type SpellId = String;

pub type RegistryKey = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectId(String);

impl EffectId {
    pub fn from_str(s: impl Into<String>) -> Self {
        EffectId(s.into())
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
