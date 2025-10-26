use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use uom::si::{
    f32::{Length, Mass},
    length::foot,
    mass::pound,
};

use crate::components::{
    actions::targeting::TargetingRange,
    damage::DamageType,
    dice::{DiceSet, DieSize},
    id::ItemId,
    items::{
        equipment::{
            armor::Armor,
            weapon::{Weapon, WeaponCategory, WeaponKind, WeaponProperties},
        },
        inventory::ItemInstance,
        item::{Item, ItemRarity},
        money::MonetaryValue,
    },
};

pub static ITEM_REGISTRY: LazyLock<HashMap<ItemId, ItemInstance>> = LazyLock::new(|| {
    HashMap::from([
        (CHAINMAIL_ID.clone(), CHAINMAIL.to_owned()),
        (DAGGER_ID.clone(), DAGGER.to_owned()),
        (FLAIL_ID.clone(), FLAIL.to_owned()),
        (GREATSWORD_ID.clone(), GREATSWORD.to_owned()),
        (JAVELIN_ID.clone(), JAVELIN.to_owned()),
        (LONGBOW_ID.clone(), LONGBOW.to_owned()),
        (LONGSWORD_ID.clone(), LONGSWORD.to_owned()),
        (ROBE_ID.clone(), ROBE.to_owned()),
        (QUARTERSTAFF_ID.clone(), QUARTERSTAFF.to_owned()),
        (SCALE_MAIL_ID.clone(), SCALE_MAIL.to_owned()),
        (SCIMITAR_ID.clone(), SCIMITAR.to_owned()),
        (SHORTBOW_ID.clone(), SHORTBOW.to_owned()),
        (SHORTSWORD_ID.clone(), SHORTSWORD.to_owned()),
        (SPEAR_ID.clone(), SPEAR.to_owned()),
        (
            STUDDED_LEATHER_ARMOR_ID.clone(),
            STUDDED_LEATHER_ARMOR.to_owned(),
        ),
        (TRAVELERS_CLOTHES_ID.clone(), TRAVELERS_CLOTHES.to_owned()),
    ])
});

pub static CHAINMAIL_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.chainmail"));

static CHAINMAIL: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Armor(Armor::heavy(
        Item {
            id: CHAINMAIL_ID.clone(),
            name: "Chainmail".to_string(),
            description: "A suit of chainmail armor, providing good protection.".to_string(),
            weight: Mass::new::<pound>(55.0),
            value: MonetaryValue::from("75 GP"),
            rarity: ItemRarity::Uncommon,
        },
        16,
        Vec::new(),
    ))
});

pub static DAGGER_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.dagger"));

static DAGGER: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: DAGGER_ID.clone(),
            name: "Dagger".to_string(),
            description: "A simple dagger.".to_string(),
            weight: Mass::new::<pound>(1.0),
            value: MonetaryValue::from("2 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::from([
            WeaponProperties::Finesse,
            WeaponProperties::Light,
            WeaponProperties::Thrown,
            WeaponProperties::Range(TargetingRange::with_max::<foot>(20.0, 60.0)),
        ]),
        vec![(1, DieSize::D4, DamageType::Piercing)],
        // TODO: 'Nick' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static FLAIL_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.flail"));

