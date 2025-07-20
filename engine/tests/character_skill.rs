// extern crate nat20_rs;

// mod tests {
//     use std::sync::Arc;

//     use nat20_rs::{
//         components::{
//             ability::{Ability, AbilityScore},
//             d20_check::{AdvantageType, RollMode},
//             effects::{
//                 effects::{Effect, EffectDuration},
//                 hooks::SkillCheckHook,
//             },
//             id::EffectId,
//             modifier::ModifierSource,
//             proficiency::Proficiency,
//             skill::Skill,
//         },
//         entities::character,
//     };

//     #[test]
//     fn character_skill_modifier() {
//         let mut character = character::Character::default();

//         character
//             .ability_scores_mut()
//             .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
//         character.ability_scores_mut().add_modifier(
//             Ability::Strength,
//             ModifierSource::Item("Ring of Strength".to_string()),
//             2,
//         );

//         character.skills_mut().add_modifier(
//             Skill::Athletics,
//             ModifierSource::Item("Athlete's Belt".to_string()),
//             1,
//         );

//         // Strength: 17 (base) + 2 (item) = 19
//         assert_eq!(character.ability_scores().total(Ability::Strength), 19);
//         // Calculate the expected skill modifier
//         // 4 (ability) + 1 (item) = 5
//         let skill_modifiers = character
//             .skills()
//             .check(Skill::Athletics, &World, Entity)
//             .modifier_breakdown;
//         println!(
//             "Athletics Modifier: {} = {:?}",
//             skill_modifiers.total(),
//             skill_modifiers
//         );
//         assert_eq!(skill_modifiers.total(), 5);

//         // Perform the skill check
//         let result = character.skill_check(Skill::Athletics);
//         println!("Athletics check: {}", result);
//         // Min roll: 1 + 5 (mod) = 6
//         // Max roll: 20 + 5 (mod) = 25
//         assert!(result.total >= 6);
//         assert!(result.total <= 25);
//     }

//     #[test]
//     fn character_skill_proficiency() {
//         let mut character = character::Character::default();

//         character.ability_scores_mut().set(
//             Ability::Dexterity,
//             AbilityScore::new(Ability::Dexterity, 15),
//         );
//         character.ability_scores_mut().add_modifier(
//             Ability::Dexterity,
//             ModifierSource::Item("Ring of Dexterity".to_string()),
//             2,
//         );

//         character
//             .skills_mut()
//             .set_proficiency(Skill::Stealth, Proficiency::Expertise);

//         // 15 (base) + 2 (item) = 17
//         assert_eq!(character.ability_scores().total(Ability::Dexterity), 17);
//         // Calculate the expected skill modifier
//         // 3 (ability) + 4 (proficiency) = 7
//         let skill_modifiers = character
//             .skills()
//             .check(Skill::Stealth, &World, Entity)
//             .modifier_breakdown;
//         print!(
//             "Stealth Modifier: {} = {:?}",
//             skill_modifiers.total(),
//             skill_modifiers
//         );
//         assert_eq!(skill_modifiers.total(), 7);
//     }

//     #[test]
//     fn character_skill_advantage() {
//         let mut character = character::Character::default();

//         character.ability_scores_mut().set(
//             Ability::Intelligence,
//             AbilityScore::new(Ability::Intelligence, 14),
//         );

//         let mut arcana_effect = Effect::new(
//             EffectId::from_str("effect.ring_of_arcana"),
//             ModifierSource::Item("Ring of Arcana".to_string()),
//             EffectDuration::Permanent,
//         );
//         arcana_effect.on_skill_check = Some(SkillCheckHook {
//             key: Skill::Arcana,
//             check_hook: Arc::new(|_, d20_check| {
//                 d20_check.advantage_tracker_mut().add(
//                     AdvantageType::Advantage,
//                     ModifierSource::Item("Ring of Arcana".to_string()),
//                 );
//             }),
//             result_hook: Arc::new(|_, _| {}),
//         });
//         character.add_effect(arcana_effect);

//         character
//             .skills_mut()
//             .set_proficiency(Skill::Arcana, Proficiency::Proficient);

//         // Calculate the expected skill modifier
//         // 2 (ability) + 2 (proficiency) = 4
//         let check_result = character.skill_check(Skill::Arcana);
//         let skill_modifiers = check_result.modifier_breakdown;
//         print!(
//             "Arcana Modifier: {} = {:?}",
//             skill_modifiers.total(),
//             skill_modifiers
//         );
//         assert_eq!(skill_modifiers.total(), 4);
//         assert!(check_result.advantage_tracker.roll_mode() == RollMode::Advantage);
//     }
// }
