use std::{collections::HashMap, sync::LazyLock};

use crate::{
    components::{
        ability::Ability,
        background::Background,
        id::BackgroundId,
        level_up::{ChoiceItem, ChoiceSpec},
        skill::Skill,
    },
    registry,
};

pub static BACKGROUND_REGISTRY: LazyLock<HashMap<BackgroundId, Background>> = LazyLock::new(|| {
    HashMap::from([
        (ACOLYTE_ID.clone(), ACOLYTE.to_owned()),
        (CRIMINAL_ID.clone(), CRIMINAL.to_owned()),
        (SAGE_ID.clone(), SAGE.to_owned()),
        (SOLDIER_ID.clone(), SOLDIER.to_owned()),
    ])
});

pub static ACOLYTE_ID: LazyLock<BackgroundId> =
    LazyLock::new(|| BackgroundId::from_str("background.acolyte"));

static ACOLYTE: LazyLock<Background> = LazyLock::new(|| {
    Background::new(
        ACOLYTE_ID.clone(),
        [Ability::Wisdom, Ability::Charisma, Ability::Intelligence],
        // TODO: Placeholder
        registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
        [Skill::Insight, Skill::Religion],
        ChoiceSpec::single(
            "Acolyte Starting Equipment",
            vec![
                ChoiceItem::Equipment {
                    items: vec![(1, registry::items::ROBE_ID.clone())],
                    money: "8 GP".to_string(),
                },
                ChoiceItem::Equipment {
                    items: Vec::new(),
                    money: "50 GP".to_string(),
                },
            ],
        )
        .with_id("choice.starting_equipment.acolyte")
        .clone(),
    )
});

pub static CRIMINAL_ID: LazyLock<BackgroundId> =
    LazyLock::new(|| BackgroundId::from_str("background.criminal"));

static CRIMINAL: LazyLock<Background> = LazyLock::new(|| {
    Background::new(
        CRIMINAL_ID.clone(),
        [Ability::Dexterity, Ability::Charisma, Ability::Intelligence],
        // TODO: Placeholder
        registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
        [Skill::SleightOfHand, Skill::Stealth],
        ChoiceSpec::single(
            "Criminal Starting Equipment",
            vec![
                ChoiceItem::Equipment {
                    items: vec![
                        (2, registry::items::DAGGER_ID.clone()),
                        (1, registry::items::TRAVELERS_CLOTHES_ID.clone()),
                    ],
                    money: "16 GP".to_string(),
                },
                ChoiceItem::Equipment {
                    items: Vec::new(),
                    money: "50 GP".to_string(),
                },
            ],
        )
        .with_id("choice.starting_equipment.criminal")
        .clone(),
    )
});

pub static SAGE_ID: LazyLock<BackgroundId> =
    LazyLock::new(|| BackgroundId::from_str("background.sage"));

static SAGE: LazyLock<Background> = LazyLock::new(|| {
    Background::new(
        SAGE_ID.clone(),
        [Ability::Intelligence, Ability::Wisdom, Ability::Charisma],
        // TODO: Placeholder
        registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
        [Skill::Arcana, Skill::History],
        ChoiceSpec::single(
            "Sage Starting Equipment",
            vec![
                ChoiceItem::Equipment {
                    items: vec![
                        (1, registry::items::QUARTERSTAFF_ID.clone()),
                        (1, registry::items::ROBE_ID.clone()),
                    ],
                    money: "8 GP".to_string(),
                },
                ChoiceItem::Equipment {
                    items: Vec::new(),
                    money: "50 GP".to_string(),
                },
            ],
        )
        .with_id("choice.starting_equipment.sage")
        .clone(),
    )
});

pub static SOLDIER_ID: LazyLock<BackgroundId> =
    LazyLock::new(|| BackgroundId::from_str("background.soldier"));

static SOLDIER: LazyLock<Background> = LazyLock::new(|| {
    Background::new(
        SOLDIER_ID.clone(),
        [Ability::Strength, Ability::Constitution, Ability::Charisma],
        // TODO: Placeholder
        // Savage Attacker could be a reaction that's triggered on a melee attack
        // (in the SRD it says "when you hit a target with a weapon"), which costs
        // a charge of "Savage Attacker", which is recharged every turn, and then
        // it would re-roll the damage dice of the attack and use the highest roll
        // Gameplay wise I feel like it would be a bit annoying to have a reaction
        // pop-up every turn, so maybe it should just be a passive? In that case,
        // it's a bit too powerful to be a background feat.
        registry::feats::FIGHTING_STYLE_ARCHERY_ID.clone(),
        [Skill::Athletics, Skill::Intimidation],
        ChoiceSpec::single(
            "Soldier Starting Equipment",
            vec![
                ChoiceItem::Equipment {
                    items: vec![
                        (1, registry::items::SPEAR_ID.clone()),
                        (1, registry::items::SHORTBOW_ID.clone()),
                        (1, registry::items::TRAVELERS_CLOTHES_ID.clone()),
                    ],
                    money: "14 GP".to_string(),
                },
                ChoiceItem::Equipment {
                    items: Vec::new(),
                    money: "50 GP".to_string(),
                },
            ],
        )
        .with_id("choice.starting_equipment.soldier")
        .clone(),
    )
});
