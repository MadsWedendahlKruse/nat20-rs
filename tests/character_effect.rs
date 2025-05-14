extern crate nat20_rs;

mod tests {
    use std::{collections::HashSet, sync::Arc};

    use nat20_rs::{
        combat::damage::{DamageRoll, DamageType},
        creature::character::Character,
        dice::dice::DieSize,
        effects::effects::{Effect, EffectDuration},
        item::{
            equipment::{
                equipment::{EquipmentItem, EquipmentType, GeneralEquipmentSlot, HandSlot},
                weapon::{Weapon, WeaponCategory, WeaponType},
            },
            item::ItemRarity,
        },
        stats::{
            d20_check::{AdvantageType, RollMode},
            modifier::ModifierSource,
        },
    };

    #[test]
    fn character_pre_attack_roll_effect() {
        // Create a ring that grants advantage on attack rolls
        let mut advantage_effect = Effect::new(
            ModifierSource::Item("Ring of Advantage".to_string()),
            EffectDuration::Temporary(1),
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
        character.equip_weapon(weapon, HandSlot::Main);

        // No advantage before equipping the ring
        let attack_roll = character.attack_roll(WeaponType::Melee, HandSlot::Main);
        assert_eq!(
            attack_roll.advantage_tracker().roll_mode(),
            RollMode::Normal
        );

        // Advantage after equipping the ring
        character.equip_item(GeneralEquipmentSlot::Ring(0), ring);
        let attack_roll = character.attack_roll(WeaponType::Melee, HandSlot::Main);
        assert_eq!(
            attack_roll.advantage_tracker().roll_mode(),
            RollMode::Advantage
        );
        println!("{:?}", attack_roll);

        // No advantage after unequipping the ring
        character.unequip_item(GeneralEquipmentSlot::Ring(0));
        let attack_roll = character.attack_roll(WeaponType::Melee, HandSlot::Main);
        assert_eq!(
            attack_roll.advantage_tracker().roll_mode(),
            RollMode::Normal
        );
        println!("{:?}", attack_roll);
    }
}
