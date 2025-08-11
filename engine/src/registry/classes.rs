use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use crate::{
    components::{
        ability::{Ability, AbilityScoreDistribution},
        class::{Class, ClassBase, ClassName, SpellcastingProgression, Subclass, SubclassName},
        dice::DieSize,
        items::equipment::{armor::ArmorType, weapon::WeaponCategory},
        level_up::LevelUpPrompt,
        resource::{RechargeRule, Resource},
        skill::Skill,
    },
    registry,
};

pub static CLASS_REGISTRY: LazyLock<HashMap<ClassName, Class>> = LazyLock::new(|| {
    HashMap::from([
        // (ClassName::Barbarian, BARBARIAN.to_owned()),
        // (ClassName::Bard, BARD.to_owned()),
        // (ClassName::Cleric, CLERIC.to_owned()),
        // (ClassName::Druid, DRUID.to_owned()),
        (ClassName::Fighter, FIGHTER.to_owned()),
        // (ClassName::Monk, MONK.to_owned()),
        // (ClassName::Paladin, PALADIN.to_owned()),
        // (ClassName::Ranger, RANGER.to_owned()),
        // (ClassName::Rogue, ROGUE.to_owned()),
        // (ClassName::Sorcerer, SORCERER.to_owned()),
        (ClassName::Warlock, WARLOCK.to_owned()),
        (ClassName::Wizard, WIZARD.to_owned()),
    ])
});

// TODO-LIST
// [x] Level 1: Second Wind
// [ ] Level 1: Weapon Mastery
// [x] Level 2: Action Surge
// [ ] Level 2: Tactical Mind
// [x] Level 3: Fighter Subclass
// [x] Level 4: Ability Score Improvement
// [x] Level 5: Extra Attack
// [ ] Level 5: Tactical Shift
// [ ] Level 9: Indomitable
//     Requires that a reaction can be triggered outside of combat
// [ ] Level 9: Tactical Master
// [x] Level 11: Two Extra Attacks
// [ ] Level 13: Studied Attacks
// [ ] Level 19: Epic Boon
// [x] Level 20: Three Extra Attacks
static FIGHTER: LazyLock<Class> = LazyLock::new(|| {
    Class::new(
        ClassName::Fighter,
        DieSize::D10,
        6,
        AbilityScoreDistribution {
            scores: HashMap::from([
                (Ability::Strength, 15),
                (Ability::Dexterity, 14),
                (Ability::Constitution, 13),
                (Ability::Intelligence, 8),
                (Ability::Wisdom, 10),
                (Ability::Charisma, 12),
            ]),
            plus_2_bonus: Ability::Strength,
            plus_1_bonus: Ability::Constitution,
        },
        [Ability::Strength, Ability::Constitution],
        3,
        HashMap::from([(CHAMPION.name.clone(), CHAMPION.to_owned())]),
        HashSet::from([4, 6, 8, 12, 14, 16]),
        HashSet::from([
            Skill::Acrobatics,
            Skill::AnimalHandling,
            Skill::Athletics,
            Skill::History,
            Skill::Insight,
            Skill::Intimidation,
            Skill::Perception,
            Skill::Survival,
        ]),
        2,
        HashSet::from([ArmorType::Light, ArmorType::Medium, ArmorType::Heavy]),
        HashSet::from([WeaponCategory::Simple, WeaponCategory::Martial]),
        SpellcastingProgression::None,
        HashMap::from([
            (5, vec![registry::effects::EXTRA_ATTACK_ID.clone()]),
            (11, vec![registry::effects::TWO_EXTRA_ATTACKS_ID.clone()]),
            (20, vec![registry::effects::THREE_EXTRA_ATTACKS_ID.clone()]),
        ]),
        HashMap::from([
            (1, vec![second_wind(2)]),
            (
                2,
                vec![
                    Resource::new(
                        registry::resources::ACTION_SURGE.clone(),
                        1,
                        RechargeRule::OnShortRest,
                    )
                    .unwrap(),
                ],
            ),
            (4, vec![second_wind(3)]),
            (10, vec![second_wind(4)]),
            (
                17,
                vec![
                    Resource::new(
                        registry::resources::ACTION_SURGE.clone(),
                        2,
                        RechargeRule::OnShortRest,
                    )
                    .unwrap(),
                ],
            ),
        ]),
        HashMap::from([(
            1,
            vec![LevelUpPrompt::Feat(vec![
                registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
                registry::feats::FIGHTING_STYLE_DEFENSE_ID.clone(),
                registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
            ])],
        )]),
        HashMap::from([
            (1, vec![registry::actions::SECOND_WIND_ID.clone()]),
            (2, vec![registry::actions::ACTION_SURGE_ID.clone()]),
        ]),
    )
});

fn second_wind(charges: u8) -> Resource {
    Resource::new(
        registry::resources::SECOND_WIND.clone(),
        charges,
        RechargeRule::OnShortRest,
    )
    .unwrap()
}

