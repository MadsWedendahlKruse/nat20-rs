extern crate nat20_rs;

mod tests {
    use nat20_rs::combat::damage::*;
    use nat20_rs::creature::character::*;
    use nat20_rs::dice::dice::*;
    use nat20_rs::stats::modifier::*;

    #[test]
    fn character_creation() {
        // let mut class_levels = HashMap::new();
        // class_levels.insert(CharacterClass::Fighter, 3);
        // class_levels.insert(CharacterClass::Wizard, 2);

        // let mut abilities = HashMap::new();
        // abilities.insert(Ability::Strength, AbilityScore::new(Ability::Strength, 16));

        // let mut skills = HashMap::new();
        // skills.insert(Skill::Athletics, D20Check::new(Proficiency::Proficient));

        let character = Character::default();

        assert_eq!(character.name, "John Doe");
        assert_eq!(character.max_hp, 10);
        assert_eq!(character.current_hp, 10);
    }

    #[test]
    fn character_total_level() {
        let mut character = Character::default();
        character.add_class_level(CharacterClass::Fighter, 2);
        character.add_class_level(CharacterClass::Wizard, 3);

        assert_eq!(character.total_level(), 5);
    }

    #[test]
    fn character_proficiency_bonus() {
        let mut character = Character::default();
        character.add_class_level(CharacterClass::Fighter, 10);

        assert_eq!(character.proficiency_bonus(), 4);

        character.add_class_level(CharacterClass::Cleric, 3);

        assert_eq!(character.proficiency_bonus(), 5);

        character.add_class_level(CharacterClass::Rogue, 5);

        assert_eq!(character.proficiency_bonus(), 6);
    }

    #[test]
    fn character_skill_modifier() {

    }

    #[test]
    fn character_take_damage() {
        let mut character = Character::default();
        character.max_hp = 20;
        character.current_hp = 20;
        assert_eq!(character.current_hp, 20);

        character.resistances_mut().add_effect(
            DamageType::Fire,
            DamageMitigationEffect {
                source: ModifierSource::Item("Boots of Fire Resistance".to_string()),
                operation: MitigationOperation::Resistance,
            },
        );

        let damage_roll_result = DamageRollResult {
            label: "Fireball".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Fire,
                result: DiceSetRollResult {
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
    fn character_heal() {
        let mut character = Character::default();
        character.max_hp = 20;
        character.current_hp = 20;

        let damage_roll_result = DamageRollResult {
            label: "Arrow".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceSetRollResult {
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
    fn character_is_alive() {
        let mut character = Character::default();

        assert!(character.is_alive());

        let damage_roll_result = DamageRollResult {
            label: "Power Word Kill".to_string(),
            components: vec![DamageComponentResult {
                damage_type: DamageType::Piercing,
                result: DiceSetRollResult {
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
    fn character_saving_throw_modifier() {
        // TODO: Move test to SavingThrowSet

        // let mut abilities = HashMap::new();
        // let mut strength = AbilityScore::new(Ability::Strength, 16);
        // strength
        //     .modifiers
        //     .add_modifier(ModifierSource::Item("Ring of Strength".to_string()), 2);
        // abilities.insert(Ability::Strength, strength);

        // let mut saving_throws = HashMap::new();
        // let mut strength_saving_throw = D20Check::new(Proficiency::Proficient);
        // strength_saving_throw.modifiers.add_modifier(
        //     ModifierSource::Item("Strength Saving Throw Item".to_string()),
        //     3,
        // );
        // saving_throws.insert(Ability::Strength, strength_saving_throw);

        // let mut class_levels = HashMap::new();
        // class_levels.insert(CharacterClass::Fighter, 5);

        // let character = Character::new(
        //     "Thorin",
        //     class_levels,
        //     20,
        //     abilities,
        //     HashMap::new(),
        //     saving_throws,
        //     DamageResistances::new(),
        // );

        // // 4 (ability) + 3 (item) + 3 (proficiency) = 10
        // assert_eq!(
        //     character.saving_throw_modifier(Ability::Strength).total(),
        //     10
        // );
        // println!(
        //     "Strength Saving Throw Modifier: {} = {:?}",
        //     character.saving_throw_modifier(Ability::Strength).total(),
        //     character.saving_throw_modifier(Ability::Strength)
        // );
    }
}
