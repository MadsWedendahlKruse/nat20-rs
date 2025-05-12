use crate::stats::modifier::ModifierSource;

pub enum EffectDuration {
    Instant,
    Temporary(usize),
    Persistent,
}

pub trait Effect {
    // fn id(&self) -> EffectId;
    // fn source(&self) -> ModifierSource;
    // fn duration(&self) -> DurationKind;

    // // These are like hooks into the engine lifecycle:
    // fn on_attack(&self, attacker: &mut Character, damage: &mut DamageRoll);
    // fn on_turn_start(&self, character: &mut Character);
    // fn is_expired(&self, turn: usize) -> bool;
}
