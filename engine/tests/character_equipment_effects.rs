extern crate nat20_rs;

mod tests {

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::Ability,
            d20_check::RollMode,
            id::ItemId,
            items::{
                equipment::{
                    armor::Armor,
                    equipment::{EquipmentItem, EquipmentKind},
                    slots::EquipmentSlot,
                },
                item::{Item, ItemRarity},
            },
            saving_throw::SavingThrowSet,
            skill::{Skill, SkillSet},
        },
        entities::character::Character,
        registry, systems,
        test_utils::fixtures,
    };

    #[test]
    fn character_pre_attack_roll_effect() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        let ring = EquipmentItem {
            item: Item {
                id: ItemId::from_str("item.ring_of_attacking"),
                name: "Ring of Attacking".to_string(),
                description: "A magical ring that grants advantage on attack rolls.".to_string(),
                weight: 0.1,
                value: 1000,
                rarity: ItemRarity::Rare,
            },
            kind: EquipmentKind::Ring,
            effects: vec![registry::effects::RING_OF_ATTACKING_ID.clone()],
        };

        let _ = systems::loadout::equip(&mut world, entity, fixtures::weapons::dagger_light());

        // Before equipping the ring
        let roll = systems::combat::attack_roll(&world, entity, &EquipmentSlot::MeleeMainHand)
            .roll(&world, entity);
        assert_eq!(
            roll.roll_result.advantage_tracker.roll_mode(),
            RollMode::Normal
        );

        // Equip the ring
        let _ = systems::loadout::equip_in_slot(&mut world, entity, &EquipmentSlot::Ring1, ring);
        let roll = systems::combat::attack_roll(&world, entity, &EquipmentSlot::MeleeMainHand)
            .roll(&world, entity);
        assert_eq!(
            roll.roll_result.advantage_tracker.roll_mode(),
            RollMode::Advantage
        );

        // Unequip the ring
        systems::loadout::unequip(&mut world, entity, &EquipmentSlot::Ring1);
        let roll = systems::combat::attack_roll(&world, entity, &EquipmentSlot::MeleeMainHand)
            .roll(&world, entity);
        assert_eq!(
            roll.roll_result.advantage_tracker.roll_mode(),
            RollMode::Normal
        );
    }

    #[test]
    fn character_skill_bonus_effect() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        let armor = Armor::light(
            Item {
                id: ItemId::from_str("item.armor_of_sneaking"),
                name: "Armor of Sneaking".to_string(),
                description: "A magical armor that grants a bonus to Stealth.".to_string(),
                weight: 0.5,
                value: 500,
                rarity: ItemRarity::Rare,
            },
            12,
            vec![registry::effects::ARMOR_OF_SNEAKING_ID.clone()],
        );
        systems::loadout::equip(&mut world, entity, armor);

        let check = systems::helpers::get_component::<SkillSet>(&world, entity).check(
            Skill::Stealth,
            &world,
            entity,
        );
        assert_eq!(check.total_modifier, 2);

        let _ = systems::loadout::unequip(&mut world, entity, &EquipmentSlot::Armor)
            .expect("Failed to unequip armor");

        let check = systems::helpers::get_component::<SkillSet>(&world, entity).check(
            Skill::Stealth,
            &world,
            entity,
        );
        assert_eq!(check.total_modifier, 0);
    }

    #[test]
    fn character_saving_throw_effect() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        let armor = Armor::heavy(
            Item {
                id: ItemId::from_str("item.armor_of_constitution_saving_throws"),
                name: "Armor of Constitution Saving Throws".to_string(),
                description: "A magical armor that grants advantage on Constitution saving throws."
                    .to_string(),
                weight: 10.0,
                value: 1000,
                rarity: ItemRarity::VeryRare,
            },
            18,
            vec![registry::effects::ARMOR_OF_CONSTITUTION_SAVING_THROWS_ID.clone()],
        );
        systems::loadout::equip(&mut world, entity, armor);

        let throw = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Constitution,
            &world,
            entity,
        );
        assert_eq!(throw.advantage_tracker.roll_mode(), RollMode::Advantage);

        systems::loadout::unequip(&mut world, entity, &EquipmentSlot::Armor)
            .expect("Failed to unequip armor");

        let throw = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Constitution,
            &world,
            entity,
        );
        assert_eq!(throw.advantage_tracker.roll_mode(), RollMode::Normal);
    }
}
