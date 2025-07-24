#[derive(Debug, Clone)]
pub struct HitPoints {
    current: u32,
    max: u32,
}

impl HitPoints {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn with_current(current: u32, max: u32) -> Self {
        Self { current, max }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn max(&self) -> u32 {
        self.max
    }

    pub fn update_max(&mut self, new_max: u32) {
        if new_max < self.current {
            self.current = new_max;
        }
        self.max = new_max;
    }

    pub(crate) fn damage(&mut self, amount: u32) {
        if amount >= self.current {
            self.current = 0;
        } else {
            self.current -= amount;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_current_and_max() {
        let hp = HitPoints::new(10);
        assert_eq!(hp.current(), 10);
        assert_eq!(hp.max(), 10);
    }

    #[test]
    fn with_current_initializes_current_and_max() {
        let hp = HitPoints::with_current(5, 10);
        assert_eq!(hp.current(), 5);
        assert_eq!(hp.max(), 10);
    }

    #[test]
    fn damage_reduces_current() {
        let mut hp = HitPoints::new(10);
        hp.damage(3);
        assert_eq!(hp.current(), 7);
    }

    #[test]
    fn damage_does_not_go_below_zero() {
        let mut hp = HitPoints::new(5);
        hp.damage(10);
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
    fn damage_exact_to_zero() {
        let mut hp = HitPoints::new(4);
        hp.damage(4);
        assert_eq!(hp.current(), 0);
    }
}