// [x] Level 3: Improved Critical
// [~] Level 3: Remarkable Athlete
//     Missing the part about crits not provoking opportunity attacks
// [x] Level 7: Additional Fighting Style
// [ ] Level 10: Heroic Warrior
///    Pretty complicated to implement
// [x] Level 15: Superior Critical
// [ ] Level 18: Survivor
static CHAMPION: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Fighter,
        name: "Champion".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_prompts: 0,
        armor_proficiencies: HashSet::new(),
        weapon_proficiencies: HashSet::new(),
        spellcasting: SpellcastingProgression::None,
        effects_by_level: HashMap::from([
            (
                3,
                vec![
                    registry::effects::IMPROVED_CRITICAL_ID.clone(),
                    registry::effects::REMARKABLE_ATHLETE_ID.clone(),
                ],
            ),
            (15, vec![registry::effects::SUPERIOR_CRITICAL_ID.clone()]),
        ]),
        resources_by_level: HashMap::new(),
        prompts_by_level: HashMap::from([(
            7,
            vec![LevelUpPrompt::Feat(vec![
                registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
                registry::feats::FIGHTING_STYLE_DEFENSE_ID.clone(),
                registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
            ])],
        )]),
        actions_by_level: HashMap::new(),
    },
});

static WARLOCK: LazyLock<Class> = LazyLock::new(|| {
    Class::new(
        ClassName::Warlock,
        DieSize::D8,
        5,
        AbilityScoreDistribution {
            scores: HashMap::from([
                (Ability::Strength, 8),
                (Ability::Dexterity, 14),
                (Ability::Constitution, 13),
                (Ability::Intelligence, 12),
                (Ability::Wisdom, 10),
                (Ability::Charisma, 15),
            ]),
            plus_2_bonus: Ability::Charisma,
            plus_1_bonus: Ability::Constitution,
        },
        [Ability::Wisdom, Ability::Charisma],
        3,
        HashMap::from([(FIEND_PATRON.name.clone(), FIEND_PATRON.to_owned())]),
        HashSet::from([4, 8, 12, 16, 19]),
        HashSet::from([
            Skill::Arcana,
            Skill::Deception,
            Skill::History,
            Skill::Intimidation,
            Skill::Investigation,
            Skill::Nature,
            Skill::Religion,
        ]),
        2,
        HashSet::from([ArmorType::Light]),
        HashSet::from([WeaponCategory::Simple]),
        // TODO: Warlock spellcasting is unique, needs to be handled differently
        SpellcastingProgression::Third,
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    )
});

static FIEND_PATRON: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Warlock,
        name: "Fiend Patron".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_prompts: 0,
        armor_proficiencies: HashSet::new(),
        weapon_proficiencies: HashSet::new(),
        spellcasting: SpellcastingProgression::Third,
        effects_by_level: HashMap::from([
            // TODO: Fiendish Resilience at level 6
            (6, vec![]),
        ]),
        resources_by_level: HashMap::new(),
        prompts_by_level: HashMap::new(),
        actions_by_level: HashMap::new(),
    },
});

static WIZARD: LazyLock<Class> = LazyLock::new(|| {
    Class::new(
        ClassName::Wizard,
        DieSize::D6,
        4,
        AbilityScoreDistribution {
            scores: HashMap::from([
                (Ability::Strength, 8),
                (Ability::Dexterity, 12),
                (Ability::Constitution, 13),
                (Ability::Intelligence, 15),
                (Ability::Wisdom, 14),
                (Ability::Charisma, 10),
            ]),
            plus_2_bonus: Ability::Intelligence,
            plus_1_bonus: Ability::Constitution,
        },
        [Ability::Intelligence, Ability::Wisdom],
        3,
        HashMap::from([(EVOKER.name.clone(), EVOKER.to_owned())]),
        HashSet::from([4, 8, 12, 16, 19]),
        HashSet::from([
            Skill::Arcana,
            Skill::History,
            Skill::Insight,
            Skill::Investigation,
            Skill::Medicine,
            Skill::Religion,
        ]),
        2,
        HashSet::from([ArmorType::Light]),
        HashSet::from([WeaponCategory::Simple]),
        SpellcastingProgression::Full,
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    )
});

static EVOKER: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Wizard,
        name: "Evoker".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_prompts: 0,
        armor_proficiencies: HashSet::new(),
        weapon_proficiencies: HashSet::new(),
        spellcasting: SpellcastingProgression::Full,
        effects_by_level: HashMap::from([
            // TODO: Evocation Savant at level 2
            (2, vec![]),
            // TODO: Sculpt Spells at level 3
            (3, vec![]),
            // TODO: Potent Cantrip at level 6
            (6, vec![]),
        ]),
        resources_by_level: HashMap::new(),
        prompts_by_level: HashMap::new(),
        actions_by_level: HashMap::new(),
    },
});
