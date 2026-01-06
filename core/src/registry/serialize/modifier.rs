use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::Ability,
        d20::AdvantageType,
        damage::{DamageSource, DamageType, MitigationOperation},
        saving_throw::SavingThrowKind,
        skill::Skill,
    },
    registry::serialize::quantity::LengthExpressionDefinition,
};

/// For spec types that:
/// - implement `FromStr<Err = String>`
/// - have a `pub raw: String` field
macro_rules! impl_string_backed_spec {
    ($type:ty) => {
        impl TryFrom<String> for $type {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl From<$type> for String {
            fn from(spec: $type) -> Self {
                spec.raw
            }
        }
    };
}

/// For types that:
/// - implement `FromStr<Err = String>`
/// - implement `Display`
/// - and you want Serde round-trip via `String`
macro_rules! impl_display_roundtrip_spec {
    ($type:ty) => {
        impl TryFrom<String> for $type {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl From<$type> for String {
            fn from(value: $type) -> Self {
                value.to_string()
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct D20CheckModifierProvider<T: Clone + DeserializeOwned + IntoEnumIterator> {
    #[serde(skip)]
    pub kind: Vec<T>,
    #[serde(skip)]
    pub delta: Option<i32>,
    #[serde(skip)]
    pub advantage: Option<AdvantageType>,
    pub raw: String,
}

impl<T> FromStr for D20CheckModifierProvider<T>
where
    T: Clone + DeserializeOwned + IntoEnumIterator,
{
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Examples:
        // "perception advantage"
        // "stealth-1"
        // "investigation+2"
        // "strength disadvantage"

        let normalized = normalize_spec_string(input);

        let (check_str, modifier_str) =
            split_first_delimiter(&normalized, &[' ', '+', '-'], "D20CheckModifierProvider")?;

        if check_str.to_lowercase().eq("all") {
            let (delta, advantage) = parse_delta_or_advantage(modifier_str, &normalized)?;

            return Ok(D20CheckModifierProvider {
                raw: normalized,
                kind: T::iter().collect(),
                delta,
                advantage,
            });
        }

        let kind: T = parse_plain_enum(check_str, "check kind", &normalized)?;

        let (delta, advantage) = parse_delta_or_advantage(modifier_str, &normalized)?;

        Ok(D20CheckModifierProvider {
            raw: normalized,
            kind: vec![kind],
            delta,
            advantage,
        })
    }
}

impl<T> TryFrom<String> for D20CheckModifierProvider<T>
where
    T: Clone + DeserializeOwned + IntoEnumIterator,
{
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<T> From<D20CheckModifierProvider<T>> for String
where
    T: Clone + DeserializeOwned + IntoEnumIterator,
{
    fn from(spec: D20CheckModifierProvider<T>) -> Self {
        spec.raw
    }
}

pub type SkillModifierProvider = D20CheckModifierProvider<Skill>;
pub type SavingThrowModifierProvider = D20CheckModifierProvider<SavingThrowKind>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AbilityModifierProvider {
    #[serde(skip)]
    pub ability: Ability,
    #[serde(skip)]
    pub delta: i32,
    pub raw: String,
}

impl FromStr for AbilityModifierProvider {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Examples:
        // "strength+2"
        // "intelligence-1"

        let (name, delta_part) =
            split_first_delimiter(input, &['+', '-'], "AbilityModifierProvider")?;

        let ability: Ability = parse_plain_enum(name, "ability", input)?;

        let delta: i32 = delta_part
            .parse()
            .map_err(|_| format!("Invalid modifier in '{}'", input))?;

        Ok(AbilityModifierProvider {
            raw: input.to_string(),
            ability,
            delta,
        })
    }
}

impl_string_backed_spec!(AbilityModifierProvider);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DamageResistanceProvider {
    #[serde(skip)]
    pub damage_type: DamageType,
    #[serde(skip)]
    pub operation: MitigationOperation,
    pub raw: String,
}

impl FromStr for DamageResistanceProvider {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Examples:
        // "fire resistance"
        // "cold immunity"
        // "force -2"
        let normalized = normalize_spec_string(input);
        let parts: Vec<&str> = normalized.split_whitespace().collect();

        if parts.len() != 2 {
            return Err(format!("Invalid DamageResistanceProvider: {}", input));
        }

        let damage_type: DamageType = parse_plain_enum(parts[0].trim(), "damage type", input)?;

        let operation: MitigationOperation =
            if let Ok(operation) = serde_plain::from_str(parts[1].trim()) {
                operation
            } else if parts[1].trim().starts_with('-') {
                let amount_str = parts[1].trim().trim_start_matches('-');
                let amount: i32 = amount_str
                    .parse()
                    .map_err(|_| format!("Invalid flat reduction amount in '{}'", input))?;
                MitigationOperation::FlatReduction(amount)
            } else {
                return Err(format!("Unknown mitigation operation in '{}'", input));
            };

        Ok(DamageResistanceProvider {
            raw: input.to_string(),
            damage_type,
            operation,
        })
    }
}

impl_string_backed_spec!(DamageResistanceProvider);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum AttackRollModifier {
    FlatBonus(i32),
    Advantage(AdvantageType),
    CritThreshold(u8),
}

impl Display for AttackRollModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttackRollModifier::FlatBonus(bonus) => {
                if *bonus >= 0 {
                    write!(f, "+{}", bonus)
                } else {
                    write!(f, "{}", bonus)
                }
            }
            AttackRollModifier::Advantage(advantage) => {
                write!(f, "{}", serde_plain::to_string(advantage).unwrap())
            }
            AttackRollModifier::CritThreshold(modifier) => {
                write!(f, "crit(-{})", modifier)
            }
        }
    }
}

impl FromStr for AttackRollModifier {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_spec_string(input);

        if let Ok(advantage) = serde_plain::from_str::<AdvantageType>(&normalized) {
            return Ok(AttackRollModifier::Advantage(advantage));
        }

        if normalized.starts_with("crit(-") && normalized.ends_with(')') {
            let inner = &normalized[6..normalized.len() - 1];
            let threshold: u8 = inner
                .parse()
                .map_err(|_| format!("Invalid crit threshold in '{}'", input))?;
            return Ok(AttackRollModifier::CritThreshold(threshold));
        }

        let bonus: i32 = normalized
            .parse()
            .map_err(|_| format!("Invalid flat bonus in '{}'", input))?;

        Ok(AttackRollModifier::FlatBonus(bonus))
    }
}

