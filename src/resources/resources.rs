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

use crate::utils::id::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RechargeRule {
    OnTurn = 0,
    OnAnyRest = 1,
    OnShortRest = 2,
    OnLongRest = 3,
    Daily = 4,
    Never = 5,
}

impl RechargeRule {
    /// Returns the hierarchy level of the recharge rule.
    pub fn hierarchy(&self) -> u8 {
        *self as u8
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

    pub fn recharge(&mut self, rest_type: RechargeRule) {
        // If the rest type is higher or equal to the resource's recharge rule,
        // recharge the resource.
        if rest_type.hierarchy() >= self.recharge.hierarchy() {
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
}

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
        res.recharge(RechargeRule::OnShortRest);
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
        res.recharge(RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 2);
    }

    #[test]
    fn test_recharge_on_any_rest() {
        let mut res = Resource {
            kind: ResourceId::from_str("Channel Oath"),
            max_uses: 1,
            current_uses: 0,
            recharge: RechargeRule::OnAnyRest,
        };
        res.recharge(RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 1);

        res.spend(1).unwrap(); // Use it up
        assert!(res.is_empty());

        res.recharge(RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 1);
    }

    #[test]
    fn test_recharge_rule_hierarchy_order() {
        assert!(RechargeRule::OnAnyRest.hierarchy() > RechargeRule::OnTurn.hierarchy());
        assert!(RechargeRule::OnShortRest.hierarchy() > RechargeRule::OnAnyRest.hierarchy());
        assert!(RechargeRule::OnLongRest.hierarchy() > RechargeRule::OnShortRest.hierarchy());
        assert!(RechargeRule::Daily.hierarchy() > RechargeRule::OnLongRest.hierarchy());
        assert!(RechargeRule::Never.hierarchy() > RechargeRule::Daily.hierarchy());
    }

    #[test]
    fn test_recharge_rule_hierarchy_equality() {
        assert_eq!(RechargeRule::OnTurn.hierarchy(), 0);
        assert_eq!(RechargeRule::OnAnyRest.hierarchy(), 1);
        assert_eq!(RechargeRule::OnShortRest.hierarchy(), 2);
        assert_eq!(RechargeRule::OnLongRest.hierarchy(), 3);
        assert_eq!(RechargeRule::Daily.hierarchy(), 4);
        assert_eq!(RechargeRule::Never.hierarchy(), 5);
    }

    #[test]
    fn test_resource_recharge_respects_hierarchy() {
        let mut res = Resource {
            kind: ResourceId::from_str("Test Resource"),
            max_uses: 2,
            current_uses: 0,
            recharge: RechargeRule::OnShortRest,
        };
        // Should recharge on short rest
        res.recharge(RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 2);

        // Use up resource
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should recharge on long rest (higher in hierarchy)
        res.recharge(RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 2);

        // Use up resource again
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should recharge on daily (highest in hierarchy)
        res.recharge(RechargeRule::Daily);
        assert_eq!(res.current_uses, 2);

        // Use up resource again
        res.spend(2).unwrap();
        assert_eq!(res.current_uses, 0);

        // Should NOT recharge on turn (lower in hierarchy)
        res.recharge(RechargeRule::OnTurn);
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
        res.recharge(RechargeRule::OnShortRest);
        assert_eq!(res.current_uses, 0);
        res.recharge(RechargeRule::OnLongRest);
        assert_eq!(res.current_uses, 0);
        res.recharge(RechargeRule::Daily);
        assert_eq!(res.current_uses, 0);
    }
}
