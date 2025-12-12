// pub static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID: LazyLock<EffectId> =
//     LazyLock::new(|| EffectId::from_str("effect.fighting_style.great_weapon_fighting"));

// static FIGHTING_STYLE_GREAT_WEAPON_FIGHTING: LazyLock<Effect> = LazyLock::new(|| {
//     let mut effect = Effect::new(
//         FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
//         ModifierSource::Feat(registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone()),
//         EffectDuration::Permanent,
//     );
//     effect.post_damage_roll = Arc::new(|world, entity, damage_roll_result| {
//         // Great weapon fighting only applies to melee attacks (with both hands)
//         if match &damage_roll_result.source {
//             DamageSource::Weapon(weapon_type) => *weapon_type != WeaponKind::Melee,
//             _ => false,
//         } {
//             return;
//         }

//         let loadout = systems::helpers::get_component::<loadout::Loadout>(world, entity);
//         if !loadout.is_wielding_weapon_with_both_hands(&WeaponKind::Melee) {
//             return;
//         }

//         // TODO: When does this ever happen?
//         if damage_roll_result.components.is_empty() {
//             return;
//         }

//         // First component is the primary damage component
//         let primary_damage_rolls = &mut damage_roll_result.components[0].result.rolls;
//         for i in 0..primary_damage_rolls.len() {
//             // Any roll that is less than 3 is rerolled to 3
//             if primary_damage_rolls[i] < 3 {
//                 primary_damage_rolls[i] = 3
//             }
//         }
//     });
//     effect
// });
