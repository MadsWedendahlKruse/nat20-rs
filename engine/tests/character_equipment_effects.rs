extern crate nat20_rs;

mod tests {
    use std::{collections::HashSet, sync::Arc};

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::Ability,
            d20_check::RollMode,
            damage::DamageType,
            dice::DieSize,
            items::{
                equipment::{
                    armor::Armor,
                    equipment::{EquipmentItem, EquipmentType, GeneralEquipmentSlot, HandSlot},
                    weapon::{Weapon, WeaponCategory, WeaponType},
                },
                item::ItemRarity,
            },
            saving_throw::SavingThrowSet,
            skill::{Skill, SkillSet},
        },
        entities::character::Character,
        registry, systems,
    };

    #[test]
    fn character_pre_attack_roll_effect() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        let mut ring = EquipmentItem::new(
            "Ring of Advantage".to_string(),
            "A ring that grants advantage on attack rolls.".to_string(),
            0.1,
            100,
            ItemRarity::Uncommon,
            EquipmentType::Ring,
        );

        ring.add_effect(registry::effects::RING_OF_ATTACKING_ID.clone());

        let weapon = Weapon::new(
            EquipmentItem::new(
                "Sword".to_string(),
                "A sharp sword.".to_string(),
                3.0,
                10,
                ItemRarity::Common,
                EquipmentType::MeleeWeapon,
            ),
            WeaponCategory::Martial,
            HashSet::new(),
            vec![(1, DieSize::D8, DamageType::Slashing)],
            vec![],
        );

        let _ = systems::loadout::equip_weapon(&mut world, entity, weapon, HandSlot::Main);

        // Before equipping the ring
        let roll =
            systems::combat::attack_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main)
                .roll(&world, entity);
        assert_eq!(
            roll.roll_result.advantage_tracker.roll_mode(),
            RollMode::Normal
        );

        // Equip the ring
        let _ =
            systems::loadout::equip_item(&mut world, entity, &GeneralEquipmentSlot::Ring(0), ring);
        let roll =
            systems::combat::attack_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main)
                .roll(&world, entity);
        assert_eq!(
            roll.roll_result.advantage_tracker.roll_mode(),
            RollMode::Advantage
        );

        // Unequip the ring
        systems::loadout::unequip_item(&mut world, entity, &GeneralEquipmentSlot::Ring(0));
        let roll =
            systems::combat::attack_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main)
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

        let armor_name = Arc::new("Armor of Sneaking".to_string());

        let mut item = EquipmentItem::new(
            armor_name.to_string(),
            "Lightweight and silent.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );

        item.add_effect(registry::effects::ARMOR_OF_SNEAKING_ID.clone());

        let armor = Armor::light(item, 12);
        systems::loadout::equip_armor(&mut world, entity, armor);

        let check = systems::helpers::get_component::<SkillSet>(&world, entity).check(
            Skill::Stealth,
            &world,
            entity,
        );
        assert_eq!(check.total_modifier, 2);

        let unequipped =
            systems::loadout::unequip_armor(&mut world, entity).expect("Failed to unequip armor");
        assert_eq!(unequipped.equipment.item.name, "Armor of Sneaking");

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

        let armor_name = Arc::new("Armor of Resilience".to_string());

        let mut item = EquipmentItem::new(
            armor_name.to_string(),
            "Grants advantage on Constitution saves.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );

        item.add_effect(registry::effects::ARMOR_OF_CONSTITUTION_SAVING_THROWS_ID.clone());
        let armor = Armor::heavy(item, 12);
        systems::loadout::equip_armor(&mut world, entity, armor);

        let throw = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Constitution,
            &world,
            entity,
        );
        assert_eq!(throw.advantage_tracker.roll_mode(), RollMode::Advantage);

        systems::loadout::unequip_armor(&mut world, entity);

        let throw = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Constitution,
            &world,
            entity,
        );
        assert_eq!(throw.advantage_tracker.roll_mode(), RollMode::Normal);
    }
}
