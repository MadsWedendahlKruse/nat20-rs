// TODO: Not sure if this is the best file name or if it's even a good idea to have a separate file for this.

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use crate::{
    creature::classes::class::SubclassName,
    dice::dice::DieSize,
    items::equipment::{armor::ArmorType, weapon::WeaponCategory},
    stats::{ability::Ability, skill::Skill},
};

use crate::creature::classes::class::{
    Class, ClassBase, ClassName, SpellcastingProgression, Subclass,
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

static FIGHTER: LazyLock<Class> = LazyLock::new(|| Class {
    name: ClassName::Fighter,
    hit_die: DieSize::D10,
    hp_per_level: 6,
    saving_throw_proficiencies: [Ability::Strength, Ability::Constitution],
    subclass_level: 3,
    subclasses: HashMap::from([(CHAMPION.name.clone(), CHAMPION.to_owned())]),
    feat_levels: HashSet::from([4, 6, 8, 12, 14, 16]),
    base: ClassBase {
        skill_proficiencies: HashSet::from([
            Skill::Acrobatics,
            Skill::AnimalHandling,
            Skill::Athletics,
            Skill::History,
            Skill::Insight,
            Skill::Intimidation,
            Skill::Perception,
            Skill::Survival,
        ]),
        skill_choices: 2,
        armor_proficiencies: HashSet::from([ArmorType::Light, ArmorType::Medium, ArmorType::Heavy]),
        weapon_proficiencies: HashSet::from([WeaponCategory::Simple, WeaponCategory::Martial]),
        spellcasting: SpellcastingProgression::None,
        effects_by_level: HashMap::from([
            // TODO: Fighting style at level 1? This requires a choice mechanism, so maybe it's not an effect?
            (1, vec![]),
        ]),
        resources_by_level: HashMap::new(),
    },
});

static CHAMPION: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Fighter,
        name: "Champion".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_choices: 0,
        armor_proficiencies: HashSet::new(),
        weapon_proficiencies: HashSet::new(),
        spellcasting: SpellcastingProgression::None,
        effects_by_level: HashMap::from([
            // TODO: Improved Critical at level 3
            (3, vec![]),
        ]),
        resources_by_level: HashMap::new(),
    },
});

static WARLOCK: LazyLock<Class> = LazyLock::new(|| Class {
    name: ClassName::Warlock,
    hit_die: DieSize::D8,
    hp_per_level: 5,
    saving_throw_proficiencies: [Ability::Wisdom, Ability::Charisma],
    subclass_level: 3,
    subclasses: HashMap::from([(FIEND_PATRON.name.clone(), FIEND_PATRON.to_owned())]),
    feat_levels: HashSet::from([4, 8, 12, 16, 19]),
    base: ClassBase {
        skill_proficiencies: HashSet::from([
            Skill::Arcana,
            Skill::Deception,
            Skill::History,
            Skill::Intimidation,
            Skill::Investigation,
            Skill::Nature,
            Skill::Religion,
        ]),
        skill_choices: 2,
        armor_proficiencies: HashSet::from([ArmorType::Light]),
        weapon_proficiencies: HashSet::from([WeaponCategory::Simple]),
        spellcasting: SpellcastingProgression::Third,
        effects_by_level: HashMap::new(),
        resources_by_level: HashMap::new(),
    },
});

static FIEND_PATRON: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Warlock,
        name: "Fiend Patron".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_choices: 0,
        armor_proficiencies: HashSet::new(),
        weapon_proficiencies: HashSet::new(),
        spellcasting: SpellcastingProgression::Third,
        effects_by_level: HashMap::from([
            // TODO: Fiendish Resilience at level 6
            (6, vec![]),
        ]),
        resources_by_level: HashMap::new(),
    },
});

static WIZARD: LazyLock<Class> = LazyLock::new(|| Class {
    name: ClassName::Wizard,
    hit_die: DieSize::D6,
    hp_per_level: 4,
    saving_throw_proficiencies: [Ability::Intelligence, Ability::Wisdom],
    subclass_level: 3,
    subclasses: HashMap::from([(EVOKER.name.clone(), EVOKER.to_owned())]),
    feat_levels: HashSet::from([4, 8, 12, 16, 19]),
    base: ClassBase {
        skill_proficiencies: HashSet::from([
            Skill::Arcana,
            Skill::History,
            Skill::Insight,
            Skill::Investigation,
            Skill::Medicine,
            Skill::Religion,
        ]),
        skill_choices: 2,
        armor_proficiencies: HashSet::from([ArmorType::Light]),
        weapon_proficiencies: HashSet::from([WeaponCategory::Simple]),
        spellcasting: SpellcastingProgression::Full,
        effects_by_level: HashMap::new(),
        resources_by_level: HashMap::new(),
    },
});

static EVOKER: LazyLock<Subclass> = LazyLock::new(|| Subclass {
    name: SubclassName {
        class: ClassName::Wizard,
        name: "Evoker".to_string(),
    },
    base: ClassBase {
        skill_proficiencies: HashSet::new(),
        skill_choices: 0,
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
    },
});
