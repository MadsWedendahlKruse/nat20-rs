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
        assert_eq!(character.max_hp(), 20);
        assert_eq!(character.hp(), 20);
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
    fn character_skill_modifier() {}

    #[test]
    fn character_take_damage() {
        let mut character = Character::default();
        assert_eq!(character.hp(), 20);

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
        assert_eq!(character.hp(), 4);
    }

    #[test]
    fn character_heal() {
        let mut character = Character::default();

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
        assert_eq!(character.hp(), 15);
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
}
