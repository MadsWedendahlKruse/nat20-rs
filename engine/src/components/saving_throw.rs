use std::fmt::Display;

use hecs::{Entity, World};
use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::Ability,
        d20::{D20CheckDC, D20CheckSet},
        effects::hooks::D20CheckHooks,
    },
    systems::{self},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SavingThrowKind {
    Ability(Ability),
    Death, // sounds a bit edgy lol
}

impl IntoEnumIterator for SavingThrowKind {
    type Iterator = std::vec::IntoIter<SavingThrowKind>;

    fn iter() -> Self::Iterator {
        let mut v = Vec::with_capacity(1 + Ability::iter().len());
        v.push(SavingThrowKind::Death);
        v.extend(Ability::iter().map(SavingThrowKind::Ability));
        v.into_iter()
    }
}

impl Display for SavingThrowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SavingThrowKind::Ability(ability) => write!(f, "{}", ability),
            SavingThrowKind::Death => write!(f, "Death"),
        }
    }
}

pub type SavingThrowSet = D20CheckSet<SavingThrowKind>;

pub type SavingThrowDC = D20CheckDC<SavingThrowKind>;

pub fn get_saving_throw_hooks(
    kind: SavingThrowKind,
    world: &World,
    entity: Entity,
) -> Vec<D20CheckHooks> {
    systems::effects::effects(world, entity)
        .iter()
        .filter_map(|e| e.on_saving_throw.get(&kind))
        .cloned()
        .collect()
}

impl Default for SavingThrowSet {
    fn default() -> Self {
        Self::new(
            |kind| match kind {
                SavingThrowKind::Ability(ability) => Some(ability),
                SavingThrowKind::Death => None,
            },
            get_saving_throw_hooks,
        )
    }
}
