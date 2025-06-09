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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RechargeRule {
    OnTurn,
    OnShortRest,
    OnLongRest,
    OnAnyRest,
    // TODO: Daily is the same as long rest?
    Daily,
    None,
}

#[derive(Debug, Clone)]
pub struct Resource {
    kind: String,
    max_uses: u8,
    current_uses: u8,
    recharge: RechargeRule,
}

impl Resource {
    pub fn new(kind: &str, max_uses: u8, recharge: RechargeRule) -> Result<Self, ResourceError> {
        if max_uses == 0 {
            return Err(ResourceError::ZeroMaxUses);
        }
        Ok(Self {
            kind: kind.to_string(),
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
        if self.recharge == rest_type
            || (self.recharge == RechargeRule::OnAnyRest
                && (rest_type == RechargeRule::OnShortRest
                    || rest_type == RechargeRule::OnLongRest))
        {
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

    pub fn max_uses(&self) -> u8 {
        self.max_uses
    }

    pub fn current_uses(&self) -> u8 {
        self.current_uses
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn recharge_rule(&self) -> RechargeRule {
        self.recharge
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    InsufficientResources {
        kind: String,
        needed: u8,
        available: u8,
    },
    InvalidResourceKind(String),
    ZeroMaxUses,
    NegativeMaxUses {
        kind: String,
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
            kind: "Ki Point".to_string(),
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
            kind: "Rage".to_string(),
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
                assert_eq!(kind, "Rage");
                assert_eq!(needed, 2);
                assert_eq!(available, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_recharge() {
        let mut res = Resource {
            kind: "Bardic Inspiration".to_string(),
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
            kind: "Channel Divinity".to_string(),
            max_uses: 1,
            current_uses: 0,
            recharge: RechargeRule::OnShortRest,
        };
        assert!(res.is_empty());
    }

    #[test]
    fn test_add_uses() {
        let mut res = Resource {
            kind: "Wild Shape".to_string(),
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
            kind: "Sorcery Point".to_string(),
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
            kind: "Lay On Hands".to_string(),
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
                assert_eq!(kind, "Lay On Hands");
                assert_eq!(reduction, 2);
                assert_eq!(max_uses, 1);
            }
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_add_and_remove_single_use() {
        let mut res = Resource {
            kind: "Superiority Die".to_string(),
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
            kind: "Ki Point".to_string(),
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
            kind: "Arcane Recovery".to_string(),
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
            kind: "Channel Oath".to_string(),
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
}
