use std::collections::HashSet;

use hecs::{Entity, World};
use parry3d::na::Point3;
use uom::{
    Conversion,
    si::{
        f32::Length,
        length::{Unit, meter},
    },
};

use crate::{
    components::health::life_state::LifeState,
    engine::game_state::GameState,
    entities::{character::CharacterTag, monster::MonsterTag},
    systems,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TargetingKind {
    SelfTarget, // e.g. Second Wind
    Single,
    Multiple {
        max_targets: u8,
    },
    Area {
        shape: AreaShape,
        fixed_on_actor: bool,
    },
}

// TODO: parry3d shapes?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AreaShape {
    Cone { angle: u32, length: u32 },      // e.g. Cone of Cold
    Sphere { radius: u32 },                // e.g. Fireball
    Cube { side_length: u32 },             // e.g. Wall of Force
    Cylinder { radius: u32, height: u32 }, // e.g. Cloudkill
    Line { length: u32, width: u32 },      // e.g. Lightning Bolt
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityFilter {
    All,
    Characters,
    Monsters,
    Specific(HashSet<Entity>),
    LifeStates(HashSet<LifeState>),
    NotLifeStates(HashSet<LifeState>),
}

impl EntityFilter {
    pub fn not_dead() -> Self {
        EntityFilter::NotLifeStates(HashSet::from([LifeState::Dead, LifeState::Defeated]))
    }

    pub fn matches(&self, world: &World, entity: &Entity) -> bool {
        match self {
            EntityFilter::All => true,
            EntityFilter::Characters => world.get::<&CharacterTag>(*entity).is_ok(),
            EntityFilter::Monsters => world.get::<&MonsterTag>(*entity).is_ok(),
            EntityFilter::Specific(entities) => entities.contains(entity),
            EntityFilter::LifeStates(states) => {
                if let Ok(life_state) = world.get::<&LifeState>(*entity) {
                    states.contains(&life_state)
                } else {
                    false
                }
            }
            EntityFilter::NotLifeStates(states) => {
                if let Ok(life_state) = world.get::<&LifeState>(*entity) {
                    !states.contains(&life_state)
                } else {
                    true
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetInstance {
    Entity(Entity),
    Point(Point3<f32>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetSelection {
    targets: Vec<TargetInstance>,
    max_targets: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetingError {
    ExceedsMaxTargets,
    OutOfRange {
        target: TargetInstance,
        distance: Length,
        max_range: Length,
    },
    NoLineOfSight {
        target: TargetInstance,
    },
}

impl TargetSelection {
    pub fn new(max_allowed_targets: usize) -> Self {
        TargetSelection {
            targets: Vec::new(),
            max_targets: max_allowed_targets,
        }
    }

    pub fn add_target(&mut self, target: TargetInstance) -> Result<(), TargetingError> {
        if self.targets.len() >= self.max_targets {
            return Err(TargetingError::ExceedsMaxTargets);
        }
        self.targets.push(target);
        Ok(())
    }

    pub fn targets(&self) -> &Vec<TargetInstance> {
        &self.targets
    }
}

/// Defines the range parameters for targeting an action.
///
/// `normal` is the range within which the action can be used without penalty.
/// `max` is the maximum range at which the action can be used. Targeting beyond
/// `normal` range may incur penalties (e.g., disadvantage on attack rolls).
///
/// For melee actions, `normal` and `max` are typically the same.
///
/// Note that since there's several places where it is useful to be able to `Hash`
/// a `TargetingRange`, and `f32` does not implement `Hash` (as a consequence neither
/// does `uom::si::Length`), we store the range values as `u32` internally. For
/// the sake of accuracy, these `u32` values represent the range in millimeters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetingRange {
    /// Normal range of the action. Attacks made outside the normal range have
    /// disadvantage on their attack rolls
    normal: u32,
    /// Max range of the action. The action cannot target anything beyond this
    /// range
    max: u32,
}

impl TargetingRange {
    pub fn new<U>(normal: f32) -> Self
    where
        U: Unit + Conversion<f32, T = f32>,
    {
        let normal = Self::length_to_mm(Length::new::<U>(normal));
        TargetingRange {
            max: normal,
            normal,
        }
    }

    pub fn with_max<U>(normal: f32, max: f32) -> Self
    where
        U: Unit + Conversion<f32, T = f32>,
    {
        let normal = Self::length_to_mm(Length::new::<U>(normal));
        let max = Self::length_to_mm(Length::new::<U>(max));
        TargetingRange { normal, max }
    }

    pub fn normal(&self) -> Length {
        Self::mm_to_length(self.normal)
    }

    pub fn max(&self) -> Length {
        Self::mm_to_length(self.max)
    }

    pub fn in_range(&self, distance: Length) -> bool {
        let distance_mm = Self::length_to_mm(distance);
        distance_mm <= self.max
    }

    fn length_to_mm(length: Length) -> u32 {
        let length_mm = length.get::<meter>() * 1000.0;
        length_mm.round() as u32
    }

    fn mm_to_length(mm: u32) -> Length {
        Length::new::<meter>(mm as f32 / 1000.0)
    }
}

#[derive(Debug, Clone)]
pub struct TargetingContext {
    pub kind: TargetingKind,
    pub range: TargetingRange,
    pub require_line_of_sight: bool,
    pub allowed_targets: EntityFilter,
}

impl TargetingContext {
    pub fn new(
        kind: TargetingKind,
        range: TargetingRange,
        require_line_of_sight: bool,
        allowed_targets: EntityFilter,
    ) -> Self {
        TargetingContext {
            kind,
            range,
            require_line_of_sight,
            allowed_targets,
        }
    }

    pub fn self_target() -> Self {
        TargetingContext {
            kind: TargetingKind::SelfTarget,
            range: TargetingRange::new::<meter>(0.0),
            require_line_of_sight: false,
            allowed_targets: EntityFilter::All,
        }
    }

    pub fn validate_targets(
        &self,
        game_state: &GameState,
        actor: Entity,
        targets: &[Entity],
    ) -> Result<(), TargetingError> {
        for target in targets {
            // Check target type
            // if !matches!(
            //     (&self.valid_target, target),
            //     (TargetType::Entity { .. }, TargetTypeInstance::Entity(_))
            //         | (TargetType::Point, TargetTypeInstance::Point(_))
            // ) {
            //     return Err(TargetingError::InvalidTargetType {
            //         target: target.clone(),
            //         valid: self.valid_target.clone(),
            //     });
            // }

            // Check range
            let (_, actor_shape_pose) =
                systems::geometry::get_shape(&game_state.world, actor).unwrap();
            // let distance = match target {
            //     TargetTypeInstance::Entity(entity) => {
            //         let target_position =
            //             // systems::geometry::get_position(&game_state.world, entity.id()).unwrap();
            //             systems::geometry::get_position(&game_state.world, *entity).unwrap();
            //         Length::new::<meter>((target_position - actor_position).norm())
            //     }

            //     TargetTypeInstance::Point(point) => {
            //         Length::new::<meter>((point - actor_position).norm())
            //     }
            // };
            let (_, target_shape_pose) =
                systems::geometry::get_shape(&game_state.world, *target).unwrap();
            let distance = Length::new::<meter>(
                (target_shape_pose.translation.vector - actor_shape_pose.translation.vector).norm(),
            );

            if !self.range.in_range(distance) {
                return Err(TargetingError::OutOfRange {
                    target: TargetInstance::Entity(*target),
                    distance,
                    max_range: self.range.max(),
                });
            }

            // Check line of sight
            if self.require_line_of_sight
                && !systems::geometry::line_of_sight_entity_entity(game_state, actor, *target)
            {
                return Err(TargetingError::NoLineOfSight {
                    target: TargetInstance::Entity(*target),
                });
            }
        }

        Ok(())
    }
}
