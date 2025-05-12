extern crate nat20_rs;

#[cfg(test)]
mod tests {
    use nat20_rs::combat::damage::*;
    use nat20_rs::creature::character::*;
    use nat20_rs::dice::dice::*;
    use nat20_rs::item::equipment::armor::Armor;
    use nat20_rs::item::equipment::equipment::EquipmentItem;
    use nat20_rs::item::equipment::equipment::EquipmentType;
    use nat20_rs::item::item::ItemRarity;
    use nat20_rs::stats::ability::*;
    use nat20_rs::stats::d20_check::*;
    use nat20_rs::stats::modifier::*;
    use nat20_rs::stats::proficiency::Proficiency;
    use nat20_rs::stats::skill::*;
    use std::collections::HashMap;

    #[test]
    fn test_character_skill_modifier() {
        let mut abilities = HashMap::new();
        let mut strength = AbilityScore::new(Ability::Strength, 17);
        strength
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Strength".to_string()), 2);
        abilities.insert(Ability::Strength, strength);

        let mut skills = HashMap::new();
        let mut athletics = D20Check::new(Proficiency::Proficient);
        athletics
            .modifiers
            .add_modifier(ModifierSource::Item("Athlete's Belt".to_string()), 1);
        skills.insert(Skill::Athletics, athletics);

        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 5);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            skills,
            HashMap::new(),
            DamageResistances::new(),
        );

        // 17 (base) + 2 (item) = 19
        assert_eq!(character.ability_total(Ability::Strength), 19);
        // Calculate the expected skill modifier
        // 4 (ability) + 1 (item) + 3 (proficiency) = 8
        assert_eq!(character.skill_modifier(Skill::Athletics).total(), 8);
        print!(
            "Athletics Modifier: {} = {:?}",
            character.skill_modifier(Skill::Athletics).total(),
            character.skill_modifier(Skill::Athletics)
        );
    }

    #[test]
    fn test_character_take_damage() {
        let mut resistances = DamageResistances::new();
        resistances.add_effect(
            DamageType::Fire,
            DamageMitigationEffect {
                source: ModifierSource::Item("Boots of Fire Resistance".to_string()),
                op: MitigationOp::Resistance,
            },
        );
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            resistances,
        );

        assert_eq!(character.current_hp, 20);

        let damage_roll_result = DamageRollResult {
            label: "Fireball".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Fire,
                result: DiceGroupRollResult {
                    label: "Fireball".to_string(),
                    rolls: vec![4, 4, 4, 4, 4, 4, 4, 4],
                    die_size: DieSize::D8,
                    modifiers: ModifierSet::new(),
                    subtotal: 32,
                },
            }],
            total: 32,
        };

        character.take_damage(&damage_roll_result);
        // Fire resistance halves the damage
        // 20 - (32 / 2) = 4
        assert_eq!(character.current_hp, 4);
    }

    #[test]
    fn test_character_heal() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        let damage_roll_result = DamageRollResult {
            label: "Arrow".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceGroupRollResult {
                    label: "Arrow".to_string(),
                    rolls: vec![5, 5],
                    die_size: DieSize::D6,
                    modifiers: ModifierSet::new(),
                    subtotal: 10,
                },
            }],
            total: 10,
        };

        character.take_damage(&damage_roll_result);
        character.heal(5);
        assert_eq!(character.current_hp, 15);
    }

    #[test]
    fn test_character_is_alive() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert!(character.is_alive());

        let damage_roll_result = DamageRollResult {
            label: "Power Word Kill".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceGroupRollResult {
                    label: "Power Word Kill".to_string(),
                    rolls: vec![100],
                    die_size: DieSize::D100,
                    modifiers: ModifierSet::new(),
                    subtotal: 100,
                },
            }],
            total: 100,
        };

        character.take_damage(&damage_roll_result);
        assert!(!character.is_alive());
    }

    #[test]
    fn test_character_saving_throw_modifier() {
        let mut abilities = HashMap::new();
        let mut strength = AbilityScore::new(Ability::Strength, 16);
        strength
            .modifiers
            .add_modifier(ModifierSource::Item("Ring of Strength".to_string()), 2);
        abilities.insert(Ability::Strength, strength);

        let mut saving_throws = HashMap::new();
        let mut strength_saving_throw = D20Check::new(Proficiency::Proficient);
        strength_saving_throw.modifiers.add_modifier(
            ModifierSource::Item("Strength Saving Throw Item".to_string()),
            3,
        );
        saving_throws.insert(Ability::Strength, strength_saving_throw);

        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 5);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            HashMap::new(),
            saving_throws,
            DamageResistances::new(),
        );

        // 4 (ability) + 3 (item) + 3 (proficiency) = 10
        assert_eq!(
            character.saving_throw_modifier(Ability::Strength).total(),
            10
        );
        println!(
            "Strength Saving Throw Modifier: {} = {:?}",
            character.saving_throw_modifier(Ability::Strength).total(),
            character.saving_throw_modifier(Ability::Strength)
        );
    }

    #[test]
    fn test_character_armor_class_no_dex() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        let equipment: EquipmentItem = EquipmentItem::new(
            "Adamantine Splint Armour".to_string(),
            "The adamantine plates lock and slide together perfectly - offering protection against even the deadliest of blades.".to_string(),
            19.0,
            18,
            ItemRarity::VeryRare,
            EquipmentType::Armor,
        );
        let armor = Armor::heavy(equipment, 18);

        character.equip_armor(armor);

        let armor_class = character.armor_class();
        assert_eq!(18, armor_class.total());
        println!("{:?}", armor_class);
    }

    #[test]
    fn test_character_armor_class_dex_and_bonus() {
        let mut character = Character::new(
            "Thorin",
            HashMap::new(),
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );
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
            "Spidersilk Armour".to_string(),
            "Tracings of glossy black spider-web mark this drow-made armour. Its supple, but strong - and made to blend in with the dark caves and crevices of the Underdark.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );
        equipment.add_on_equip(|character| {
            character.add_skill_modifier(
                Skill::Stealth,
                ModifierSource::Item("Spidersilk Armour".to_string()),
                1,
            );
            character
                .saving_throws
                .get_mut(&Ability::Constitution)
                .unwrap()
                .advantage_tracker_mut()
                .add(
                    AdvantageType::Advantage,
                    ModifierSource::Item("Spidersilk Armour".to_string()),
                );
        });
        equipment.add_on_unequip(|character| {
            character.remove_skill_modifier(
                Skill::Stealth,
                &ModifierSource::Item("Spidersilk Armour".to_string()),
            );
            character
                .saving_throws
                .get_mut(&Ability::Constitution)
                .unwrap()
                .advantage_tracker_mut()
                .remove(&ModifierSource::Item("Spidersilk Armour".to_string()));
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
        assert_eq!(armor_name, "Spidersilk Armour");
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
