extern crate nat20_rs;

mod tests {
    use std::sync::Arc;

    use nat20_rs::{
        creature::character::Character,
        effects::{
            effects::{Effect, EffectDuration},
            hooks::SavingThrowHook,
        },
        stats::{
            ability::{Ability, AbilityScore},
            d20_check::{AdvantageType, RollMode},
            modifier::ModifierSource,
            proficiency::Proficiency,
        },
    };

    #[test]
    fn character_saving_throw_modifier() {
        let mut character = Character::default();

        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        character.ability_scores_mut().add_modifier(
            Ability::Strength,
            ModifierSource::Item("Ring of Strength".to_string()),
            2,
        );

        // 17 (base) + 2 (item) = 19
        assert_eq!(character.ability_scores().total(Ability::Strength), 19);
        // Calculate the expected saving throw modifier
        // 4 (ability) = 4
        let saving_throw_modifiers = character.saving_throw(Ability::Strength).modifier_breakdown;
        print!(
            "Saving Throw Modifier: {} = {:?}",
            saving_throw_modifiers.total(),
            saving_throw_modifiers
        );
        assert_eq!(saving_throw_modifiers.total(), 4);
    }

    #[test]
    fn character_saving_throw_proficiency() {
        let mut character = Character::default();

        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        character
            .saving_throws_mut()
            .set_proficiency(Ability::Strength, Proficiency::Proficient);

        // 17 (base) = 17
        assert_eq!(character.ability_scores().total(Ability::Strength), 17);
        // Calculate the expected saving throw modifier
        // 3 (ability) + 2 (proficiency) = 5
        let saving_throw_modifiers = character.saving_throw(Ability::Strength).modifier_breakdown;
        print!(
            "Saving Throw Modifier: {} = {:?}",
            saving_throw_modifiers.total(),
            saving_throw_modifiers
        );
        assert_eq!(saving_throw_modifiers.total(), 5);
    }

    #[test]
    fn character_saving_throw_proficiency_expertise() {
        let mut character = Character::default();

        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        character
            .saving_throws_mut()
            .set_proficiency(Ability::Strength, Proficiency::Expertise);

        // 17 (base) = 17
        assert_eq!(character.ability_scores().total(Ability::Strength), 17);
        // Calculate the expected saving throw modifier
        // 3 (ability) + 4 (proficiency) = 7
        let saving_throw_modifiers = character.saving_throw(Ability::Strength).modifier_breakdown;
        print!(
            "Saving Throw Modifier: {} = {:?}",
            saving_throw_modifiers.total(),
            saving_throw_modifiers
        );
        assert_eq!(saving_throw_modifiers.total(), 7);
    }

    #[test]
    fn character_saving_throw_disadvantage() {
        let mut character = Character::default();

        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 17));
        let mut disadvantage_effect = Effect::new(
            ModifierSource::Spell("Curse of Weakness".to_string()),
            EffectDuration::Temporary(2),
        );
        disadvantage_effect.saving_throw_hook = Some(SavingThrowHook {
            key: Ability::Strength,
            check_hook: Arc::new(move |_, d20_check| {
                d20_check.advantage_tracker_mut().add(
                    AdvantageType::Disadvantage,
                    ModifierSource::Spell("Curse of Weakness".to_string()),
                );
            }),
            result_hook: Arc::new(|_, _| {}),
        });
        character.add_effect(disadvantage_effect);

        let result = character.saving_throw(Ability::Strength);
        assert!(result.advantage_tracker.roll_mode() == RollMode::Disadvantage);
    }
}
