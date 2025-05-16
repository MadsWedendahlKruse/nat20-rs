use std::{fmt, sync::Arc};

use crate::{
    creature::character::Character,
    stats::{
        ability::Ability,
        d20_check::{D20Check, D20CheckResult},
        skill::Skill,
    },
};

pub type EffectHook = Arc<dyn Fn(&mut Character) + Send + Sync>;
pub type D20CheckHook = Arc<dyn Fn(&Character, &mut D20Check) + Send + Sync>;
pub type D20CheckResultHook = Arc<dyn Fn(&Character, &mut D20CheckResult) + Send + Sync>;

#[derive(Clone)]
pub struct D20CheckHooks<K> {
    pub key: K,
    pub check_hook: D20CheckHook,
    pub result_hook: D20CheckResultHook,
}

impl<K: fmt::Debug> fmt::Debug for D20CheckHooks<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("D20CheckHookPair")
            .field("key", &self.key)
            .field("check_hook", &"<Fn>")
            .field("result_hook", &"<Fn>")
            .finish()
    }
}

impl<K> D20CheckHooks<K> {
    pub fn new(key: K) -> Self {
        use std::sync::LazyLock;
        static NOOP_CHECK: LazyLock<D20CheckHook> = LazyLock::new(|| Arc::new(|_, _| {}));
        static NOOP_RESULT: LazyLock<D20CheckResultHook> = LazyLock::new(|| Arc::new(|_, _| {}));

        Self {
            key,
            check_hook: NOOP_CHECK.clone(),
            result_hook: NOOP_RESULT.clone(),
        }
    }
}

pub type SkillCheckHook = D20CheckHooks<Skill>;
pub type SavingThrowHook = D20CheckHooks<Ability>;
