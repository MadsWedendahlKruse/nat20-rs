use std::{collections::HashMap, fmt, sync::Arc};

use hecs::{Entity, World};

use crate::components::{
    ability::Ability,
    actions::action::{Action, ActionContext},
    d20_check::{D20Check, D20CheckResult},
    damage::{AttackRoll, AttackRollResult, DamageRoll, DamageRollResult},
    id::ResourceId,
    modifier::ModifierSet,
    skill::Skill,
};

pub type EffectHook = Arc<dyn Fn(&mut World, Entity) + Send + Sync>;
pub type AttackRollHook = Arc<dyn Fn(&World, Entity, &mut AttackRoll) + Send + Sync>;
pub type AttackRollResultHook = Arc<dyn Fn(&World, Entity, &mut AttackRollResult) + Send + Sync>;
pub type ArmorClassHook = Arc<dyn Fn(&World, Entity, &mut ModifierSet) + Send + Sync>;
pub type D20CheckHook = Arc<dyn Fn(&World, Entity, &mut D20Check) + Send + Sync>;
pub type D20CheckResultHook = Arc<dyn Fn(&World, Entity, &mut D20CheckResult) + Send + Sync>;
pub type DamageRollHook = Arc<dyn Fn(&World, Entity, &mut DamageRoll) + Send + Sync>;
pub type DamageRollResultHook = Arc<dyn Fn(&World, Entity, &mut DamageRollResult) + Send + Sync>;
pub type ActionHook = Arc<dyn Fn(&mut World, Entity, &Action, &ActionContext) + Send + Sync>;
// TODO: Struct or type alias for the resource map?
pub type ResourceCostHook =
    Arc<dyn Fn(&World, Entity, &ActionContext, &mut HashMap<ResourceId, u8>) + Send + Sync>;

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
        static NOOP_CHECK: LazyLock<D20CheckHook> = LazyLock::new(|| Arc::new(|_, _, _| {}));
        static NOOP_RESULT: LazyLock<D20CheckResultHook> = LazyLock::new(|| Arc::new(|_, _, _| {}));

        Self {
            key,
            check_hook: NOOP_CHECK.clone(),
            result_hook: NOOP_RESULT.clone(),
        }
    }
}

pub type SkillCheckHook = D20CheckHooks<Skill>;
pub type SavingThrowHook = D20CheckHooks<Ability>;
