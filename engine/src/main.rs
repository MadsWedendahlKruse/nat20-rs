use hecs::World;
use nat20_rs::entities::character::Character;

fn main() {
    let mut world = World::new();

    let character = world.spawn(Character::new("Hero"));
}
