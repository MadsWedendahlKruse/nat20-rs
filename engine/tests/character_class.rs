// extern crate nat20_rs;

// mod tests {

//     use std::collections::{HashMap, HashSet};

//     use nat20_rs::{
//         components::{
//             ability::Ability,
//             class::{ClassName, SubclassName},
//             proficiency::Proficiency,
//             skill::Skill,
//         },
//         entities::character::Character,
//         registry,
//         systems::level_up::{LevelUpSelection, PredefinedChoiceProvider},
//     };

//     #[test]
//     fn character_level_up_fighter() {
//         let mut character = Character::new("John Fighter");
//         let mut choice_provider = PredefinedChoiceProvider::new(
//             character.name().to_string(),
//             vec![
//                 LevelUpSelection::Class(ClassName::Fighter),
//                 LevelUpSelection::Effect(
//                     registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
//                 ),
//                 LevelUpSelection::SkillProficiency(HashSet::from([
//                     Skill::Athletics,
//                     Skill::Perception,
//                 ])),
//                 LevelUpSelection::Class(ClassName::Fighter),
//                 LevelUpSelection::Class(ClassName::Fighter),
//                 LevelUpSelection::Subclass(SubclassName {
//                     class: ClassName::Fighter,
//                     name: "Champion".to_string(),
//                 }),
//             ],
//         );

//         for level in 1..=3 {
//             print!("{}", character);
//             println!("{} reached level {}", character.name(), level);

//             let mut level_up_session = character.level_up();
//             level_up_session
//                 .advance(&mut choice_provider)
//                 .expect("Level-up failed");
//         }

//         assert_eq!(character.total_level(), 3);
//         assert_eq!(
//             *character.classes(),
//             HashMap::from([(ClassName::Fighter, 3),])
//         );
//         assert_eq!(
//             character.subclass(&ClassName::Fighter),
//             Some(&SubclassName {
//                 class: ClassName::Fighter,
//                 name: "Champion".to_string()
//             })
//         );

//         assert_eq!(character.effects().len(), 2);
//         for effect_id in [
//             &registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID,
//             &registry::effects::IMPROVED_CRITICAL_ID,
//         ] {
//             assert!(
//                 character
//                     .effects()
//                     .contains(&registry::effects::EFFECT_REGISTRY.get(&effect_id).unwrap())
//             );
//         }

//         for skill in [Skill::Athletics, Skill::Perception] {
//             assert_eq!(
//                 character.skills().get(skill).proficiency(),
//                 &Proficiency::Proficient
//             );
//         }

//         for ability in [Ability::Strength, Ability::Constitution] {
//             assert_eq!(
//                 character.saving_throws().get(ability).proficiency(),
//                 &Proficiency::Proficient
//             );
//         }
//     }
// }
