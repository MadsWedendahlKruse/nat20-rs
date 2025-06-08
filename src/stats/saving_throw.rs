use crate::{creature::character::Character, effects::hooks::SavingThrowHook};

use super::{
    ability::Ability,
    d20_check::{D20Check, D20CheckDC, D20CheckResult, D20CheckSet},
};

pub type SavingThrowSet = D20CheckSet<Ability, SavingThrowHook>;

pub fn get_saving_throw_hooks(ability: Ability, character: &Character) -> Vec<&SavingThrowHook> {
    character
        .effects()
        .iter()
        .filter_map(|e| e.on_saving_throw.as_ref())
        .filter(|hook| hook.key == ability)
        .collect()
}

pub fn apply_check_hook(hook: &SavingThrowHook, character: &Character, d20: &mut D20Check) {
    (hook.check_hook)(character, d20);
}

pub fn apply_result_hook(
    hook: &SavingThrowHook,
    character: &Character,
    result: &mut D20CheckResult,
) {
    (hook.result_hook)(character, result);
}

pub fn create_saving_throw_set() -> SavingThrowSet {
    SavingThrowSet::new(
        get_saving_throw_hooks,
        apply_check_hook,
        apply_result_hook,
        |k| k,
    )
}

pub type SavingThrowDC = D20CheckDC<Ability>;
