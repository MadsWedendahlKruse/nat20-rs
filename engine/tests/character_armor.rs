extern crate nat20_rs;

mod tests {
    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreSet},
            items::{
                equipment::{
                    armor::Armor,
                    equipment::{EquipmentItem, EquipmentType},
                },
                item::ItemRarity,
            },
            modifier::ModifierSource,
        },
        entities::character::Character,
        registry, systems,
    };

    #[test]
    fn character_armor_class_no_dex() {
        let mut world = World::new();
        let character = world.spawn(Character::default());

        let equipment: EquipmentItem = EquipmentItem::new(
            "Adamantium Armour".to_string(),
            "A suit of armor made from adamantium.".to_string(),
            19.0,
            5000,
            ItemRarity::VeryRare,
            EquipmentType::Armor,
        );
        let armor = Armor::heavy(equipment, 19);

        systems::loadout::equip_armor(&mut world, character, armor);

        let armor_class = systems::loadout::armor_class(&world, character);
        assert_eq!(19, armor_class.total());
        println!("{:?}", armor_class);

        // Check that the heavy armor gives stealth disadvantage
        let effects = systems::effects::effects(&world, character);
        assert!(!effects.is_empty());
        assert!(
            effects
                .iter()
                .any(|e| { *e.id() == *registry::effects::ARMOR_STEALTH_DISADVANTAGE_ID })
        );
    }

    #[test]
    fn character_armor_class_dex_and_bonus() {
        // Create a character with a Dexterity modifier of +3
        let mut world = World::new();
        let character = world.spawn(Character::default());

        {
            let mut ability_scores =
                systems::helpers::get_component_mut::<AbilityScoreSet>(&mut world, character);
            ability_scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 15),
            );
            ability_scores.add_modifier(
                Ability::Dexterity,
                ModifierSource::Item("Ring of Dexterity".to_string()),
                2,
            );
        }

        let equipment: EquipmentItem = EquipmentItem::new(
            "Light Armor".to_string(),
            "A suit of light armor.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );

        let armor = Armor::light(equipment, 12);

        systems::loadout::equip_armor(&mut world, character, armor);

        {
            let armor_class = systems::loadout::armor_class(&world, character);
            // Armour Class
            // Dex: 15 + 2 (item) = 17
            // 12 (armor) + 3 (Dex mod) = 15
            println!("{:?}", armor_class);
            assert_eq!(15, armor_class.total());
        }

        // Un-equip the armor
        let armor_name = systems::loadout::unequip_armor(&mut world, character)
            .unwrap()
            .equipment
            .item
            .name;
        let armor_class = systems::loadout::armor_class(&world, character);
        println!("Un-equipped {:?}", armor_name);
        assert_eq!(armor_name, "Light Armor");
        // Check if the armor class is updated
        println!("{:?}", armor_class);
        assert_eq!(10, armor_class.total());
    }
}
