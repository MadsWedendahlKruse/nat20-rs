// TODO: Is a spell slot a resource? What about actions, bonus actions, reactions? Movement?

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
};

use crate::{components::id::ResourceId, registry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RechargeRule {
    Turn,
    ShortRest,
    LongRest,
    Daily,
    Never,
}

impl RechargeRule {
    /// Checks if this recharge rule is recharged by another rule.
    /// For example, `OnShortRest` is also recharged by `OnLongRest`.
    pub fn is_recharged_by(&self, other: &RechargeRule) -> bool {
        *other >= *self
    }
}

impl Display for RechargeRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBudget {
    pub current_uses: u8,
    pub max_uses: u8,
}

impl ResourceBudget {
    pub fn new(current_uses: u8, max_uses: u8) -> Result<Self, ResourceError> {
        if max_uses == 0 {
            return Err(ResourceError::ZeroMaxUses);
        }
        if current_uses > max_uses {
            return Err(ResourceError::CurrentUsesAboveMax {
                current_uses,
                max_uses,
            });
        }
        Ok(Self {
            current_uses,
            max_uses,
        })
    }

    pub fn with_max_uses(max_uses: u8) -> Result<Self, ResourceError> {
        Self::new(max_uses, max_uses)
    }

    pub fn can_afford(&self, amount: u8) -> bool {
        self.current_uses >= amount
    }

    pub fn spend(&mut self, amount: u8) -> Result<(), ResourceError> {
        if self.current_uses < amount {
            return Err(ResourceError::InsufficientResources {
                needed: amount,
                available: self.current_uses,
            });
        }
        self.current_uses -= amount;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.current_uses == 0
    }

    pub fn add_uses(&mut self, amount: u8) -> Result<(), ResourceError> {
        self.max_uses += amount;
        self.current_uses += amount;
        Ok(())
    }

    pub fn remove_uses(&mut self, amount: u8) -> Result<(), ResourceError> {
        if amount > self.max_uses {
            return Err(ResourceError::NegativeMaxUses {
                reduction: amount,
                max_uses: self.max_uses,
            });
        }
        self.max_uses -= amount;
        if self.current_uses > self.max_uses {
            self.current_uses = self.max_uses;
        }
        Ok(())
    }

    pub fn set_current_uses(&mut self, current_uses: u8) -> Result<(), ResourceError> {
        if current_uses > self.max_uses {
            return Err(ResourceError::CurrentUsesAboveMax {
                current_uses,
                max_uses: self.max_uses,
            });
        }
        self.current_uses = current_uses;
        Ok(())
    }

    pub fn set_max_uses(&mut self, max_uses: u8) -> Result<(), ResourceError> {
        if max_uses == 0 {
            return Err(ResourceError::ZeroMaxUses);
        }
        if max_uses < self.current_uses {
            self.current_uses = max_uses;
        }
        self.max_uses = max_uses;
        Ok(())
    }

    pub fn recharge_full(&mut self) {
        self.current_uses = self.max_uses;
    }

    // TODO: return type is just for the macro impl_resource_amount_router
    pub fn restore(&mut self, amount: u8) -> Result<(), ResourceError> {
        self.current_uses = (self.current_uses + amount).min(self.max_uses);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Flat(ResourceBudget),
    // Key is tier level
    Tiered(BTreeMap<u8, ResourceBudget>),
}

macro_rules! impl_resource_amount_router {
    ($( $fn_name:ident => $inner:ident ),+ $(,)?) => {
        $(
            pub fn $fn_name(&mut self, __arg: &ResourceAmount) -> Result<(), ResourceError> {
                match (&mut *self, __arg) {
                    (ResourceKind::Flat(budget), ResourceAmount::Flat(amt)) => {
                        budget.$inner(*amt)
                    }
                    (ResourceKind::Tiered(budgets), ResourceAmount::Tiered { tier, amount }) => {
                        if let Some(budget) = budgets.get_mut(tier) {
                            budget.$inner(*amount)
                        } else {
                            Err(ResourceError::InvalidTier {
                                tier: *tier,
                            })
                        }
                    }
                    _ => Err(ResourceError::MistmatchCostAndKind {
                        cost: __arg.clone(),
                        kind: self.clone(),
                    }),
                }
            }
        )+
    };
}

impl ResourceKind {
    pub fn recharge_full(&mut self) {
        match self {
            ResourceKind::Flat(budget) => budget.recharge_full(),
            ResourceKind::Tiered(budgets) => {
                for budget in budgets.values_mut() {
                    budget.recharge_full();
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ResourceKind::Flat(budget) => budget.is_empty(),
            ResourceKind::Tiered(budgets) => budgets.values().all(|b| b.is_empty()),
        }
    }

    pub fn can_afford(&self, cost: &ResourceAmount) -> bool {
        match (self, cost) {
            (ResourceKind::Flat(budget), ResourceAmount::Flat(amt)) => budget.can_afford(*amt),
            (ResourceKind::Tiered(budgets), ResourceAmount::Tiered { tier, amount }) => {
                if let Some(budget) = budgets.get(tier) {
                    budget.can_afford(*amount)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    impl_resource_amount_router! {
        spend => spend,
        restore => restore,
        add_uses => add_uses,
        remove_uses => remove_uses,
        set_current_uses => set_current_uses,
        set_max_uses => set_max_uses,
    }

    pub fn max_uses(&self) -> Vec<ResourceAmount> {
        match self {
            ResourceKind::Flat(budget) => vec![ResourceAmount::Flat(budget.max_uses)],
            ResourceKind::Tiered(budgets) => budgets
                .iter()
                .map(|(tier, budget)| ResourceAmount::Tiered {
                    tier: *tier,
                    amount: budget.max_uses,
                })
                .collect(),
        }
    }

    pub fn current_uses(&self) -> Vec<ResourceAmount> {
        match self {
            ResourceKind::Flat(budget) => vec![ResourceAmount::Flat(budget.current_uses)],
            ResourceKind::Tiered(budgets) => budgets
                .iter()
                .map(|(tier, budget)| ResourceAmount::Tiered {
                    tier: *tier,
                    amount: budget.current_uses,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceAmount {
    Flat(u8),
    Tiered { tier: u8, amount: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    id: ResourceId,
    kind: ResourceKind,
    recharge: RechargeRule,
}

impl Resource {
    pub fn new(id: &ResourceId, kind: ResourceKind, recharge: RechargeRule) -> Self {
        Self {
            id: id.clone(),
            kind,
            recharge,
        }
    }

    pub fn spend(&mut self, cost: &ResourceAmount) -> Result<(), ResourceError> {
        self.kind.spend(cost)
    }

    pub fn recharge_full(&mut self, rest_type: &RechargeRule) {
        // If the rest type is higher or equal to the resource's recharge rule,
        // recharge the resource.
        if self.recharge.is_recharged_by(rest_type) {
            self.kind.recharge_full();
        }
    }

    pub fn restore(&mut self, amount: &ResourceAmount) -> Result<(), ResourceError> {
        self.kind.restore(amount)
    }

    pub fn add_uses(&mut self, amount: &ResourceAmount) {
        self.kind.add_uses(amount).unwrap();
    }

    pub fn remove_uses(&mut self, amount: &ResourceAmount) -> Result<(), ResourceError> {
        self.kind.remove_uses(amount)
    }

    pub fn set_max_uses(&mut self, max_uses: &ResourceAmount) -> Result<(), ResourceError> {
        self.kind.set_max_uses(max_uses)
    }

    pub fn set_current_uses(&mut self, current_uses: &ResourceAmount) -> Result<(), ResourceError> {
        self.kind.set_current_uses(current_uses)
    }

    pub fn max_uses(&self) -> Vec<ResourceAmount> {
        self.kind.max_uses()
    }

    pub fn current_uses(&self) -> Vec<ResourceAmount> {
        self.kind.current_uses()
    }

    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    pub fn recharge_rule(&self) -> RechargeRule {
        self.recharge
    }

    pub fn can_afford(&self, cost: &ResourceAmount) -> bool {
        self.kind.can_afford(cost)
    }

    pub fn kind(&self) -> &ResourceKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    InsufficientResources {
        needed: u8,
        available: u8,
    },
    InvalidTier {
        tier: u8,
    },
    MistmatchCostAndKind {
        cost: ResourceAmount,
        kind: ResourceKind,
    },
    InvalidResourceKind(String),
    ZeroMaxUses,
    NegativeMaxUses {
        reduction: u8,
        max_uses: u8,
    },
    CurrentUsesAboveMax {
        current_uses: u8,
        max_uses: u8,
    },
}

pub type ResourceAmountMap = HashMap<ResourceId, ResourceAmount>;

#[derive(Debug, Clone)]
pub struct ResourceMap {
    resources: HashMap<ResourceId, Resource>,
}

impl ResourceMap {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    // TODO: Can't this also be used to reduce max uses? Probbaly not intended
    pub fn add(&mut self, resource: Resource, set_current_uses: bool) {
        self.resources
            .entry(resource.id.clone())
            .and_modify(|existing| match (&mut existing.kind, resource.kind()) {
                (ResourceKind::Flat(existing_budget), ResourceKind::Flat(new_budget)) => {
                    existing_budget.set_max_uses(new_budget.max_uses).unwrap();
                    if set_current_uses {
                        existing_budget
                            .set_current_uses(new_budget.current_uses)
                            .unwrap();
                    }
                }

                (ResourceKind::Tiered(existing_budgets), ResourceKind::Tiered(new_budgets)) => {
                    for (tier, new_budget) in new_budgets {
                        existing_budgets
                            .entry(*tier)
                            .and_modify(|existing_budget| {
                                existing_budget.set_max_uses(new_budget.max_uses).unwrap();
                                if set_current_uses {
                                    existing_budget
                                        .set_current_uses(new_budget.current_uses)
                                        .unwrap();
                                }
                            })
                            .or_insert(new_budget.clone());
                    }
                }

                _ => {
                    panic!(
                        "Mismatched resource kinds for resource id {}. Existing: {:?}, new: {:?}",
                        resource.id,
                        existing.kind(),
                        resource.kind()
                    );
                }
            })
            .or_insert(resource);
    }

    pub fn get(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources.get(id)
    }

    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut Resource> {
        self.resources.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceId, &Resource)> {
        self.resources.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ResourceId, &mut Resource)> {
        self.resources.iter_mut()
    }

    pub fn can_afford(&self, id: &ResourceId, cost: &ResourceAmount) -> bool {
        if let Some(resource) = self.resources.get(id) {
            resource.can_afford(cost)
        } else {
            false
        }
    }

    pub fn can_afford_all(&self, cost: &ResourceAmountMap) -> bool {
        for (res_id, res_cost) in cost {
            if !self.can_afford(res_id, res_cost) {
                return false;
            }
        }

        true
    }

    pub fn spend(&mut self, id: &ResourceId, cost: &ResourceAmount) -> Result<(), ResourceError> {
        if let Some(resource) = self.resources.get_mut(id) {
            resource.spend(cost)
        } else {
            Err(ResourceError::InvalidResourceKind(format!(
                "Resource with id {} not found",
                id
            )))
        }
    }

    pub fn spend_all(&mut self, cost: &ResourceAmountMap) -> Result<(), ResourceError> {
        if !self.can_afford_all(cost) {
            return Err(ResourceError::InsufficientResources {
                needed: 0,
                available: 0,
            });
        }

        for (id, res_cost) in cost {
            self.spend(id, res_cost)?;
        }

        Ok(())
    }

    pub fn restore(
        &mut self,
        id: &ResourceId,
        restoration: &ResourceAmount,
    ) -> Result<(), ResourceError> {
        if let Some(resource) = self.resources.get_mut(id) {
            resource.restore(restoration)
        } else {
            Err(ResourceError::InvalidResourceKind(format!(
                "Resource with id {} not found",
                id
            )))
        }
    }

    pub fn restore_all(&mut self, restoration: &ResourceAmountMap) -> Result<(), ResourceError> {
        for (id, res_restoration) in restoration {
            self.restore(id, res_restoration)?;
        }

        Ok(())
    }
}

impl Default for ResourceMap {
    fn default() -> Self {
        let mut map = ResourceMap::new();
        map.resources = HashMap::from([
            (
                registry::resources::ACTION_ID.clone(),
                registry::resources::ACTION.build_resource(1),
            ),
            (
                registry::resources::BONUS_ACTION_ID.clone(),
                registry::resources::BONUS_ACTION.build_resource(1),
            ),
            (
                registry::resources::REACTION_ID.clone(),
                registry::resources::REACTION.build_resource(1),
            ),
        ]);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_resource(id: &str, current: u8, max: u8, recharge: RechargeRule) -> Resource {
        Resource::new(
            &ResourceId::from_str(id),
            ResourceKind::Flat(ResourceBudget::new(current, max).unwrap()),
            recharge,
        )
    }

    fn tiered_resource(id: &str, tiers: &[(u8, u8, u8)], recharge: RechargeRule) -> Resource {
        let mut map = BTreeMap::new();
        for (tier, current, max) in tiers {
            map.insert(*tier, ResourceBudget::new(*current, *max).unwrap());
        }
        Resource::new(
            &ResourceId::from_str(id),
            ResourceKind::Tiered(map),
            recharge,
        )
    }

    #[test]
    fn test_flat_spend_success() {
        let mut res = flat_resource("Ki Point", 3, 3, RechargeRule::ShortRest);
        assert!(res.spend(&ResourceAmount::Flat(2)).is_ok());
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(1));
    }

    #[test]
    fn test_flat_spend_insufficient() {
        let mut res = flat_resource("Rage", 1, 2, RechargeRule::LongRest);
        let err = res.spend(&ResourceAmount::Flat(2)).unwrap_err();
        match err {
            ResourceError::InsufficientResources { needed, available } => {
                assert_eq!(needed, 2);
                assert_eq!(available, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_flat_recharge() {
        let mut res = flat_resource("Bardic Inspiration", 2, 5, RechargeRule::LongRest);
        res.kind.recharge_full();
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(5));
    }

    #[test]
    fn test_flat_is_empty() {
        let res = flat_resource("Channel Divinity", 0, 1, RechargeRule::ShortRest);
        assert!(res.kind.is_empty());
    }

    #[test]
    fn test_flat_add_uses() {
        let mut res = flat_resource("Wild Shape", 2, 2, RechargeRule::ShortRest);
        res.add_uses(&ResourceAmount::Flat(3));
        assert_eq!(res.max_uses()[0], ResourceAmount::Flat(5));
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(5));
    }

    #[test]
    fn test_flat_remove_uses_success() {
        let mut res = flat_resource("Sorcery Point", 4, 4, RechargeRule::LongRest);
        assert!(res.remove_uses(&ResourceAmount::Flat(2)).is_ok());
        assert_eq!(res.max_uses()[0], ResourceAmount::Flat(2));
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(2));
    }

    #[test]
    fn test_flat_remove_uses_too_many() {
        let mut res = flat_resource("Lay On Hands", 1, 1, RechargeRule::LongRest);
        let err = res.remove_uses(&ResourceAmount::Flat(2)).unwrap_err();
        match err {
            ResourceError::NegativeMaxUses {
                reduction,
                max_uses,
            } => {
                assert_eq!(reduction, 2);
                assert_eq!(max_uses, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_flat_set_current_uses() {
        let mut res = flat_resource("Superiority Die", 3, 3, RechargeRule::ShortRest);
        assert!(res.set_current_uses(&ResourceAmount::Flat(2)).is_ok());
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(2));
    }

    #[test]
    fn test_flat_set_max_uses() {
        let mut res = flat_resource("Superiority Die", 3, 3, RechargeRule::ShortRest);
        assert!(res.set_max_uses(&ResourceAmount::Flat(4)).is_ok());
        assert_eq!(res.max_uses()[0], ResourceAmount::Flat(4));
    }

    #[test]
    fn test_flat_recharge_rule_order() {
        assert!(RechargeRule::ShortRest > RechargeRule::Turn);
        assert!(RechargeRule::LongRest > RechargeRule::ShortRest);
        assert!(RechargeRule::Daily > RechargeRule::LongRest);
        assert!(RechargeRule::Never > RechargeRule::Daily);
    }

    #[test]
    fn test_flat_resource_recharge_respects_order() {
        let mut res = flat_resource("Test Resource", 0, 2, RechargeRule::ShortRest);
        res.recharge_full(&RechargeRule::ShortRest);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(2));
        res.spend(&ResourceAmount::Flat(2)).unwrap();
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
        res.recharge_full(&RechargeRule::LongRest);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(2));
        res.spend(&ResourceAmount::Flat(2)).unwrap();
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
        res.recharge_full(&RechargeRule::Daily);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(2));
        res.spend(&ResourceAmount::Flat(2)).unwrap();
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
        res.recharge_full(&RechargeRule::Turn);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
    }

    #[test]
    fn test_flat_resource_recharge_rule_never() {
        let mut res = flat_resource("No Recharge", 0, 1, RechargeRule::Never);
        res.recharge_full(&RechargeRule::ShortRest);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
        res.recharge_full(&RechargeRule::LongRest);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
        res.recharge_full(&RechargeRule::Daily);
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(0));
    }

    #[test]
    fn test_tiered_spend_success() {
        let mut res = tiered_resource(
            "Spell Slot",
            &[(1, 2, 2), (2, 1, 1)],
            RechargeRule::LongRest,
        );
        assert!(
            res.spend(&ResourceAmount::Tiered { tier: 1, amount: 1 })
                .is_ok()
        );
        let uses = res.current_uses();
        assert_eq!(
            uses,
            vec![
                ResourceAmount::Tiered { tier: 1, amount: 1 },
                ResourceAmount::Tiered { tier: 2, amount: 1 }
            ]
        );
    }

    #[test]
    fn test_tiered_spend_invalid_tier() {
        let mut res = tiered_resource("Spell Slot", &[(1, 2, 2)], RechargeRule::LongRest);
        let err = res
            .spend(&ResourceAmount::Tiered { tier: 2, amount: 1 })
            .unwrap_err();
        match err {
            ResourceError::InvalidTier { tier } => assert_eq!(tier, 2),
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_tiered_add_uses() {
        let mut res = tiered_resource(
            "Spell Slot",
            &[(1, 2, 2), (2, 1, 1)],
            RechargeRule::LongRest,
        );
        res.add_uses(&ResourceAmount::Tiered { tier: 1, amount: 1 });
        let uses = res.max_uses();
        assert_eq!(
            uses,
            vec![
                ResourceAmount::Tiered { tier: 1, amount: 3 },
                ResourceAmount::Tiered { tier: 2, amount: 1 }
            ]
        );
    }

    #[test]
    fn test_tiered_remove_uses_success() {
        let mut res = tiered_resource(
            "Spell Slot",
            &[(1, 2, 2), (2, 1, 1)],
            RechargeRule::LongRest,
        );
        assert!(
            res.remove_uses(&ResourceAmount::Tiered { tier: 1, amount: 1 })
                .is_ok()
        );
        let uses = res.max_uses();
        assert_eq!(
            uses,
            vec![
                ResourceAmount::Tiered { tier: 1, amount: 1 },
                ResourceAmount::Tiered { tier: 2, amount: 1 }
            ]
        );
    }

    #[test]
    fn test_tiered_remove_uses_too_many() {
        let mut res = tiered_resource("Spell Slot", &[(1, 2, 2)], RechargeRule::LongRest);
        let err = res
            .remove_uses(&ResourceAmount::Tiered { tier: 1, amount: 3 })
            .unwrap_err();
        match err {
            ResourceError::NegativeMaxUses {
                reduction,
                max_uses,
            } => {
                assert_eq!(reduction, 3);
                assert_eq!(max_uses, 2);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_tiered_recharge() {
        let mut res = tiered_resource(
            "Spell Slot",
            &[(1, 0, 2), (2, 0, 1)],
            RechargeRule::LongRest,
        );
        res.kind.recharge_full();
        let uses = res.current_uses();
        assert_eq!(
            uses,
            vec![
                ResourceAmount::Tiered { tier: 1, amount: 2 },
                ResourceAmount::Tiered { tier: 2, amount: 1 }
            ]
        );
    }

    #[test]
    fn test_tiered_is_empty() {
        let res = tiered_resource(
            "Spell Slot",
            &[(1, 0, 2), (2, 0, 1)],
            RechargeRule::LongRest,
        );
        assert!(res.kind.is_empty());
    }

    #[test]
    fn test_resource_map_add_and_get() {
        let mut map = ResourceMap::new();
        let res = flat_resource("Action", 1, 1, RechargeRule::Turn);
        map.add(res.clone(), false);
        let got = map.get(&ResourceId::from_str("Action")).unwrap();
        assert_eq!(got.max_uses(), vec![ResourceAmount::Flat(1)]);
    }

    #[test]
    fn test_resource_map_iter() {
        let mut map = ResourceMap::new();
        map.add(flat_resource("Action", 1, 1, RechargeRule::Turn), false);
        map.add(
            flat_resource("Bonus Action", 1, 1, RechargeRule::Turn),
            false,
        );
        let ids: Vec<_> = map.iter().map(|(id, _)| id.to_string()).collect();
        assert!(ids.contains(&"Action".to_string()));
        assert!(ids.contains(&"Bonus Action".to_string()));
    }

    #[test]
    fn test_resource_map_can_afford_flat() {
        let mut map = ResourceMap::new();
        map.add(
            flat_resource("Ki Point", 3, 3, RechargeRule::ShortRest),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(ResourceId::from_str("Ki Point"), ResourceAmount::Flat(2));

        assert!(map.can_afford_all(&cost));
    }

    #[test]
    fn test_resource_map_can_afford_flat_insufficient() {
        let mut map = ResourceMap::new();
        map.add(
            flat_resource("Ki Point", 1, 3, RechargeRule::ShortRest),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(ResourceId::from_str("Ki Point"), ResourceAmount::Flat(2));

        assert!(!map.can_afford_all(&cost));
    }

    #[test]
    fn test_resource_map_can_afford_tier() {
        let mut map = ResourceMap::new();
        map.add(
            tiered_resource(
                "Spell Slot",
                &[(1, 2, 2), (2, 1, 1)],
                RechargeRule::LongRest,
            ),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(
            ResourceId::from_str("Spell Slot"),
            ResourceAmount::Tiered { tier: 1, amount: 2 },
        );

        assert!(map.can_afford_all(&cost));
    }

    #[test]
    fn test_resource_map_can_afford_tier_insufficient() {
        let mut map = ResourceMap::new();
        map.add(
            tiered_resource(
                "Spell Slot",
                &[(1, 1, 2), (2, 1, 1)],
                RechargeRule::LongRest,
            ),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(
            ResourceId::from_str("Spell Slot"),
            ResourceAmount::Tiered { tier: 1, amount: 2 },
        );

        assert!(!map.can_afford_all(&cost));
    }

    #[test]
    fn test_resource_map_spend_flat_success() {
        let mut map = ResourceMap::new();
        map.add(
            flat_resource("Ki Point", 3, 3, RechargeRule::ShortRest),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(ResourceId::from_str("Ki Point"), ResourceAmount::Flat(2));

        assert!(map.spend_all(&cost).is_ok());
        let res = map.get(&ResourceId::from_str("Ki Point")).unwrap();
        assert_eq!(res.current_uses()[0], ResourceAmount::Flat(1));
    }

    #[test]
    fn test_resource_map_spend_flat_insufficient() {
        let mut map = ResourceMap::new();
        map.add(
            flat_resource("Ki Point", 1, 3, RechargeRule::ShortRest),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(ResourceId::from_str("Ki Point"), ResourceAmount::Flat(2));

        let err = map.spend_all(&cost).unwrap_err();
        match err {
            ResourceError::InsufficientResources { .. } => {}
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_resource_map_spend_tier_success() {
        let mut map = ResourceMap::new();
        map.add(
            tiered_resource(
                "Spell Slot",
                &[(1, 2, 2), (2, 1, 1)],
                RechargeRule::LongRest,
            ),
            false,
        );

        let mut cost = ResourceAmountMap::new();
        cost.insert(
            ResourceId::from_str("Spell Slot"),
            ResourceAmount::Tiered { tier: 1, amount: 2 },
        );

        assert!(map.spend_all(&cost).is_ok());
        let res = map.get(&ResourceId::from_str("Spell Slot")).unwrap();
        let uses = res.current_uses();
        assert_eq!(
            uses,
            vec![
                ResourceAmount::Tiered { tier: 1, amount: 0 },
                ResourceAmount::Tiered { tier: 2, amount: 1 }
            ]
        );
    }

    #[test]
    fn test_resource_map_spend_multiple_resources() {
        let mut map = ResourceMap::new();
        map.add(
            flat_resource("Ki Point", 3, 3, RechargeRule::ShortRest),
            false,
        );
        map.add(flat_resource("Rage", 2, 2, RechargeRule::LongRest), false);

        let mut cost = ResourceAmountMap::new();
        cost.insert(ResourceId::from_str("Ki Point"), ResourceAmount::Flat(2));
        cost.insert(ResourceId::from_str("Rage"), ResourceAmount::Flat(1));

        assert!(map.spend_all(&cost).is_ok());
        let ki = map.get(&ResourceId::from_str("Ki Point")).unwrap();
        let rage = map.get(&ResourceId::from_str("Rage")).unwrap();
        assert_eq!(ki.current_uses()[0], ResourceAmount::Flat(1));
        assert_eq!(rage.current_uses()[0], ResourceAmount::Flat(1));
    }
}
