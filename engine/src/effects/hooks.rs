use std::{collections::HashMap, fmt, sync::Arc};

use crate::{
    actions::action::{Action, ActionContext},
    combat::damage::{AttackRoll, AttackRollResult, DamageRoll, DamageRollResult},
    creature::character::Character,
    stats::{
        ability::Ability,
        d20_check::{D20Check, D20CheckResult},
        modifier::ModifierSet,
        skill::Skill,
    },
    utils::id::ResourceId,
};

pub type EffectHook = Arc<dyn Fn(&mut Character) + Send + Sync>;
pub type AttackRollHook = Arc<dyn Fn(&Character, &mut AttackRoll) + Send + Sync>;
pub type AttackRollResultHook = Arc<dyn Fn(&Character, &mut AttackRollResult) + Send + Sync>;
pub type ArmorClassHook = Arc<dyn Fn(&Character, &mut ModifierSet) + Send + Sync>;
pub type D20CheckHook = Arc<dyn Fn(&Character, &mut D20Check) + Send + Sync>;
pub type D20CheckResultHook = Arc<dyn Fn(&Character, &mut D20CheckResult) + Send + Sync>;
pub type DamageRollHook = Arc<dyn Fn(&Character, &mut DamageRoll) + Send + Sync>;
pub type DamageRollResultHook = Arc<dyn Fn(&Character, &mut DamageRollResult) + Send + Sync>;
pub type ActionHook = Arc<dyn Fn(&mut Character, &Action, &ActionContext) + Send + Sync>;
// TODO: Struct or type alias for the resource map?
pub type ResourceCostHook =
    Arc<dyn Fn(&Character, &ActionContext, &mut HashMap<ResourceId, u8>) + Send + Sync>;

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