impl_display_roundtrip_spec!(AttackRollModifier);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AttackRollModifierProvider {
    #[serde(skip)]
    pub source: Option<DamageSource>,
    #[serde(skip)]
    pub modifier: Option<AttackRollModifier>,
    pub raw: String,
}

impl FromStr for AttackRollModifierProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Examples:
        // "ranged +2"
        // "spell advantage"
        // "crit(-2)"
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        let (source, modifier) = if parts.len() == 1 {
            if let Ok(modifier) = parts[0].trim().parse::<AttackRollModifier>() {
                (None, Some(modifier))
            } else {
                return Err(format!("Invalid AttackRollModifierProvider: {}", s));
            }
        } else if parts.len() == 2 {
            let source_str = parts[0].trim();
            let modifier_str = parts[1].trim();

            let source = match serde_plain::from_str::<DamageSource>(source_str) {
                Ok(src) => Some(src),
                Err(_) => return Err(format!("Unknown damage source in '{}'", s)),
            };

            let modifier = match modifier_str.parse::<AttackRollModifier>() {
                Ok(modifier) => Some(modifier),
                Err(e) => return Err(e),
            };

            (source, modifier)
        } else {
            (None, None)
        };

        Ok(AttackRollModifierProvider {
            raw: s.to_string(),
            source,
            modifier,
        })
    }
}

impl_string_backed_spec!(AttackRollModifierProvider);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ArmorClassModifierProvider {
    #[serde(skip)]
    pub delta: i32,
    pub raw: String,
}

impl FromStr for ArmorClassModifierProvider {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_spec_string(input);

        let delta: i32 = normalized
            .parse()
            .map_err(|_| format!("Invalid modifier in '{}'", input))?;

        Ok(ArmorClassModifierProvider {
            raw: normalized,
            delta,
        })
    }
}

impl_string_backed_spec!(ArmorClassModifierProvider);

fn find_index_of_first(s: &str, chars: &[char]) -> Option<usize> {
    s.char_indices()
        .find(|(_, c)| chars.contains(c))
        .map(|(i, _)| i)
}

fn normalize_spec_string(input: &str) -> String {
    input.trim().to_lowercase()
}

fn split_first_delimiter<'a>(
    s: &'a str,
    delimiters: &[char],
    kind_name: &str,
) -> Result<(&'a str, &'a str), String> {
    if let Some(index) = find_index_of_first(s, delimiters) {
        let (left, right) = s.split_at(index);
        Ok((left.trim(), right.trim()))
    } else {
        Err(format!("Invalid {} spec: {}", kind_name, s))
    }
}

fn parse_plain_enum<T>(name: &str, field_name: &str, whole: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_plain::from_str(name).map_err(|_| format!("Unknown {} in '{}'", field_name, whole))
}