static FLAIL: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: FLAIL_ID.clone(),
            name: "Flail".to_string(),
            description: "A flail with a spiked head, effective against armored foes.".to_string(),
            weight: Mass::new::<pound>(2.0),
            value: MonetaryValue::from("10 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::new(),
        vec![(1, DieSize::D8, DamageType::Bludgeoning)],
        // TODO: 'Sap' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static GREATSWORD_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.greatsword"));

static GREATSWORD: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: GREATSWORD_ID.clone(),
            name: "Greatsword".to_string(),
            description: "A large two-handed sword, capable of dealing heavy damage.".to_string(),
            weight: Mass::new::<pound>(6.0),
            value: MonetaryValue::from("50 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::from([WeaponProperties::TwoHanded, WeaponProperties::Heavy]),
        vec![(2, DieSize::D6, DamageType::Slashing)],
        // TODO: 'Graze' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static JAVELIN_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.javelin"));

static JAVELIN: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: JAVELIN_ID.clone(),
            name: "Javelin".to_string(),
            description: "A versatile javelin, effective in both melee and ranged combat."
                .to_string(),
            weight: Mass::new::<pound>(2.0),
            value: MonetaryValue::from("5 SP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Simple,
        HashSet::from([
            WeaponProperties::Thrown,
            WeaponProperties::Range(TargetingRange::with_max::<foot>(30.0, 120.0)),
        ]),
        vec![(1, DieSize::D6, DamageType::Piercing)],
        // TODO: 'Slow' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static LONGBOW_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.longbow"));

static LONGBOW: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: LONGBOW_ID.clone(),
            name: "Longbow".to_string(),
            description: "A powerful longbow, effective for ranged combat.".to_string(),
            weight: Mass::new::<pound>(2.0),
            value: MonetaryValue::from("50 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Ranged,
        WeaponCategory::Martial,
        HashSet::from([
            WeaponProperties::Range(TargetingRange::with_max::<foot>(150.0, 600.0)),
            WeaponProperties::TwoHanded,
            WeaponProperties::Heavy,
        ]),
        vec![(1, DieSize::D8, DamageType::Piercing)],
        Vec::new(), // TODO: 'Slow' mastery action
        Vec::new(),
    ))
});

pub static LONGSWORD_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.longsword"));

static LONGSWORD: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: LONGSWORD_ID.clone(),
            name: "Longsword".to_string(),
            description:
                "A versatile longsword, effective in both one-handed and two-handed combat."
                    .to_string(),
            weight: Mass::new::<pound>(3.0),
            value: MonetaryValue::from("15 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::from([WeaponProperties::Versatile(DiceSet::from("1d10"))]),
        vec![(1, DieSize::D8, DamageType::Slashing)],
        Vec::new(), // TODO: 'Sap' mastery action
        Vec::new(),
    ))
});

pub static ROBE_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.robe"));

static ROBE: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Armor(Armor::clothing(
        Item {
            id: ROBE_ID.clone(),
            name: "Robe".to_string(),
            description: "A simple robe, providing minimal protection.".to_string(),
            weight: Mass::new::<pound>(4.0),
            value: MonetaryValue::from("1 GP"),
            rarity: ItemRarity::Common,
        },
        Vec::new(),
    ))
});

pub static QUARTERSTAFF_ID: LazyLock<ItemId> =
    LazyLock::new(|| ItemId::from_str("item.quarterstaff"));

static QUARTERSTAFF: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: QUARTERSTAFF_ID.clone(),
            name: "Quarterstaff".to_string(),
            description: "A sturdy quarterstaff, useful for both combat and support.".to_string(),
            weight: Mass::new::<pound>(4.0),
            value: MonetaryValue::from("2 SP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Simple,
        HashSet::from([WeaponProperties::Versatile(DiceSet::from("1d8"))]),
        vec![(1, DieSize::D6, DamageType::Bludgeoning)],
        // TODO: 'Topple' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static SCALE_MAIL_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.scale_mail"));

static SCALE_MAIL: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Armor(Armor::medium(
        Item {
            id: SCALE_MAIL_ID.clone(),
            name: "Scale Mail".to_string(),
            description:
                "A suit of scale mail armor, providing good protection with moderate weight."
                    .to_string(),
            weight: Mass::new::<pound>(45.0),
            value: MonetaryValue::from("50 GP"),
            rarity: ItemRarity::Uncommon,
        },
        14,
        true,
        Vec::new(),
    ))
});

pub static SCIMITAR_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.scimitar"));

