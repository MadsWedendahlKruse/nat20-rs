// TODO: For now let's just hardcode a heigh for each size and assume all creatures
// have a "capsule" collision shape. Later we can make this more complex if needed.

use std::{collections::HashMap, path, sync::LazyLock};

use glam::{Vec2, Vec3};
use hecs::{Entity, World};
use parry3d::{
    na::{Isometry3, Point3, Vector3},
    query::{self, Ray, RayCast},
    shape::{Capsule, Shape, SharedShape},
};
use polyanya::Coords;
use uom::si::{f32::Length, length::meter};

use crate::{
    components::race::CreatureSize,
    engine::{
        game_state::{self, GameState},
        geometry::{self, WorldPath},
    },
};

pub static EPSILON: f32 = 1e-6;

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

pub fn get_eye_height(world: &World, entity: Entity) -> Option<f32> {
    get_height(world, entity).map(|h| h * 0.9)
}

pub fn get_eye_position(world: &World, entity: Entity) -> Option<Point3<f32>> {
    let pose = world.get::<&CreaturePose>(entity).ok()?;
    let eye_height = get_eye_height(world, entity)?;
    Some((pose.translation.vector + Vector3::y() * eye_height).into())
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

#[derive(Debug, Clone, PartialEq)]
pub enum RaycastHitKind {
    World,
    Creature(Entity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaycastHit {
    pub kind: RaycastHitKind,
    /// Time of impact along the ray (distance from ray origin)
    pub toi: f32,
    /// Point of impact in world space
    pub poi: Point3<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RaycastResult {
    pub outcomes: Vec<RaycastHit>,
    pub closest_index: Option<usize>,
}

impl RaycastResult {
    pub fn closest(&self) -> Option<&RaycastHit> {
        self.closest_index.and_then(|i| self.outcomes.get(i))
    }

    pub fn world_hit(&self) -> Option<&RaycastHit> {
        self.outcomes
            .iter()
            .find(|o| matches!(o.kind, RaycastHitKind::World))
    }

    pub fn creature_hit(&self) -> Option<&RaycastHit> {
        self.outcomes
            .iter()
            .find(|o| matches!(o.kind, RaycastHitKind::Creature(_)))
    }
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

    let mut outcomes = vec![];

    if let Some(toi) = game_state
        .geometry
        .trimesh
        .cast_local_ray(ray, max_time_of_impact, true)
    {
        outcomes.push(RaycastHit {
            kind: RaycastHitKind::World,
            toi,
            poi: ray.origin + ray.dir * toi,
        });
    }

    let entity_result = world
        .query::<&CreaturePose>()
        .iter()
        .filter_map(|(entity, pose)| {
            if let Some(shape) = get_shape(world, entity) {
                let toi = shape.cast_ray(pose, ray, max_time_of_impact, true);
                toi.map(|toi| RaycastHit {
                    kind: RaycastHitKind::Creature(entity),
                    toi,
                    poi: ray.origin + ray.dir * toi,
                })
            } else {
                None
            }
        })
        .min_by(|a, b| a.toi.partial_cmp(&b.toi).unwrap());

    if let Some(entity_outcome) = entity_result {
        outcomes.push(entity_outcome);
    }

    if outcomes.is_empty() {
        None
    } else {
        let closest_index = outcomes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.toi.partial_cmp(&b.toi).unwrap())
            .map(|(i, _)| i);
        Some(RaycastResult {
            outcomes,
            closest_index,
        })
    }
}

pub fn raycast_entity_point(
    game_state: &GameState,
    entity: Entity,
    point: Point3<f32>,
) -> Option<RaycastResult> {
    let world = &game_state.world;
    let start = get_eye_position(world, entity)?;
    raycast_point_point(game_state, start, point)
}

pub fn raycast_entity_direction(
    game_state: &GameState,
    entity: Entity,
    direction: Vector3<f32>,
) -> Option<RaycastResult> {
    let world = &game_state.world;
    let start = get_eye_position(world, entity)?;
    raycast_point_direction(game_state, start, direction)
}

pub fn raycast_point_point(
    game_state: &GameState,
    start: Point3<f32>,
    end: Point3<f32>,
) -> Option<RaycastResult> {
    let dir = Vector3::normalize(&(end - start));
    let ray = Ray::new(start, dir);
    raycast(game_state, &ray)
}

pub fn raycast_point_direction(
    game_state: &GameState,
    start: Point3<f32>,
    direction: Vector3<f32>,
) -> Option<RaycastResult> {
    let dir = Vector3::normalize(&direction);
    let ray = Ray::new(start, dir);
    raycast(game_state, &ray)
}

// TODO: How to do this properly? Just because you can't see their eyes doesn't
// mean you can't see them at all.
pub fn entity_line_of_sight(game_state: &GameState, from: Entity, to: Entity) -> Option<bool> {
    let world = &game_state.world;
    let from_pos = get_eye_position(world, from)?;
    let to_pos = get_eye_position(world, to)?;
    let result = raycast_point_point(game_state, from_pos, to_pos)?;
    if let Some(closest) = result.closest()
        && closest.toi < (to_pos - from_pos).norm()
    {
        match &closest.kind {
            RaycastHitKind::World => Some(false),
            RaycastHitKind::Creature(e) if *e == to => Some(true),
            RaycastHitKind::Creature(_) => Some(false),
            _ => Some(false),
        }
    } else {
        // Closest hit is beyond the target
        Some(true)
    }
}

pub fn teleport_to(world: &mut World, entity: Entity, new_position: &Point3<f32>) {
    if let Ok(mut pose) = world.get::<&mut CreaturePose>(entity) {
        pose.translation = new_position.clone().into();
    }
}

pub fn move_to(game_state: &mut GameState, entity: Entity, new_position: &Point3<f32>) {
    let entity_height = get_height(&game_state.world, entity).unwrap_or(0.0);
    // TODO: Can't seem to find an easy way to check for intersection between
    // the creature and the world geometry?

    let mut target = new_position.clone();

    // Raycast straight down to find the ground
    let down_ray = Ray::new(
        Point3::new(target.x, target.y + 1.0, target.z),
        Vector3::new(0.0, -1.0, 0.0),
    );

    if let Some(toi) = game_state
        .geometry
        .trimesh
        .cast_local_ray(&down_ray, 10.0, true)
    {
        // Adjust the creature's Y position to be on the ground
        target.y = down_ray.origin.y + down_ray.dir.y * toi + entity_height / 2.0;
    }

    teleport_to(&mut game_state.world, entity, &target);
}

pub fn navmesh_nearest_point(game_state: &GameState, point: Point3<f32>) -> Option<Point3<f32>> {
    let closest_coord = game_state
        .geometry
        .polyanya_mesh
        .get_closest_point(Coords::on_mesh(Vec2::new(point.x, point.z)))?;

    let coord_3d = closest_coord.position_with_height(&game_state.geometry.polyanya_mesh);
    Some(Point3::new(coord_3d.x, coord_3d.y, coord_3d.z))
}

pub fn path_point_point(
    game_state: &GameState,
    start: Point3<f32>,
    goal: Point3<f32>,
) -> Option<WorldPath> {
    let start = navmesh_nearest_point(game_state, start)?;
    let goal = navmesh_nearest_point(game_state, goal)?;

    let mut path = game_state.geometry.path(start, goal)?;
    let num_points = path.points.len();

    // Snap remaining path points to navmesh
    for point in &mut path.points[1..(num_points - 1)] {
        if let Some(nav_point) = navmesh_nearest_point(game_state, *point) {
            *point = nav_point;
        }
    }

    Some(path)
}

pub fn path(game_state: &GameState, entity: Entity, goal: Point3<f32>) -> Option<WorldPath> {
    let start = game_state.world.get::<&CreaturePose>(entity).ok()?;
    let start_point = Point3::from(start.translation.vector);

    path_point_point(game_state, start_point, goal)
}
