#[derive(Debug, Clone)]
pub struct HitPoints {
    current: u32,
    max: u32,
    temp: u32,
}

impl HitPoints {
    pub fn new(max: u32) -> Self {
        Self {
            current: max,
            max,
            temp: 0,
        }
    }

    pub fn with_current(current: u32, max: u32) -> Self {
        Self {
            current,
            max,
            temp: 0,
        }
    }

    pub fn with_temp(current: u32, max: u32, temp: u32) -> Self {
        Self { current, max, temp }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn max(&self) -> u32 {
        self.max
    }

    pub fn temp(&self) -> u32 {
        self.temp
    }

    pub fn update_max(&mut self, new_max: u32) {
        if new_max < self.current {
            self.current = new_max;
        }
        self.max = new_max;
    }

    pub(crate) fn damage(&mut self, amount: u32) {
        // Damage is applied to temp HP first, then to current HP
        let temp_damage = amount.min(self.temp);
        self.temp -= temp_damage;
        let remaining = amount - temp_damage;
        if remaining >= self.current {
            self.current = 0;
        } else {
            self.current -= remaining;
        }
    }

    pub(crate) fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub(crate) fn heal_full(&mut self) {
        self.current = self.max;
    }

    pub fn is_full(&self) -> bool {
        self.current == self.max
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    /// Sets temporary hit points. If the new value is higher than the current
    /// temp HP, replaces it. If lower or equal, does nothing (DND 5e rules).
    pub fn set_temp(&mut self, temp: u32) {
        if temp > self.temp {
            self.temp = temp;
        }
    }

    pub fn clear_temp(&mut self) {
        self.temp = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_current_and_max() {
        let hp = HitPoints::new(10);
        assert_eq!(hp.current(), 10);
        assert_eq!(hp.max(), 10);
        assert_eq!(hp.temp(), 0);
    }

    #[test]
    fn with_current_initializes_current_and_max() {
        let hp = HitPoints::with_current(5, 10);
        assert_eq!(hp.current(), 5);
        assert_eq!(hp.max(), 10);
        assert_eq!(hp.temp(), 0);
    }

    #[test]
    fn with_temp_initializes_all_fields() {
        let hp = HitPoints::with_temp(5, 10, 7);
        assert_eq!(hp.current(), 5);
        assert_eq!(hp.max(), 10);
        assert_eq!(hp.temp(), 7);
    }

    #[test]
    fn damage_reduces_temp_first_then_current() {
        let mut hp = HitPoints::with_temp(10, 10, 5);
        hp.damage(3);
        assert_eq!(hp.temp(), 2);
        assert_eq!(hp.current(), 10);

        hp.damage(4); // 2 temp left, so 2 temp gone, 2 to current
        assert_eq!(hp.temp(), 0);
        assert_eq!(hp.current(), 8);
    }

    #[test]
    fn damage_does_not_go_below_zero() {
        let mut hp = HitPoints::with_temp(5, 5, 3);
        hp.damage(10);
        assert_eq!(hp.temp(), 0);
        assert_eq!(hp.current(), 0);
    }

    #[test]
    fn heal_increases_current_but_not_above_max() {
        let mut hp = HitPoints::new(10);
        hp.damage(5);
        hp.heal(3);
        assert_eq!(hp.current(), 8);

        hp.heal(10);
        assert_eq!(hp.current(), 10);
    }

    #[test]
    fn heal_when_at_max_does_nothing() {
        let mut hp = HitPoints::new(7);
        hp.heal(5);
        assert_eq!(hp.current(), 7);
    }

    #[test]
    fn damage_exact_to_zero_with_temp() {
        let mut hp = HitPoints::with_temp(4, 4, 2);
        hp.damage(6);
        assert_eq!(hp.temp(), 0);
        assert_eq!(hp.current(), 0);
    }

    #[test]
    fn set_temp_only_increases_temp() {
        let mut hp = HitPoints::new(10);
        hp.set_temp(5);
        assert_eq!(hp.temp(), 5);
        hp.set_temp(3);
        assert_eq!(hp.temp(), 5);
        hp.set_temp(8);
        assert_eq!(hp.temp(), 8);
    }

    #[test]
    fn clear_temp_sets_temp_to_zero() {
        let mut hp = HitPoints::with_temp(10, 10, 7);
        hp.clear_temp();
        assert_eq!(hp.temp(), 0);
    }
}
