extern crate nat20_rs;

mod tests {

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreSet},
            d20_check::RollMode,
            items::{
                equipment::{
                    armor::Armor,
                    equipment::{EquipmentItem, EquipmentType},
                },
                item::ItemRarity,
            },
            modifier::ModifierSource,
            proficiency::Proficiency,
            saving_throw::SavingThrowSet,
            skill::{Skill, SkillSet},
        },
        entities::character::Character,
        systems::{self},
        test_utils::fixtures,
    };

    #[test]
    fn character_saving_throw_modifier() {
        let mut world = World::new();

        let entity = world.spawn(Character::default());

        {
            let mut ability_scores =
                systems::helpers::get_component_mut::<AbilityScoreSet>(&mut world, entity);
            ability_scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
            ability_scores.add_modifier(
                Ability::Strength,
                ModifierSource::Item("Ring of Strength".to_string()),
                2,
            );
            assert_eq!(ability_scores.get(Ability::Strength).total(), 19);
        }

        let result = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Strength,
            &world,
            entity,
        );
        assert_eq!(result.modifier_breakdown.total(), 4);
    }

    #[test]
    fn character_saving_throw_proficiency() {
        let mut world = World::new();

        // Default character is level 0, meaning it has no proficieny bonus, so
        // if we want to test that we need a character with at least one level.
        // Easiest way is to use one of the fixtures.
        let entity = fixtures::creatures::heroes::wizard(&mut world).id();

        systems::helpers::get_component_mut::<AbilityScoreSet>(&mut world, entity)
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        systems::helpers::get_component_mut::<SavingThrowSet>(&mut world, entity)
            .set_proficiency(Ability::Strength, Proficiency::Proficient);

        let result = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Strength,
            &world,
            entity,
        );
        assert_eq!(result.modifier_breakdown.total(), 6);
    }

    #[test]
    fn character_saving_throw_proficiency_expertise() {
        let mut world = World::new();

        // Default character is level 0, meaning it has no proficieny bonus, so
        // if we want to test that we need a character with at least one level.
        // Easiest way is to use one of the fixtures.
        let entity = fixtures::creatures::heroes::wizard(&mut world).id();

        systems::helpers::get_component_mut::<AbilityScoreSet>(&mut world, entity)
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        systems::helpers::get_component_mut::<SavingThrowSet>(&mut world, entity)
            .set_proficiency(Ability::Strength, Proficiency::Expertise);

        let result = systems::helpers::get_component::<SavingThrowSet>(&world, entity).check(
            Ability::Strength,
            &world,
            entity,
        );
        assert_eq!(result.modifier_breakdown.total(), 9);
    }

    #[test]
    fn character_skill_disadvantage() {
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

        let result = systems::helpers::get_component::<SkillSet>(&world, character).check(
            Skill::Stealth,
            &world,
            character,
        );
        assert!(result.advantage_tracker.roll_mode() == RollMode::Disadvantage);
    }
}
