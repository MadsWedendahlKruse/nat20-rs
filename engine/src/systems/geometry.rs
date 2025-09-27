// TODO: For now let's just hardcode a heigh for each size and assume all creatures
// have a "capsule" collision shape. Later we can make this more complex if needed.

use std::{collections::HashMap, sync::LazyLock};

use hecs::{Entity, World};
use parry3d::{
    na::{Isometry3, Point3},
    query::RayCast,
    shape::{Capsule, SharedShape},
};

use crate::{
    components::race::CreatureSize,
    engine::game_state::{self, GameState},
};

pub type CreaturePose = Isometry3<f32>;

static CREATURE_HEIGHTS: LazyLock<HashMap<CreatureSize, f32>> = LazyLock::new(|| {
    HashMap::from([
        (CreatureSize::Tiny, 0.5),
        (CreatureSize::Small, 1.0),
        (CreatureSize::Medium, 1.8),
        (CreatureSize::Large, 2.5),
        (CreatureSize::Huge, 4.0),
        (CreatureSize::Gargantuan, 6.0),
    ])
});

pub fn get_height(world: &World, entity: Entity) -> Option<f32> {
    if let Ok(size) = world.get::<&CreatureSize>(entity) {
        CREATURE_HEIGHTS.get(&size).copied()
    } else {
        None
    }
}

pub fn get_shape(world: &World, entity: Entity) -> Option<Capsule> {
    if let Some(height) = get_height(world, entity) {
        // Approximate radius as 1/4 of height
        let radius = height / 4.0;
        Some(Capsule::new_z(height / 2.0, radius))
    } else {
        None
    }
}
