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

    pub fn bonus(&self, prof_bonus: i32) -> i32 {
        (self.multiplier() * prof_bonus as f32).floor() as i32
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
    fn test_proficiency_bonus() {
        let prof = Proficiency::Proficient;
        assert_eq!(prof.bonus(2), 2);
        assert_eq!(prof.bonus(3), 3);
    }

    #[test]
    fn test_expertise_bonus() {
        let prof = Proficiency::Expertise;
        assert_eq!(prof.bonus(2), 4);
        assert_eq!(prof.bonus(3), 6);
    }

    #[test]
    fn test_half_bonus() {
        let prof = Proficiency::Half;
        assert_eq!(prof.bonus(2), 1);
        assert_eq!(prof.bonus(3), 1);
    }

    #[test]
    fn test_none_bonus() {
        let prof = Proficiency::None;
        assert_eq!(prof.bonus(2), 0);
        assert_eq!(prof.bonus(3), 0);
    }
}
