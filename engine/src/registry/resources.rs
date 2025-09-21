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

// --- DEFAULT RESOURCES ---

pub static ACTION_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.action"));

pub static ACTION: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| resource_builder_flat(&ACTION_ID, amount, RechargeRule::Turn),
        cost_builder: |amount| cost_builder_flat(amount),
    });

pub static BONUS_ACTION_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.bonus_action"));

pub static BONUS_ACTION: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| {
            resource_builder_flat(&BONUS_ACTION_ID, amount, RechargeRule::Turn)
        },
        cost_builder: |amount| cost_builder_flat(amount),
    });

pub static REACTION_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.reaction"));

pub static REACTION: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| resource_builder_flat(&REACTION_ID, amount, RechargeRule::Turn),
        cost_builder: |amount| cost_builder_flat(amount),
    });

// --- CLASS RESOURCES ---

pub static ACTION_SURGE_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.action_surge"));

pub static ACTION_SURGE: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| {
            resource_builder_flat(&ACTION_SURGE_ID, amount, RechargeRule::ShortRest)
        },
        cost_builder: |amount| cost_builder_flat(amount),
    });

pub static EXTRA_ATTACK_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.extra_attack"));

pub static EXTRA_ATTACK: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| {
            resource_builder_flat(&EXTRA_ATTACK_ID, amount, RechargeRule::Never)
        },
        cost_builder: |amount| cost_builder_flat(amount),
    });

pub static SECOND_WIND_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.second_wind"));

pub static SECOND_WIND: LazyLock<FlatResourceRegistryEntry> =
    LazyLock::new(|| FlatResourceRegistryEntry {
        resource_builer: |amount| {
            resource_builder_flat(&SECOND_WIND_ID, amount, RechargeRule::ShortRest)
        },
        cost_builder: |amount| cost_builder_flat(amount),
    });

pub static SPELL_SLOT_ID: LazyLock<ResourceId> =
    LazyLock::new(|| ResourceId::from_str("resource.spell_slot"));

pub static SPELL_SLOT: LazyLock<TieredResourceRegistryEntry> =
    LazyLock::new(|| TieredResourceRegistryEntry {
        resource_builer: |tier, amount| {
            resource_builder_tiered(&SPELL_SLOT_ID, tier, amount, RechargeRule::LongRest)
        },
        cost_builder: |tier, amount| cost_builder_tiered(tier, amount),
    });