static SCIMITAR: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: SCIMITAR_ID.clone(),
            name: "Scimitar".to_string(),
            description: "A curved sword, favored for its speed and agility.".to_string(),
            weight: Mass::new::<pound>(3.0),
            value: MonetaryValue::from("25 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::from([WeaponProperties::Finesse, WeaponProperties::Light]),
        vec![(1, DieSize::D6, DamageType::Slashing)],
        Vec::new(), // TODO: 'Nick' mastery action
        Vec::new(),
    ))
});

pub static SHORTBOW_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.shortbow"));

static SHORTBOW: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: SHORTBOW_ID.clone(),
            name: "Shortbow".to_string(),
            description: "A lightweight bMass::new::<pound>(ow,) ideal for quick shots."
                .to_string(),
            weight: Mass::new::<pound>(2.0),
            value: MonetaryValue::from("25 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Ranged,
        WeaponCategory::Simple,
        HashSet::from([
            WeaponProperties::Range(TargetingRange::with_max::<foot>(80.0, 320.0)),
            WeaponProperties::TwoHanded,
        ]),
        vec![(1, DieSize::D6, DamageType::Piercing)],
        // TODO: 'Vex' mastery action
        Vec::new(),
        Vec::new(),
    ))
});

pub static SHORTSWORD_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.shortsword"));

static SHORTSWORD: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: SHORTSWORD_ID.clone(),
            name: "Shortsword".to_string(),
            description: "A versatile shortsword, effective in close combat.".to_string(),
            weight: Mass::new::<pound>(2.0),
            value: MonetaryValue::from("10 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Martial,
        HashSet::from([WeaponProperties::Finesse, WeaponProperties::Light]),
        vec![(1, DieSize::D6, DamageType::Slashing)],
        Vec::new(), // TODO: 'Vex' mastery action
        Vec::new(),
    ))
});

pub static SPEAR_ID: LazyLock<ItemId> = LazyLock::new(|| ItemId::from_str("item.spear"));

static SPEAR: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Weapon(Weapon::new(
        Item {
            id: SPEAR_ID.clone(),
            name: "Spear".to_string(),
            description: "A versatile spear, effective in both melee and ranged combat."
                .to_string(),
            weight: Mass::new::<pound>(3.0),
            value: MonetaryValue::from("1 GP"),
            rarity: ItemRarity::Common,
        },
        WeaponKind::Melee,
        WeaponCategory::Simple,
        HashSet::from([
            WeaponProperties::Versatile(DiceSet::from("1d8")),
            WeaponProperties::Thrown,
            WeaponProperties::Range(TargetingRange::with_max::<foot>(20.0, 60.0)),
        ]),
        vec![(1, DieSize::D6, DamageType::Piercing)],
        Vec::new(),
        Vec::new(),
    ))
});

pub static STUDDED_LEATHER_ARMOR_ID: LazyLock<ItemId> =
    LazyLock::new(|| ItemId::from_str("item.studded_leather_armor"));

static STUDDED_LEATHER_ARMOR: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Armor(Armor::light(
        Item {
            id: STUDDED_LEATHER_ARMOR_ID.clone(),
            name: "Studded Leather Armor".to_string(),
            description:
                "A suit of studded leather armor, providing good protection with minimal weight."
                    .to_string(),
            weight: Mass::new::<pound>(13.0),
            value: MonetaryValue::from("45 GP"),
            rarity: ItemRarity::Uncommon,
        },
        12,
        Vec::new(),
    ))
});

pub static TRAVELERS_CLOTHES_ID: LazyLock<ItemId> =
    LazyLock::new(|| ItemId::from_str("item.travelers_clothes"));

static TRAVELERS_CLOTHES: LazyLock<ItemInstance> = LazyLock::new(|| {
    ItemInstance::Armor(Armor::clothing(
        Item {
            id: TRAVELERS_CLOTHES_ID.clone(),
            name: "Traveler's Clothes".to_string(),
            description: "Resilient garments designed for travel in various environments."
                .to_string(),
            weight: Mass::new::<pound>(4.0),
            value: MonetaryValue::from("2 GP"),
            rarity: ItemRarity::Common,
        },
        Vec::new(),
    ))
});
