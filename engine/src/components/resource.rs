// TODO: Is a spell slot a resource? What about actions, bonus actions, reactions? Movement?

// TODO: Not sure if an enum is the best way to represent these resources, could also just use a string
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum ResourceKind {
//     ArcaneRecovery,
//     BardicInspiration,
//     ChannelDivinity,
//     ChannelOath,
//     KiPoint,
//     LayOnHandsCharge,
//     RageCharge,
//     SorceryPoint,
//     SuperiorityDie,
//     WildShapeCharge,
//     // add more as needed
// }

use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::components::id::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RechargeRule {
    OnTurn,
    OnShortRest,
    OnLongRest,
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

#[derive(Debug, Clone)]
pub struct Resource {
    kind: ResourceId,
    max_uses: u8,
    current_uses: u8,
    recharge: RechargeRule,
}

impl Resource {
    pub fn new(
        kind: ResourceId,
        max_uses: u8,
        recharge: RechargeRule,
    ) -> Result<Self, ResourceError> {
        if max_uses == 0 {
            return Err(ResourceError::ZeroMaxUses);
        }
        Ok(Self {
            kind,
            max_uses,
            current_uses: max_uses,
            recharge,
        })
    }

    /// Attempt to use `amount` points from this resource. Returns Err if not enough left.
    pub fn spend(&mut self, amount: u8) -> Result<(), ResourceError> {
        if self.current_uses < amount {
            Err(ResourceError::InsufficientResources {
                kind: self.kind.clone(),
                needed: amount,
                available: self.current_uses,
            })
        } else {
            self.current_uses -= amount;
            Ok(())
        }
    }

    /// Refill back to max.
    fn recharge_internal(&mut self) {
        self.current_uses = self.max_uses;
    }

    pub fn recharge(&mut self, rest_type: &RechargeRule) {
        // If the rest type is higher or equal to the resource's recharge rule,
        // recharge the resource.
        if self.recharge.is_recharged_by(rest_type) {
            self.recharge_internal();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current_uses == 0
    }

    pub fn add_uses(&mut self, amount: u8) -> Result<(), ResourceError> {
        self.max_uses += amount;
        self.current_uses += amount;
        Ok(())
    }

    pub fn add_use(&mut self) -> Result<(), ResourceError> {
        self.add_uses(1)
    }

    pub fn remove_uses(&mut self, amount: u8) -> Result<(), ResourceError> {
        if self.max_uses < amount {
            Err(ResourceError::NegativeMaxUses {
                kind: self.kind.clone(),
                reduction: amount,
                max_uses: self.max_uses,
            })
        } else {
            self.max_uses -= amount;
            if self.current_uses > self.max_uses {
                self.current_uses = self.max_uses;
            }
            Ok(())
        }
    }

    pub fn remove_use(&mut self) -> Result<(), ResourceError> {
        self.remove_uses(1)
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

    pub fn set_current_uses(&mut self, current_uses: u8) -> Result<(), ResourceError> {
        if current_uses > self.max_uses {
            return Err(ResourceError::TooManyCurrentUses {
                kind: self.kind.clone(),
                current_uses,
                max_uses: self.max_uses,
            });
        }
        self.current_uses = current_uses;
        Ok(())
    }

    pub fn max_uses(&self) -> u8 {
        self.max_uses
    }

    pub fn current_uses(&self) -> u8 {
        self.current_uses
    }

    pub fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn recharge_rule(&self) -> RechargeRule {
        self.recharge
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    InsufficientResources {
        kind: ResourceId,
        needed: u8,
        available: u8,
    },
    InvalidResourceKind(String),
    ZeroMaxUses,
    NegativeMaxUses {
        kind: ResourceId,
        reduction: u8,
        max_uses: u8,
    },
    TooManyCurrentUses {
        kind: ResourceId,
        current_uses: u8,
        max_uses: u8,
    },
}

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

    pub fn add(&mut self, resource: Resource, set_current_uses: bool) {
        self.resources
            .entry(resource.kind().clone())
            .and_modify(|r| {
                r.set_max_uses(resource.max_uses()).unwrap();
                if set_current_uses {
                    r.set_current_uses(resource.current_uses()).unwrap();
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
}

pub type ResourceCostMap = HashMap<ResourceId, u8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spend_success() {
        let mut res = Resource {
            kind: ResourceId::from_str("Ki Point"),
            max_uses: 3,
            current_uses: 3,
            recharge: RechargeRule::OnShortRest,
        };
        assert!(res.spend(2).is_ok());
        assert_eq!(res.current_uses, 1);
    }

    #[test]
    fn test_spend_insufficient() {
        let mut res = Resource {
            kind: ResourceId::from_str("Rage"),
            max_uses: 2,
            current_uses: 1,
            recharge: RechargeRule::OnLongRest,
        };
        let err = res.spend(2).unwrap_err();
        match err {
            ResourceError::InsufficientResources {
                kind,
                needed,
                available,
            } => {
                assert_eq!(kind, ResourceId::from_str("Rage"));
                assert_eq!(needed, 2);
                assert_eq!(available, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_recharge() {
        let mut res = Resource {
            kind: ResourceId::from_str("Bardic Inspiration"),
            max_uses: 5,
            current_uses: 2,
            recharge: RechargeRule::OnLongRest,
        };
        res.recharge_internal();
        assert_eq!(res.current_uses, 5);
    }

    #[test]
    fn test_is_empty() {
        let res = Resource {
            kind: ResourceId::from_str("Channel Divinity"),
            max_uses: 1,
            current_uses: 0,
            recharge: RechargeRule::OnShortRest,
        };
        assert!(res.is_empty());
    }

    #[test]
    fn test_add_uses() {
        let mut res = Resource {
            kind: ResourceId::from_str("Wild Shape"),
            max_uses: 2,
            current_uses: 2,
            recharge: RechargeRule::OnShortRest,
        };
        assert!(res.add_uses(3).is_ok());
        assert_eq!(res.max_uses, 5);
        assert_eq!(res.current_uses, 5);
    }

    #[test]
    fn test_remove_uses_success() {
        let mut res = Resource {
            kind: ResourceId::from_str("Sorcery Point"),
            max_uses: 4,
            current_uses: 4,
            recharge: RechargeRule::OnLongRest,
        };
        assert!(res.remove_uses(2).is_ok());
        assert_eq!(res.max_uses, 2);
        assert_eq!(res.current_uses, 2);
    }

    #[test]
    fn test_remove_uses_too_many() {
        let mut res = Resource {
            kind: ResourceId::from_str("Lay On Hands"),
            max_uses: 1,
            current_uses: 1,
            recharge: RechargeRule::OnLongRest,
        };
        let err = res.remove_uses(2).unwrap_err();
        match err {
            ResourceError::NegativeMaxUses {
                kind,
                reduction,
                max_uses,
            } => {
                assert_eq!(kind, ResourceId::from_str("Lay On Hands"));
                assert_eq!(reduction, 2);
                assert_eq!(max_uses, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_add_and_remove_single_use() {
        let mut res = Resource {
            kind: ResourceId::from_str("Superiority Die"),
            max_uses: 3,
            current_uses: 3,
            recharge: RechargeRule::OnShortRest,
        };
        assert!(res.add_use().is_ok());
        assert_eq!(res.max_uses, 4);
        assert_eq!(res.current_uses, 4);

        assert!(res.remove_use().is_ok());
        assert_eq!(res.max_uses, 3);
        assert_eq!(res.current_uses, 3);
    }

    #[test]
    fn test_recharge_on_short_rest() {
        let mut res = Resource {
            kind: ResourceId::from_str("Ki Point"),
            max_uses: 3,
            current_uses: 1,
            recharge: RechargeRule::OnShortRest,
        };
        res.recharge(&RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 3);
    }

    #[test]
    fn test_recharge_on_long_rest() {
        let mut res = Resource {
            kind: ResourceId::from_str("Arcane Recovery"),
            max_uses: 2,
            current_uses: 0,
            recharge: RechargeRule::OnLongRest,
        };
        res.recharge(&RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 2);
    }

    #[test]
    fn test_recharge_on_any_rest() {
        let mut res = Resource {
            kind: ResourceId::from_str("Channel Oath"),
            max_uses: 1,
            current_uses: 0,
            recharge: RechargeRule::OnShortRest,
        };
        res.recharge(&RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 1);

        res.spend(1).unwrap(); // Use it up
        assert!(res.is_empty());

        res.recharge(&RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 1);
    }

    #[test]
    fn test_recharge_rule_order() {
        assert!(RechargeRule::OnShortRest > RechargeRule::OnTurn);
        assert!(RechargeRule::OnLongRest > RechargeRule::OnShortRest);
        assert!(RechargeRule::Daily > RechargeRule::OnLongRest);
        assert!(RechargeRule::Never > RechargeRule::Daily);
    }

    #[test]
    fn test_resource_recharge_respects_order() {
        let mut res = Resource {
            kind: ResourceId::from_str("Test Resource"),
            max_uses: 2,
            current_uses: 0,
            recharge: RechargeRule::OnShortRest,
        };
        // Should recharge on short rest
        res.recharge(&RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 2);

        // Use up resource
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should recharge on long rest (higher in order)
        res.recharge(&RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 2);

        // Use up resource again
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should recharge on daily (highest in order)
        res.recharge(&RechargeRule::Daily);
        assert_eq!(res.current_uses, 2);

        // Use up resource again
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should NOT recharge on turn (lower in order)
        res.recharge(&RechargeRule::OnTurn);
        assert_eq!(res.current_uses, 0);
    }

    #[test]
    fn test_resource_recharge_rule_never() {
        let mut res = Resource {
            kind: ResourceId::from_str("No Recharge"),
            max_uses: 1,
            current_uses: 0,
            recharge: RechargeRule::Never,
        };
        res.recharge(&RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 0);
        res.recharge(&RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 0);
        res.recharge(&RechargeRule::Daily);
        assert_eq!(res.current_uses, 0);
    }
}
