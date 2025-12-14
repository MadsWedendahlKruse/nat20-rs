// TODO: Consider a different name?

use std::collections::HashMap;

use uom::si::{f32::Length, length::meter};

use crate::components::modifier::ModifierSource;

// Internally, speed is stored in meters (per turn).
#[derive(Debug, Clone)]
pub struct Speed {
    flat: HashMap<ModifierSource, f32>,
    multipliers: HashMap<ModifierSource, f32>,
    moved_this_turn: f32,
}

impl Speed {
    // Construct a new Speed with a base value from any length unit
    pub fn new(base: Length) -> Self {
        let mut flat = HashMap::new();
        flat.insert(ModifierSource::Base, base.get::<meter>());
        Self {
            flat,
            multipliers: HashMap::new(),
            moved_this_turn: 0.0,
        }
    }

    pub fn add_flat_modifier<T>(&mut self, source: ModifierSource, value: T)
    where
        T: Into<f32>,
    {
        self.flat.insert(source, value.into());
    }

    pub fn remove_flat_modifier(&mut self, source: &ModifierSource) {
        self.flat.remove(source);
    }

    pub fn add_multiplier<T>(&mut self, source: ModifierSource, value: T)
    where
        T: Into<f32>,
    {
        self.multipliers.insert(source, value.into());
    }

    pub fn remove_multiplier(&mut self, source: &ModifierSource) {
        self.multipliers.remove(source);
    }

    pub fn get_total_speed(&self) -> Length {
        let base_speed: f32 = self.flat.values().sum();

        let total_multiplier: f32 = if self.multipliers.is_empty() {
            1.0
        } else {
            self.multipliers.values().product()
        };

        Length::new::<meter>(base_speed * total_multiplier)
    }

    pub fn moved_this_turn(&self) -> Length {
        Length::new::<meter>(self.moved_this_turn)
    }

    pub fn record_movement(&mut self, distance: Length) {
        let distance = distance.get::<meter>();
        if distance > self.remaining_movement().get::<meter>() {
            self.moved_this_turn = self.get_total_speed().get::<meter>();
        } else {
            self.moved_this_turn += distance;
        }
    }

    /// Should be called at the start (or end?) of each turn
    pub fn reset(&mut self) {
        self.moved_this_turn = 0.0;
    }

    pub fn remaining_movement(&self) -> Length {
        let total_speed = self.get_total_speed().get::<meter>();
        let remaining = (total_speed - self.moved_this_turn).max(0.0);
        Length::new::<meter>(remaining)
    }

    pub fn can_move(&self) -> bool {
        self.remaining_movement().get::<meter>() > 0.0
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(Length::new::<meter>(10.0))
    }
}

#[cfg(test)]
mod tests {
    use crate::components::id::{EffectId, ItemId};

    use super::*;

    #[test]
    fn new_speed() {
        let speed = Speed::default();
        assert_eq!(speed.get_total_speed().get::<meter>(), 10.0);
        assert_eq!(speed.moved_this_turn().get::<meter>(), 0.0);
    }

    #[test]
    fn add_flat_modifier() {
        let mut speed = Speed::default();
        speed.add_flat_modifier(
            ModifierSource::Item(ItemId::new("nat20_rs","Boots of Speed!")),
            5.0,
        );
        assert_eq!(speed.get_total_speed().get::<meter>(), 15.0);
    }

    #[test]
    fn remove_flat_modifier() {
        let mut speed = Speed::default();
        speed.add_flat_modifier(
            ModifierSource::Item(ItemId::new("nat20_rs","Boots of Speed!")),
            5.0,
        );
        speed.remove_flat_modifier(&ModifierSource::Item(ItemId::new("nat20_rs","Boots of Speed!")));
        assert_eq!(speed.get_total_speed().get::<meter>(), 10.0);
    }

    #[test]
    fn add_multiplier() {
        let mut speed = Speed::default();
        speed.add_multiplier(
            ModifierSource::Effect(EffectId::new("nat20_rs","Expeditious Retreat!")),
            2.0,
        );
        assert_eq!(speed.get_total_speed().get::<meter>(), 20.0);
    }

    #[test]
    fn remove_multiplier() {
        let mut speed = Speed::default();
        speed.add_multiplier(
            ModifierSource::Effect(EffectId::new("nat20_rs","Expeditious Retreat!")),
            2.0,
        );
        speed.remove_multiplier(&ModifierSource::Effect(EffectId::new("nat20_rs",
            "Expeditious Retreat!",
        )));
        assert_eq!(speed.get_total_speed().get::<meter>(), 10.0);
    }

    #[test]
    fn record_movement_and_remaining() {
        let mut speed = Speed::default();
        speed.record_movement(Length::new::<meter>(3.0));
        assert_eq!(speed.moved_this_turn().get::<meter>(), 3.0);
        assert_eq!(speed.remaining_movement().get::<meter>(), 7.0);
    }

    #[test]
    fn reset() {
        let mut speed = Speed::default();
        speed.record_movement(Length::new::<meter>(5.0));
        speed.reset();
        assert_eq!(speed.moved_this_turn().get::<meter>(), 0.0);
    }

    #[test]
    fn can_move() {
        let mut speed = Speed::default();
        assert!(speed.can_move());
        speed.record_movement(Length::new::<meter>(10.0));
        assert!(!speed.can_move());
    }

    #[test]
    fn total_speed_with_zero_multiplier() {
        let mut speed = Speed::default();
        speed.add_multiplier(ModifierSource::Effect(EffectId::new("nat20_rs","Fear!")), 0.0);
        assert_eq!(speed.get_total_speed().get::<meter>(), 0.0);
    }

    #[test]
    fn flat_and_multiplier_combination() {
        let mut speed = Speed::default();
        speed.add_flat_modifier(
            ModifierSource::Item(ItemId::new("nat20_rs","Boots of Speed!")),
            5.0,
        );
        speed.add_multiplier(
            ModifierSource::Effect(EffectId::new("nat20_rs","Expeditious Retreat!")),
            2.0,
        );
        assert_eq!(speed.get_total_speed().get::<meter>(), 30.0);
    }
}
