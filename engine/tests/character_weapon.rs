extern crate nat20_rs;

mod tests {
    use std::collections::HashSet;

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreMap},
            damage::DamageType,
            dice::{DiceSet, DieSize},
            id::ItemId,
            items::{
                equipment::{
                    equipment::{EquipmentItem, EquipmentKind},
                    loadout::Loadout,
                    slots::EquipmentSlot,
                    weapon::{self, Weapon, WeaponCategory, WeaponKind, WeaponProperties},
                },
                inventory::ItemInstance,
                item::{Item, ItemRarity},
                money::MonetaryValue,
            },
            modifier::ModifierSource,
            proficiency::ProficiencyLevel,
        },
        entities::character::Character,
        registry,
        systems::{self, helpers},
        test_utils::fixtures,
    };

    #[test]
    fn character_weapon_finesse_modifier() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        // Set Strength 14, Dexterity 16
        {
            let mut scores = helpers::get_component_mut::<AbilityScoreMap>(&mut world, entity);
            scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
            scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 16),
            );
        }

        let weapon = registry::items::ITEM_REGISTRY
            .get(&registry::items::SCIMITAR_ID)
            .unwrap()
            .clone();
        let weapon = match weapon {
            ItemInstance::Weapon(weapon) => weapon,
            _ => panic!("Expected a weapon item"),
        };

        let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(&world, entity);
        assert_eq!(
            weapon.determine_ability(&ability_scores),
            Ability::Dexterity
        );

        let damage_roll = weapon.damage_roll(
            &ability_scores,
            false, // not wielding with both hands
        );
        let result = damage_roll.roll();

        println!("{:?}", result);
        assert!(
            (4..=11).contains(&result.total),
            "Damage roll out of bounds: {}",
            result.total
        );
        assert!(
            damage_roll
                .primary
                .dice_roll
                .modifiers
                .get(&ModifierSource::Ability(Ability::Dexterity))
                .is_some()
        );
    }

    #[test]
    fn character_versatile_weapon() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        // Equip longsword
        let longsword = registry::items::ITEM_REGISTRY
            .get(&registry::items::LONGSWORD_ID)
            .unwrap()
            .clone();
        let _ = systems::loadout::equip(&mut world, entity, longsword);

        // Longsword used with two hands
        let roll = systems::combat::damage_roll(&world, entity, &EquipmentSlot::MeleeMainHand);
        assert_eq!(roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D10);

        systems::loadout::equip_in_slot(
            &mut world,
            entity,
            &EquipmentSlot::MeleeOffHand,
            registry::items::ITEM_REGISTRY
                .get(&registry::items::DAGGER_ID)
                .unwrap()
                .clone(),
        )
        .unwrap();

        // Longsword now used one-handed
        let roll = systems::combat::damage_roll(&world, entity, &EquipmentSlot::MeleeMainHand);
        assert_eq!(roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D8);

        // Unequip dagger
        let unequipped =
            systems::loadout::unequip(&mut world, entity, &EquipmentSlot::MeleeOffHand).unwrap();

        // Longsword used with two hands again
        let roll = systems::combat::damage_roll(&world, entity, &EquipmentSlot::MeleeMainHand);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D10);
    }

    #[test]
    fn character_two_handed_weapon() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        systems::loadout::equip_in_slot(
            &mut world,
            entity,
            &EquipmentSlot::MeleeOffHand,
            registry::items::ITEM_REGISTRY
                .get(&registry::items::DAGGER_ID)
                .unwrap()
                .clone(),
        )
        .unwrap();
        let _ = systems::loadout::equip_in_slot(
            &mut world,
            entity,
            &EquipmentSlot::MeleeMainHand,
            registry::items::ITEM_REGISTRY
                .get(&registry::items::LONGSWORD_ID)
                .unwrap()
                .clone(),
        );

        let unequipped = systems::loadout::equip(
            &mut world,
            entity,
            registry::items::ITEM_REGISTRY
                .get(&registry::items::GREATSWORD_ID)
                .unwrap()
                .clone(),
        )
        .unwrap();
        assert_eq!(unequipped.len(), 2);

        // Main hand has greatsword, off-hand should be empty
        let loadout = systems::helpers::get_component::<Loadout>(&world, entity);
        assert!(loadout.has_weapon_in_hand(&EquipmentSlot::MeleeMainHand));
        assert!(!loadout.has_weapon_in_hand(&EquipmentSlot::MeleeOffHand));
    }

    #[test]
    fn character_attack_roll_basic() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        {
            let mut scores = helpers::get_component_mut::<AbilityScoreMap>(&mut world, entity);
            scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
            scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 16),
            );
        }

        let longsword = Weapon::new(
            Item {
                id: ItemId::from_str("item.longsword"),
                name: "Longsword".to_string(),
                description: "A longsword.".to_string(),
                weight: 3.0,
                value: MonetaryValue::from("15 GP"),
                rarity: ItemRarity::Common,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Slashing)],
            vec![],
            vec![],
        );

        systems::loadout::equip(&mut world, entity, longsword).unwrap();

        let roll = systems::combat::attack_roll(&world, entity, &EquipmentSlot::MeleeMainHand);

        println!("{:?}", roll);
        assert!(
            roll.d20_check
                .modifiers()
                .contains_key(&ModifierSource::Ability(Ability::Dexterity))
        );
        assert!(
            !roll
                .d20_check
                .modifiers()
                .contains_key(&ModifierSource::Proficiency(ProficiencyLevel::Proficient))
        );
        assert!(
            !roll
                .d20_check
                .modifiers()
                .contains_key(&ModifierSource::Custom("Enchantment".to_string()))
        );
    }
}
