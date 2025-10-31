use hecs::{Entity, World};
use parry3d::{
    math::Isometry,
    na::Point3,
    query::{Ray, RayCast},
    shape::Ball,
};
use uom::si::{f32::Length, length::meter};

use crate::{
    components::speed::Speed,
    engine::{game_state::GameState, geometry::WorldPath},
    systems::{
        self,
        geometry::{CreaturePose, RaycastFilter},
    },
};

#[derive(Debug)]
pub enum MovementError {
    InsufficientSpeed,
    NoPathFound,
    // Strictly speaking this isn't a movement error, but it makes it easier to
    // handle in the game state if we put it here ;)
    NotYourTurn,
}

#[derive(Debug, Clone)]
pub struct PathResult {
    pub full_path: WorldPath,
    pub taken_path: WorldPath,
}

impl PathResult {
    pub fn empty() -> Self {
        Self {
            full_path: WorldPath::new(vec![]),
            taken_path: WorldPath::new(vec![]),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.full_path.points.is_empty() && self.taken_path.points.is_empty()
    }

    pub fn reaches_goal(&self) -> bool {
        !self.is_empty() && self.full_path.points.last() == self.taken_path.points.last()
    }
}

pub fn path(
    game_state: &mut GameState,
    entity: Entity,
    goal: &Point3<f32>,
    allow_partial: bool,
    move_entity: bool,
    spend_movement: bool,
) -> Result<PathResult, MovementError> {
    let full_path =
        systems::geometry::path(game_state, entity, *goal).ok_or(MovementError::NoPathFound)?;

    let remaining_movement =
        systems::helpers::get_component_mut::<Speed>(&mut game_state.world, entity)
            .remaining_movement()
            .clone();

    let taken_path = if full_path.length > remaining_movement && spend_movement {
        if !allow_partial {
            return Err(MovementError::InsufficientSpeed);
        }
        full_path.trim_to_length(remaining_movement)
    } else {
        full_path.clone()
    };

    if move_entity {
        // TODO: Actually make them move along the path rather than teleporting to the end
        systems::geometry::teleport_to_ground(
            game_state,
            entity,
            taken_path.points.last().unwrap(),
        );
        if spend_movement {
            systems::helpers::get_component_mut::<Speed>(&mut game_state.world, entity)
                .record_movement(taken_path.length);
        }
    }

    Ok(PathResult {
        full_path,
        taken_path,
    })
}

pub fn path_in_range_of_point(
    game_state: &mut GameState,
    entity: Entity,
    target: Point3<f32>,
    range: Length,
    allow_partial: bool,
    move_entity: bool,
    line_of_sight: bool,
    spend_movement: bool,
) -> Result<PathResult, MovementError> {
    let direction = (target
        - game_state
            .world
            .get::<&CreaturePose>(entity)
            .unwrap()
            .translation
            .vector)
        .to_homogeneous();

    let distance_to_target = Length::new::<meter>(direction.magnitude());

    if distance_to_target <= range {
        if !line_of_sight
            || systems::geometry::line_of_sight_entity_point(game_state, entity, target)
        {
            // Already in range
            println!("Entity is already in range of target point.");
            return Ok(PathResult::empty());
        }
    }

    let path_to_target = path(game_state, entity, &target, true, false, spend_movement)?;

    if let Some(intersection) = determine_path_sphere_intersections(
        game_state,
        entity,
        line_of_sight,
        range,
        &path_to_target.full_path,
    ) {
        return path(
            game_state,
            entity,
            &intersection,
            allow_partial,
            move_entity,
            spend_movement,
        );
    }

    if allow_partial {
        // Return the partial path even if we couldn't get in range
        println!("No intersection found, but allowing partial path.");
        return Ok(path_to_target);
    }

    Err(MovementError::NoPathFound)
}

fn determine_path_sphere_intersections(
    game_state: &mut GameState,
    entity: Entity,
    line_of_sight: bool,
    range: Length,
    path_to_target: &WorldPath,
) -> Option<Point3<f32>> {
    let path_end = path_to_target.points.last()?;

    // Entity shouldn't block its own line of sight
    let mut excluded_entities = vec![entity];
    // If an entity is standing on the end of the path, that's probably who we're
    // trying to target, so don't let them block line of sight either
    if let Some(occupant) = systems::geometry::get_entity_at_point(&game_state.world, *path_end) {
        excluded_entities.push(occupant);
    }
    let raycast_filter = RaycastFilter::ExcludeCreatures(excluded_entities);

    for (start, end) in path_to_target
        .points
        .windows(2)
        .map(|window| (window[0], window[1]))
    {
        let ray = Ray::new(start, (end - start).normalize());
        let sphere = Ball::new(range.get::<meter>());

        if let Some(toi) = sphere.cast_ray(
            &Isometry::translation(path_end.x, path_end.y, path_end.z),
            &ray,
            f32::MAX,
            true,
        ) {
            let intersection_point = ray.point_at(toi);
            let ground_at_intersection =
                systems::geometry::ground_position(&game_state, &intersection_point)?;
            let eye_pos_at_intersection = systems::geometry::get_eye_position_at_point(
                &game_state.world,
                entity,
                &ground_at_intersection,
            )?;
            if line_of_sight
                && !systems::geometry::line_of_sight_point_point(
                    game_state,
                    eye_pos_at_intersection,
                    *path_end,
                    &raycast_filter,
                )
            {
                // No line of sight to this intersection point; try next segment
                continue;
            } else {
                return Some(intersection_point);
            }
        }
    }

    None
}

pub fn recharge_movement(world: &mut World, entity: Entity) {
    systems::helpers::get_component_mut::<Speed>(world, entity).reset();
}
