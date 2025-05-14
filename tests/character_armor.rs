extern crate nat20_rs;

mod tests {
    use nat20_rs::creature::character::*;
    use nat20_rs::item::equipment::armor::Armor;
    use nat20_rs::item::equipment::equipment::EquipmentItem;
    use nat20_rs::item::equipment::equipment::EquipmentType;
    use nat20_rs::item::item::ItemRarity;
    use nat20_rs::stats::ability::*;
    use nat20_rs::stats::d20_check::*;
    use nat20_rs::stats::modifier::*;
    use nat20_rs::stats::proficiency::Proficiency;
    use nat20_rs::stats::skill::*;

    #[test]
    fn character_armor_class_no_dex() {
        let mut character = Character::default();

        let equipment: EquipmentItem = EquipmentItem::new(
            "Adamantium Armour".to_string(),
            "A suit of armor made from adamantium.".to_string(),
            19.0,
            5000,
            ItemRarity::VeryRare,
            EquipmentType::Armor,
        );
        let armor = Armor::heavy(equipment, 19);

        character.equip_armor(armor);

        let armor_class = character.armor_class();
        assert_eq!(19, armor_class.total());
        println!("{:?}", armor_class);
    }

    #[test]
    fn character_armor_class_dex_and_bonus() {
        let mut character = Character::default();
        character.ability_scores.insert(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 15),
        );
        character
            .skills
            .get_mut(&Skill::Stealth)
            .unwrap()
            .proficiency = Proficiency::Proficient;
        character
            .ability_scores
            .get_mut(&Ability::Dexterity)
            .unwrap()
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Dexterity".to_string()), 2);

        let mut equipment: EquipmentItem = EquipmentItem::new(
            "Armor of Sneaking".to_string(),
            "It's made of a lightweight material that allows for silent movement.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );
        equipment.add_on_equip(|character| {
            character.add_skill_modifier(
                Skill::Stealth,
                ModifierSource::Item("Armor of Sneaking".to_string()),
                1,
            );
            character
                .saving_throws
                .get_mut(&Ability::Constitution)
                .unwrap()
                .advantage_tracker_mut()
                .add(
                    AdvantageType::Advantage,
                    ModifierSource::Item("Armor of Sneaking".to_string()),
                );
        });
        equipment.add_on_unequip(|character| {
            character.remove_skill_modifier(
                Skill::Stealth,
                &ModifierSource::Item("Armor of Sneaking".to_string()),
            );
            character
                .saving_throws
                .get_mut(&Ability::Constitution)
                .unwrap()
                .advantage_tracker_mut()
                .remove(&ModifierSource::Item("Armor of Sneaking".to_string()));
        });
        let armor = Armor::light(equipment, 12);

        character.equip_armor(armor);

        let armor_class = character.armor_class();
        // Armour Class
        // Dex: 15 + 2 (item) = 17
        // 12 (armor) + 3 (Dex mod) = 15
        println!("{:?}", armor_class);
        assert_eq!(15, armor_class.total());
        // Stealth
        // 3 (Dex mod) + 2 (proficiency) + 1 (item) = 4
        println!("{:?}", character.skill_modifier(Skill::Stealth));
        assert_eq!(6, character.skill_modifier(Skill::Stealth).total());
        // Constitution Saving Throw
        assert!(
            character
                .saving_throws
                .get(&Ability::Constitution)
                .unwrap()
                .advantage_tracker()
                .roll_mode()
                == RollMode::Advantage
        );

        // Un-equip the armor
        let armor_name = character.unequip_armor().unwrap().equipment.item.name;
        let armor_class = character.armor_class();
        println!("Un-equipped {:?}", armor_name);
        assert_eq!(armor_name, "Armor of Sneaking");
        // Check if the armor class is updated
        println!("{:?}", armor_class);
        assert_eq!(10, armor_class.total());
        // Check if the skill modifier is removed
        println!("{:?}", character.skill_modifier(Skill::Stealth));
        assert_eq!(5, character.skill_modifier(Skill::Stealth).total());
        // Check if the advantage is removed
        assert!(
            character
                .saving_throws
                .get(&Ability::Constitution)
                .unwrap()
                .advantage_tracker()
                .roll_mode()
                == RollMode::Normal
        );
    }
}
