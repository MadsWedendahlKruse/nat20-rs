extern crate nat20_rs;

mod tests {
    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreMap},
            id::ItemId,
            items::equipment::slots::EquipmentSlot,
            modifier::ModifierSource,
        },
        entities::character::Character,
        registry::{self, registry::ItemsRegistry},
        systems,
    };

    #[test]
    fn character_armor_class_no_dex() {
        let mut world = World::new();
        let character = world.spawn(Character::default());

        let _ = systems::loadout::equip(
            &mut world,
            character,
            ItemsRegistry::get(&ItemId::from_str("item.chainmail"))
                .unwrap()
                .clone(),
        );

        let armor_class = systems::loadout::armor_class(&world, character);
        assert_eq!(16, armor_class.total());
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
                systems::helpers::get_component_mut::<AbilityScoreMap>(&mut world, character);
            ability_scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 15),
            );
            ability_scores.add_modifier(
                Ability::Dexterity,
                ModifierSource::Item(ItemId::from_str("item.ring_of_dexterity")),
                2,
            );
        }

        let _ = systems::loadout::equip(
            &mut world,
            character,
            ItemsRegistry::get(&ItemId::from_str("item.studded_leather_armor"))
                .unwrap()
                .clone(),
        );

        {
            let armor_class = systems::loadout::armor_class(&world, character);
            // Armour Class
            // Dex: 15 + 2 (item) = 17
            // 12 (armor) + 3 (Dex mod) = 15
            println!("{:?}", armor_class);
            assert_eq!(15, armor_class.total());
        }

        // Un-equip the armor
        let armor =
            systems::loadout::unequip(&mut world, character, &EquipmentSlot::Armor).unwrap();
        let armor_class = systems::loadout::armor_class(&world, character);
        println!("Un-equipped {:?}", armor);
        // Check if the armor class is updated
        println!("{:?}", armor_class);
        assert_eq!(10, armor_class.total());
    }
}
