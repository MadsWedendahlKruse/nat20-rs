use std::sync::Arc;

use crate::{
    creature::character::Character,
    stats::d20_check::{D20Check, D20CheckResult},
};

pub trait CloneableFn: Fn(&mut Character) + Send + Sync {}
impl<T> CloneableFn for T where T: Fn(&mut Character) + Send + Sync {}

pub type EffectHook = Arc<dyn CloneableFn>;
pub type AttackPreHook = Arc<dyn Fn(&Character, &mut D20Check) + Send + Sync>;
pub type AttackPostHook = Arc<dyn Fn(&Character, &mut D20CheckResult) + Send + Sync>;
