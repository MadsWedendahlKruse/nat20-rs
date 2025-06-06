use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proficiency {
    None,
    Proficient,
    Expertise,
    Half, // Optional: for features like Bard’s Jack of All Trades
}

impl Proficiency {
    pub fn multiplier(&self) -> f32 {
        match self {
            Proficiency::None => 0.0,
            Proficiency::Half => 0.5,
            Proficiency::Proficient => 1.0,
            Proficiency::Expertise => 2.0,
        }
    }

    pub fn bonus(&self, proficiency_bonus: u32) -> u32 {
        (self.multiplier() * proficiency_bonus as f32).floor() as u32
    }
}

impl fmt::Display for Proficiency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proficiency_bonus() {
        let prof = Proficiency::Proficient;
        assert_eq!(prof.bonus(2), 2);
        assert_eq!(prof.bonus(3), 3);
    }

    #[test]
    fn expertise_bonus() {
        let prof = Proficiency::Expertise;
        assert_eq!(prof.bonus(2), 4);
        assert_eq!(prof.bonus(3), 6);
    }

    #[test]
    fn half_bonus() {
        let prof = Proficiency::Half;
        assert_eq!(prof.bonus(2), 1);
        assert_eq!(prof.bonus(3), 1);
    }

    #[test]
    fn none_bonus() {
        let prof = Proficiency::None;
        assert_eq!(prof.bonus(2), 0);
        assert_eq!(prof.bonus(3), 0);
    }
}
