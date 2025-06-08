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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RechargeRule {
    OnShortRest,
    OnLongRest,
    OnAnyRest,
    // TODO: Daily is the same as long rest?
    Daily,
    None,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub kind: String,
    pub max_uses: u8,
    pub current_uses: u8,
    pub recharge: RechargeRule,
}

impl Resource {
    /// Attempt to use `amount` points from this resource. Returns Err if not enough left.
    pub fn spend(&mut self, amount: u8) -> Result<(), SpendError> {
        if self.current_uses < amount {
            Err(SpendError::InsufficientResources {
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
    pub fn recharge(&mut self) {
        self.current_uses = self.max_uses;
    }

    pub fn is_empty(&self) -> bool {
        self.current_uses == 0
    }

    pub fn add_uses(&mut self, amount: u8) -> Result<(), SpendError> {
        self.max_uses += amount;
        self.current_uses += amount;
        Ok(())
    }

    pub fn add_use(&mut self) -> Result<(), SpendError> {
        self.add_uses(1)
    }

    pub fn remove_uses(&mut self, amount: u8) -> Result<(), SpendError> {
        if self.max_uses < amount {
            Err(SpendError::InsufficientResources {
                kind: self.kind.clone(),
                needed: amount,
                available: self.max_uses,
            })
        } else {
            self.max_uses -= amount;
            if self.current_uses > self.max_uses {
                self.current_uses = self.max_uses;
            }
            Ok(())
        }
    }

    pub fn remove_use(&mut self) -> Result<(), SpendError> {
        self.remove_uses(1)
    }
}

#[derive(Debug)]
pub enum SpendError {
    InsufficientResources {
        kind: String,
        needed: u8,
        available: u8,
    },
}
