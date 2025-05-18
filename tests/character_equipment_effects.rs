extern crate nat20_rs;

mod tests {
    use std::{collections::HashSet, sync::Arc};

    use nat20_rs::{
        combat::damage::{DamageRoll, DamageType},
        creature::character::Character,
        dice::dice::DieSize,
        effects::{
            effects::{Effect, EffectDuration},
            hooks::SavingThrowHook,
        },
        item::{
            equipment::{
                armor::Armor,
                equipment::{EquipmentItem, EquipmentType, GeneralEquipmentSlot, HandSlot},
                weapon::{Weapon, WeaponCategory, WeaponType},
            },
            item::ItemRarity,
        },
        stats::{
            ability::Ability,
            d20_check::{AdvantageType, RollMode},
            modifier::ModifierSource,
            proficiency::Proficiency,
            skill::Skill,
        },
    };

    #[test]
    fn character_pre_attack_roll_effect() {
        // Create a ring that grants advantage on attack rolls
        let mut advantage_effect = Effect::new(
            ModifierSource::Item("Ring of Advantage".to_string()),
            EffectDuration::Persistent,
        );
        advantage_effect.pre_attack_roll = Arc::new(|_, d20_check| {
            d20_check.advantage_tracker_mut().add(
                AdvantageType::Advantage,
                ModifierSource::Item("Ring of Advantage".to_string()),
            );
        });
        let mut ring = EquipmentItem::new(
            "Ring of Advantage".to_string(),
            "A ring that grants advantage on attack rolls.".to_string(),
            0.1,
            100,
            ItemRarity::Uncommon,
            EquipmentType::Ring,
        );
        ring.add_effect(advantage_effect);

        // Create a weapon for the character
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
            DamageRoll::new(1, DieSize::D8, DamageType::Slashing, "Sword".to_string()),
        );

        let mut character = Character::default();
        let _ = character.equip_weapon(weapon, HandSlot::Main);

        // No advantage before equipping the ring
        let attack_roll =
            character
                .loadout()
                .attack_roll(&character, &WeaponType::Melee, HandSlot::Main);
        assert_eq!(attack_roll.advantage_tracker.roll_mode(), RollMode::Normal);

        // Advantage after equipping the ring
        let _ = character.equip_item(GeneralEquipmentSlot::Ring(0), ring);
        let attack_roll =
            character
                .loadout()
                .attack_roll(&character, &WeaponType::Melee, HandSlot::Main);
        assert_eq!(
            attack_roll.advantage_tracker.roll_mode(),
            RollMode::Advantage
        );
        println!("{:?}", attack_roll);

        // No advantage after unequipping the ring
        character.unequip_item(GeneralEquipmentSlot::Ring(0));
        let attack_roll =
            character
                .loadout()
                .attack_roll(&character, &WeaponType::Melee, HandSlot::Main);
        assert_eq!(attack_roll.advantage_tracker.roll_mode(), RollMode::Normal);
        println!("{:?}", attack_roll);
    }

    #[test]
    fn character_skill_bonus_effect() {
        let mut character = Character::default();
        character
            .skills_mut()
            .set_proficiency(Skill::Stealth, Proficiency::Proficient);

        // Create an armor that adds +2 to stealth
        let armor_name = Arc::new("Armor of Sneaking".to_string());
        let modifier_source = ModifierSource::Item(armor_name.clone().to_string());

        let mut equipment: EquipmentItem = EquipmentItem::new(
            armor_name.clone().to_string(),
            "It's made of a lightweight material that allows for silent movement.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );

        let mut armor_effect = Effect::new(modifier_source.clone(), EffectDuration::Persistent);

        // Create a modifier source for each closure (pre_hook and post_hook)
        // to avoid borrowing the same variable multiple times
        let add_skill_source = modifier_source.clone();
        let remove_skill_source = modifier_source.clone();

        armor_effect.on_apply = Arc::new(move |character| {
            character
                .skills_mut()
                .add_modifier(Skill::Stealth, add_skill_source.clone(), 2);
        });
        armor_effect.on_unapply = Arc::new(move |character| {
            character
                .skills_mut()
                .remove_modifier(Skill::Stealth, &remove_skill_source);
        });

        equipment.add_effect(armor_effect);

        let armor = Armor::light(equipment, 12);

        character.equip_armor(armor);

        // Check if the skill modifier is applied
        let stealth_check = character.skills().check(Skill::Stealth, &character);
        println!("{:?}", stealth_check);
        // 2 (proficient) + 2 (armor) = 4
        assert_eq!(4, stealth_check.total_modifier);

        // Un-equip the armor
        let armor_name = character.unequip_armor().unwrap().equipment.item.name;
        println!("Un-equipped {:?}", armor_name);
        assert_eq!(armor_name, "Armor of Sneaking");

        // Check if the skill modifier is removed
        let stealth_check = character.skills().check(Skill::Stealth, &character);
        println!("{:?}", stealth_check);
        // 2 (proficient) + 0 (armor) = 2
        assert_eq!(2, stealth_check.total_modifier);
    }

    #[test]
    fn character_saving_throw_effect() {
        let mut character = Character::default();

        let armor_name = Arc::new("Armor of Resilience".to_string());
        let modifier_source = ModifierSource::Item(armor_name.clone().to_string());

        let mut equipment = EquipmentItem::new(
            "Armor of Resilience".to_string(),
            "A suit of armor that grants advantage on Constitution saving throws.".to_string(),
            5.85,
            1000,
            ItemRarity::Rare,
            EquipmentType::Armor,
        );

        let mut armor_effect = Effect::new(modifier_source.clone(), EffectDuration::Persistent);

        let mut saving_throw_hook = SavingThrowHook::new(Ability::Constitution);
        saving_throw_hook.check_hook = Arc::new(move |_, d20_check| {
            d20_check
                .advantage_tracker_mut()
                .add(AdvantageType::Advantage, modifier_source.clone());
        });
        armor_effect.saving_throw_hook = Some(saving_throw_hook);

        equipment.add_effect(armor_effect);

        let armor = Armor::heavy(equipment, 12);

        character.equip_armor(armor);

        // Check if the advantage is applied
        let constitution_saving_throw = character
            .saving_throws()
            .check(Ability::Constitution, &character);
        assert!(constitution_saving_throw.advantage_tracker.roll_mode() == RollMode::Advantage);

        character.unequip_armor();

        // Check if the advantage is removed
        let constitution_saving_throw = character
            .saving_throws()
            .check(Ability::Constitution, &character);
        assert!(constitution_saving_throw.advantage_tracker.roll_mode() == RollMode::Normal);
    }
}
