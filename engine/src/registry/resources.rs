use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

use crate::components::{
    id::ResourceId,
    resource::{RechargeRule, Resource, ResourceAmount, ResourceBudget, ResourceKind},
};

pub struct FlatResourceRegistryEntry {
    resource_builer: fn(u8) -> Resource,
    cost_builder: fn(u8) -> ResourceAmount,
}

impl FlatResourceRegistryEntry {
    pub fn build_resource(&self, amount: u8) -> Resource {
        (self.resource_builer)(amount)
    }

    pub fn build_amount(&self, amount: u8) -> ResourceAmount {
        (self.cost_builder)(amount)
    }
}

pub struct TieredResourceRegistryEntry {
    resource_builer: fn(u8, u8) -> Resource,
    cost_builder: fn(u8, u8) -> ResourceAmount,
}

impl TieredResourceRegistryEntry {
    pub fn build_resource(&self, tier: u8, amount: u8) -> Resource {
        (self.resource_builer)(tier, amount)
    }

    pub fn build_cost(&self, tier: u8, amount: u8) -> ResourceAmount {
        (self.cost_builder)(tier, amount)
    }
}

fn resource_builder_flat(id: &ResourceId, amount: u8, recharge: RechargeRule) -> Resource {
    Resource::new(
        id,
        ResourceKind::Flat(ResourceBudget::with_max_uses(amount).unwrap()),
        recharge,
    )
}

fn resource_builder_tiered(
    id: &ResourceId,
    tier: u8,
    amount: u8,
    recharge: RechargeRule,
) -> Resource {
    Resource::new(
        id,
        ResourceKind::Tiered(BTreeMap::from([(
            tier,
            ResourceBudget::with_max_uses(amount).unwrap(),
        )])),
        recharge,
    )
}

fn cost_builder_flat(amount: u8) -> ResourceAmount {
    ResourceAmount::Flat(amount)
}

fn cost_builder_tiered(tier: u8, amount: u8) -> ResourceAmount {
    ResourceAmount::Tiered {
        tier: tier,
        amount: amount,
    }
}

macro_rules! flat_resource {
    ($name:ident, $id_name:ident, $id_str:expr, $recharge:expr) => {
        pub static $id_name: LazyLock<ResourceId> = LazyLock::new(|| ResourceId::from_str($id_str));

        pub static $name: LazyLock<FlatResourceRegistryEntry> =
            LazyLock::new(|| FlatResourceRegistryEntry {
                resource_builer: |amount| resource_builder_flat(&$id_name, amount, $recharge),
                cost_builder: |amount| cost_builder_flat(amount),
            });
    };
}

macro_rules! tiered_resource {
    ($name:ident, $id_name:ident, $id_str:expr, $recharge:expr) => {
        pub static $id_name: LazyLock<ResourceId> = LazyLock::new(|| ResourceId::from_str($id_str));

        pub static $name: LazyLock<TieredResourceRegistryEntry> =
            LazyLock::new(|| TieredResourceRegistryEntry {
                resource_builer: |tier, amount| {
                    resource_builder_tiered(&$id_name, tier, amount, $recharge)
                },
                cost_builder: |tier, amount| cost_builder_tiered(tier, amount),
            });
    };
}

// --- DEFAULT RESOURCES ---
flat_resource!(ACTION, ACTION_ID, "resource.action", RechargeRule::Turn);
flat_resource!(
    BONUS_ACTION,
    BONUS_ACTION_ID,
    "resource.bonus_action",
    RechargeRule::Turn
);
flat_resource!(
    REACTION,
    REACTION_ID,
    "resource.reaction",
    RechargeRule::Turn
);

// --- CLASS RESOURCES ---
flat_resource!(
    ACTION_SURGE,
    ACTION_SURGE_ID,
    "resource.action_surge",
    RechargeRule::ShortRest
);
flat_resource!(
    EXTRA_ATTACK,
    EXTRA_ATTACK_ID,
    "resource.extra_attack",
    RechargeRule::Never
);
flat_resource!(
    INDOMITABLE,
    INDOMITABLE_ID,
    "resource.indomitable",
    RechargeRule::LongRest
);
flat_resource!(
    SECOND_WIND,
    SECOND_WIND_ID,
    "resource.second_wind",
    RechargeRule::ShortRest
);

tiered_resource!(
    SPELL_SLOT,
    SPELL_SLOT_ID,
    "resource.spell_slot",
    RechargeRule::LongRest
);
