// TODO: For now let's just hardcode a heigh for each size and assume all creatures
// have a "capsule" collision shape. Later we can make this more complex if needed.

use std::{collections::HashMap, sync::LazyLock};

use hecs::{Entity, World};
use parry3d::{
    na::{Isometry3, Point3, Vector3},
    query::{Ray, RayCast},
    shape::{Capsule, SharedShape},
};

use crate::{
    components::race::CreatureSize,
    engine::{
        game_state::{self, GameState},
        geometry,
    },
};

pub type CreaturePose = Isometry3<f32>;

pub static CREATURE_HEIGHTS: LazyLock<HashMap<CreatureSize, f32>> = LazyLock::new(|| {
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
        // Height is supposed to be the entire capsule height, so the half cylinder
        // height is really just a quarter of the total height.
        Some(Capsule::new_y(height / 4.0, radius))
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaycastResultKind {
    World,
    Creature(Entity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaycastResult {
    pub kind: RaycastResultKind,
    pub toi: f32,
}

static DEFAULT_MAX_TOI: f32 = 10000.0;

pub fn raycast(game_state: &GameState, ray: &Ray) -> Option<RaycastResult> {
    raycast_with_toi(game_state, ray, DEFAULT_MAX_TOI)
}

pub fn raycast_with_toi(
    game_state: &GameState,
    ray: &Ray,
    max_time_of_impact: f32,
) -> Option<RaycastResult> {
    let world = &game_state.world;

    let world_result = if let Some(geometry) = &game_state.geometry {
        let mesh = &geometry.mesh;
        if let Some(toi) = mesh.cast_local_ray(ray, max_time_of_impact, true) {
            Some(RaycastResult {
                kind: RaycastResultKind::World,
                toi,
            })
        } else {
            None
        }
    } else {
        None
    };

    let entity_result = world
        .query::<&CreaturePose>()
        .iter()
        .filter_map(|(entity, pose)| {
            if let Some(shape) = get_shape(world, entity) {
                let toi = shape.cast_ray(pose, ray, max_time_of_impact, true);
                toi.map(|toi| RaycastResult {
                    kind: RaycastResultKind::Creature(entity),
                    toi,
                })
            } else {
                None
            }
        })
        .min_by(|a, b| a.toi.partial_cmp(&b.toi).unwrap());

    match (world_result, entity_result) {
        (Some(wr), Some(er)) => {
            if wr.toi < er.toi {
                Some(wr)
            } else {
                Some(er)
            }
        }
        (Some(wr), None) => Some(wr),
        (None, Some(er)) => Some(er),
        (None, None) => None,
    }
}