fn parse_delta_or_advantage(
    modifier_str: &str,
    full_input: &str,
) -> Result<(Option<i32>, Option<AdvantageType>), String> {
    let modifier_str = modifier_str.trim();
    match modifier_str {
        "" => Ok((None, None)),
        "advantage" => Ok((None, Some(AdvantageType::Advantage))),
        "disadvantage" => Ok((None, Some(AdvantageType::Disadvantage))),
        _ => {
            let delta: i32 = modifier_str
                .parse()
                .map_err(|_| format!("Invalid modifier in '{}'", full_input))?;
            Ok((Some(delta), None))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SpeedModifier {
    Flat(LengthExpressionDefinition),
    Multiplier(f32),
}

impl Display for SpeedModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpeedModifier::Flat(value) => {
                write!(f, "{}", value)
            }
            SpeedModifier::Multiplier(value) => {
                write!(f, "x{}", value)
            }
        }
    }
}

impl FromStr for SpeedModifier {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_spec_string(input);

        if normalized.starts_with('x') {
            let multiplier_str = &normalized[1..];
            let multiplier: f32 = multiplier_str
                .parse()
                .map_err(|_| format!("Invalid speed multiplier in '{}'", input))?;
            Ok(SpeedModifier::Multiplier(multiplier))
        } else {
            let length_expr: LengthExpressionDefinition = normalized
                .parse()
                .map_err(|e| format!("Invalid length expression in '{}': {}", input, e))?;
            Ok(SpeedModifier::Flat(length_expr))
        }
    }
}

impl_display_roundtrip_spec!(SpeedModifier);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SpeedModifierProvider {
    pub modifier: SpeedModifier,
    pub raw: String,
}

impl FromStr for SpeedModifierProvider {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let modifier: SpeedModifier = input.parse()?;

        Ok(SpeedModifierProvider {
            raw: input.to_string(),
            modifier,
        })
    }
}

impl_string_backed_spec!(SpeedModifierProvider);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_modifier_provider_parsing() {
        let spec: SkillModifierProvider = "stealth+2".parse().unwrap();
        assert_eq!(spec.kind[0], Skill::Stealth);
        assert_eq!(spec.delta, Some(2));

        let spec: SkillModifierProvider = "athletics-1".parse().unwrap();
        assert_eq!(spec.kind[0], Skill::Athletics);
        assert_eq!(spec.delta, Some(-1));

        let spec: SkillModifierProvider = "perception advantage".parse().unwrap();
        assert_eq!(spec.kind[0], Skill::Perception);
        assert_eq!(spec.advantage, Some(AdvantageType::Advantage));

        let spec: SkillModifierProvider = "all disadvantage".parse().unwrap();
        assert_eq!(spec.kind.len(), Skill::iter().count());
        assert_eq!(spec.advantage, Some(AdvantageType::Disadvantage));
    }

    #[test]
    fn test_saving_throw_modifier_provider_parsing() {
        let spec: SavingThrowModifierProvider = "constitution advantage".parse().unwrap();
        assert_eq!(
            spec.kind[0],
            SavingThrowKind::Ability(Ability::Constitution)
        );
        assert_eq!(spec.advantage, Some(AdvantageType::Advantage));
        assert_eq!(spec.delta, None);

        let spec: SavingThrowModifierProvider = "dex disadvantage".parse().unwrap();
        assert_eq!(spec.kind[0], SavingThrowKind::Ability(Ability::Dexterity));
        assert_eq!(spec.advantage, Some(AdvantageType::Disadvantage));
        assert_eq!(spec.delta, None);

        let spec: SavingThrowModifierProvider = "wis+2".parse().unwrap();
        assert_eq!(spec.kind[0], SavingThrowKind::Ability(Ability::Wisdom));
        assert_eq!(spec.delta, Some(2));
        assert_eq!(spec.advantage, None);

        let spec: SavingThrowModifierProvider = "all-1".parse().unwrap();
        assert_eq!(spec.kind.len(), SavingThrowKind::iter().count());
        assert_eq!(spec.delta, Some(-1));
    }

    #[test]
    fn test_ability_modifier_provider_parsing() {
        let spec: AbilityModifierProvider = "strength+2".parse().unwrap();
        assert_eq!(spec.ability, Ability::Strength);
        assert_eq!(spec.delta, 2);

        let spec: AbilityModifierProvider = "intelligence-1".parse().unwrap();
        assert_eq!(spec.ability, Ability::Intelligence);
        assert_eq!(spec.delta, -1);
    }

    #[test]
    fn test_damage_resistance_provider_parsing() {
        let spec: DamageResistanceProvider = "fire resistance".parse().unwrap();
        assert_eq!(spec.damage_type, DamageType::Fire);
        assert_eq!(spec.operation, MitigationOperation::Resistance);

        let spec: DamageResistanceProvider = "cold immunity".parse().unwrap();
        assert_eq!(spec.damage_type, DamageType::Cold);
        assert_eq!(spec.operation, MitigationOperation::Immunity);

        let spec: DamageResistanceProvider = "force -2".parse().unwrap();
        assert_eq!(spec.damage_type, DamageType::Force);
        assert_eq!(spec.operation, MitigationOperation::FlatReduction(2));
    }
}
