extern crate nat20_rs;
use nat20_rs::creature::{character::Character, level_up::CliChoiceProvider};

pub fn main() {
    let mut character = Character::new("John Hero");

    for level in 1..=5 {
        println!("{} reached level {}", character.name(), level);
        print_character_classes(&character);

        let mut level_up_session = character.level_up();
        level_up_session
            .advance(&mut CliChoiceProvider)
            .expect("Level-up failed");
    }
}

fn print_character_classes(character: &Character) {
    for (class, level) in character.classes() {
        if let Some(subclass) = character.subclass(class) {
            println!("  - {} {} (Level {})", subclass.name, class, level);
        } else {
            println!("  - {} (Level {})", class, level);
        }
    }
}
