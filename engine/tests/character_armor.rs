// extern crate nat20_rs;

// mod tests {
//     use nat20_rs::{
//         components::{
//             ability::{Ability, AbilityScore},
//             items::{
//                 equipment::{
//                     armor::Armor,
//                     equipment::{EquipmentItem, EquipmentType},
//                 },
//                 item::ItemRarity,
//             },
//             modifier::ModifierSource,
//         },
//         entities::character::Character,
//     };

//     #[test]
//     fn character_armor_class_no_dex() {
//         let mut character = Character::default();

//         let equipment: EquipmentItem = EquipmentItem::new(
//             "Adamantium Armour".to_string(),
//             "A suit of armor made from adamantium.".to_string(),
//             19.0,
//             5000,
//             ItemRarity::VeryRare,
//             EquipmentType::Armor,
//         );
//         let armor = Armor::heavy(equipment, 19);

//         character.equip_armor(armor);

//         let armor_class = character.armor_class();
//         assert_eq!(19, armor_class.total());
//         println!("{:?}", armor_class);
//     }

//     #[test]
//     fn character_armor_class_dex_and_bonus() {
//         // Create a character with a Dexterity modifier of +3
//         let mut character = Character::default();
//         character.ability_scores_mut().set(
//             Ability::Dexterity,
//             AbilityScore::new(Ability::Dexterity, 15),
//         );
//         character.ability_scores_mut().add_modifier(
//             Ability::Dexterity,
//             ModifierSource::Item("Ring of Dexterity".to_string()),
//             2,
//         );

//         let equipment: EquipmentItem = EquipmentItem::new(
//             "Light Armor".to_string(),
//             "A suit of light armor.".to_string(),
//             5.85,
//             1000,
//             ItemRarity::Rare,
//             EquipmentType::Armor,
//         );

//         let armor = Armor::light(equipment, 12);

//         character.equip_armor(armor);

//         let armor_class = character.armor_class();
//         // Armour Class
//         // Dex: 15 + 2 (item) = 17
//         // 12 (armor) + 3 (Dex mod) = 15
//         println!("{:?}", armor_class);
//         assert_eq!(15, armor_class.total());

//         // Un-equip the armor
//         let armor_name = character.unequip_armor().unwrap().equipment.item.name;
//         let armor_class = character.armor_class();
//         println!("Un-equipped {:?}", armor_name);
//         assert_eq!(armor_name, "Light Armor");
//         // Check if the armor class is updated
//         println!("{:?}", armor_class);
//         assert_eq!(10, armor_class.total());
//     }
// }
