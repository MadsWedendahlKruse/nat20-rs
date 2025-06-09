use std::collections::HashMap;

use strum::EnumIter;

use crate::resources::resources::{RechargeRule, Resource, ResourceError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ActionResource {
    Action,
    BonusAction,
    Reaction,
}

#[derive(Debug, Clone)]
pub struct ActionEconomy {
    resources: HashMap<ActionResource, Resource>,
}

impl ActionEconomy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_uses(actions: u8, bonus_actions: u8, reactions: u8) -> Self {
        let mut resources = HashMap::new();
        resources.insert(
            ActionResource::Action,
            Resource::new("resource.action", actions, RechargeRule::OnTurn).unwrap(),
        );
        resources.insert(
            ActionResource::BonusAction,
            Resource::new("resource.bonus_action", bonus_actions, RechargeRule::OnTurn).unwrap(),
        );
        resources.insert(
            ActionResource::Reaction,
            Resource::new("resource.reaction", reactions, RechargeRule::OnTurn).unwrap(),
        );
        Self { resources }
    }

    pub fn get(&self, action: ActionResource) -> &Resource {
        // We know the action must exist, so unwrap is safe
        self.resources.get(&action).unwrap()
    }

    pub fn get_mut(&mut self, action: ActionResource) -> &mut Resource {
        // We know the action must exist, so unwrap is safe
        self.resources.get_mut(&action).unwrap()
    }

    pub fn spend(&mut self, action: ActionResource, amount: u8) -> Result<(), ResourceError> {
        let resource = self.get_mut(action);
        resource.spend(amount)
    }

    /// Spend all available uses of all resources.
    /// This is mostly used for crowd control effects that prevent all actions.
    pub fn spend_all(&mut self) -> Result<(), ResourceError> {
        for (_, resource) in self.resources.iter_mut() {
            resource.spend(resource.current_uses())?;
        }
        Ok(())
    }

    pub fn recharge_all(&mut self, rest_type: RechargeRule) {
        for resource in self.resources.values_mut() {
            resource.recharge(rest_type);
        }
    }
}

impl Default for ActionEconomy {
    fn default() -> Self {
        Self::with_max_uses(1, 1, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_action_economy() {
        let ae = ActionEconomy::new();
        assert_eq!(ae.get(ActionResource::Action).max_uses(), 1);
        assert_eq!(ae.get(ActionResource::BonusAction).max_uses(), 1);
        assert_eq!(ae.get(ActionResource::Reaction).max_uses(), 1);
    }

    #[test]
    fn test_with_max_uses() {
        let ae = ActionEconomy::with_max_uses(2, 3, 4);
        assert_eq!(ae.get(ActionResource::Action).max_uses(), 2);
        assert_eq!(ae.get(ActionResource::BonusAction).max_uses(), 3);
        assert_eq!(ae.get(ActionResource::Reaction).max_uses(), 4);
    }

    #[test]
    fn test_spend_action() {
        let mut ae = ActionEconomy::with_max_uses(2, 1, 1);
        assert_eq!(ae.spend(ActionResource::Action, 1), Ok(()));
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 1);
        assert_eq!(ae.spend(ActionResource::Action, 1), Ok(()));
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 0);
        assert_eq!(
            ae.spend(ActionResource::Action, 1),
            Err(ResourceError::InsufficientResources {
                kind: "resource.action".to_string(),
                needed: 1,
                available: 0
            })
        );
    }

    #[test]
    fn test_spend_all() {
        let mut ae = ActionEconomy::with_max_uses(2, 1, 1);
        assert_eq!(ae.spend_all(), Ok(()));
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 0);
        assert_eq!(ae.get(ActionResource::BonusAction).current_uses(), 0);
        assert_eq!(ae.get(ActionResource::Reaction).current_uses(), 0);
    }

    #[test]
    fn test_recharge_all() {
        let mut ae = ActionEconomy::with_max_uses(2, 1, 1);
        ae.spend_all().unwrap();
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 0);
        ae.recharge_all(RechargeRule::OnTurn);
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 2);
        assert_eq!(ae.get(ActionResource::BonusAction).current_uses(), 1);
        assert_eq!(ae.get(ActionResource::Reaction).current_uses(), 1);
    }

    #[test]
    fn test_get_and_get_mut() {
        let mut ae = ActionEconomy::default();
        let action = ae.get(ActionResource::Action);
        assert_eq!(action.max_uses(), 1);

        let action_mut = ae.get_mut(ActionResource::Action);
        action_mut.spend(1).unwrap();
        assert_eq!(ae.get(ActionResource::Action).current_uses(), 0);
    }
}
