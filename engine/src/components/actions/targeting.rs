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
    components::{faction::Attitude, health::life_state::LifeState, id::EntityIdentifier},
    systems,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TargetingKind {
    // TODO: I think None and SelfTarget are the same?
    // None,       // e.g. Rage
    SelfTarget, // e.g. Second Wind
    Single,
    Multiple {
        max_targets: u8,
        // kind: TargetKind,
    },
    Area {
        shape: AreaShape,
        origin: Point3<f32>,
    },
    // e.g. Knock
    // Object {
    //     object_type: ObjectType,
    // },
    // Custom(Arc<dyn TargetingLogic>), // fallback for edge cases
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
pub enum TargetType {
    /// An entity in the game world, e.g. a character or a monster.
    Entity {
        /// If specified, the target must be in one of these states to be valid.
        /// If `invert` is true, the target must NOT be in one of these states.
        /// In most cases this is used to prevent targeting dead creatures, but
        /// in some cases it could be used to specifically target dead creatures
        /// (e.g. Revivify).
        allowed_states: HashSet<LifeState>,
        /// If true, the allowed_states set is inverted, i.e. the target must NOT
        /// be in one of the allowed states.
        invert: bool,
    },

    // TODO: Am I mixing up TargetType and TargetingKind here?
    /// A specific point in the game world, e.g. for area-of-effect spells.
    Point,
}

impl TargetType {
    pub fn entity_not_dead() -> Self {
        TargetType::Entity {
            allowed_states: HashSet::from([LifeState::Dead, LifeState::Defeated]),
            invert: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetTypeInstance {
    // TODO: Do we need all of these?
    // Entity(EntityIdentifier),
    Entity(Entity),
    Point(Point3<f32>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetSelection {
    targets: Vec<TargetTypeInstance>,
    valid_target: TargetType,
    max_targets: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetingError {
    ExceedsMaxTargets,
    InvalidTargetType {
        target: TargetTypeInstance,
        valid: TargetType,
    },
    OutOfRange {
        target: TargetTypeInstance,
        distance: Length,
        max_range: Length,
    },
}

impl TargetSelection {
    pub fn new(allowed_type: TargetType, max_allowed_targets: usize) -> Self {
        TargetSelection {
            targets: Vec::new(),
            valid_target: allowed_type,
            max_targets: max_allowed_targets,
        }
    }

    pub fn add_target(&mut self, target: TargetTypeInstance) -> Result<(), TargetingError> {
        if !matches!(
            (&self.valid_target, &target),
            (TargetType::Entity { .. }, TargetTypeInstance::Entity(_))
                | (TargetType::Point, TargetTypeInstance::Point(_))
        ) {
            return Err(TargetingError::InvalidTargetType {
                target,
                valid: self.valid_target.clone(),
            });
        }
        if self.targets.len() >= self.max_targets {
            return Err(TargetingError::ExceedsMaxTargets);
        }
        self.targets.push(target);
        Ok(())
    }

    pub fn targets(&self) -> &Vec<TargetTypeInstance> {
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
    pub valid_target: TargetType,
}

impl TargetingContext {
    pub fn new(kind: TargetingKind, range: TargetingRange, valid_target: TargetType) -> Self {
        TargetingContext {
            kind,
            range,
            valid_target,
        }
    }

    pub fn self_target() -> Self {
        TargetingContext {
            kind: TargetingKind::SelfTarget,
            range: TargetingRange::new::<meter>(0.0),
            valid_target: TargetType::Entity {
                allowed_states: HashSet::new(),
                invert: false,
            },
        }
    }

    pub fn validate_targets(
        &self,
        world: &World,
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
            let actor_position = systems::geometry::get_position(world, actor).unwrap();
            // let distance = match target {
            //     TargetTypeInstance::Entity(entity) => {
            //         let target_position =
            //             // systems::geometry::get_position(world, entity.id()).unwrap();
            //             systems::geometry::get_position(world, *entity).unwrap();
            //         Length::new::<meter>((target_position - actor_position).norm())
            //     }

            //     TargetTypeInstance::Point(point) => {
            //         Length::new::<meter>((point - actor_position).norm())
            //     }
            // };
            let target_position = systems::geometry::get_position(world, *target).unwrap();
            let distance = Length::new::<meter>((target_position - actor_position).norm());

            if !self.range.in_range(distance) {
                return Err(TargetingError::OutOfRange {
                    target: TargetTypeInstance::Entity(*target),
                    distance,
                    max_range: self.range.max(),
                });
            }
        }

        Ok(())
    }
}
