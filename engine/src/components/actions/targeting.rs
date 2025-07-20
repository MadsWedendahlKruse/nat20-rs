use hecs::Entity;

use crate::math::point::Point;

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
        origin: Point,
    },
    // e.g. Knock
    // Object {
    //     object_type: ObjectType,
    // },
    // Custom(Arc<dyn TargetingLogic>), // fallback for edge cases
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AreaShape {
    Cone { angle: u32, length: u32 },      // e.g. Cone of Cold
    Sphere { radius: u32 },                // e.g. Fireball
    Cube { side_length: u32 },             // e.g. Wall of Force
    Cylinder { radius: u32, height: u32 }, // e.g. Cloudkill
    Line { length: u32, width: u32 },      // e.g. Lightning Bolt
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    // An entity in the game world, e.g. a character or creature
    Entity,
    // Self,       // Explicit "this actor" (optional; could be special-cased)
    // None, // For actions that don't actually select a target
    // Future-proofing:
    // Point, // A position/tile (for teleport, movement, some spells)
    // Area,  // An area, e.g. Cloudkill, Grease (could also be modeled by targeting context)
}

#[derive(Debug)]
pub enum TargetTypeInstance {
    Entity(Entity),
    Point(Point),
    Area(AreaShape),
    None,
}

#[derive(Debug, Clone)]
pub struct TargetingContext {
    pub kind: TargetingKind,
    /// Normal range of the action. Attacks made outside the normal range have
    /// disadvantage on their attack rolls
    pub normal_range: u32,
    /// Max range of the action. The action cannot target anything beyond this
    /// range
    pub max_range: u32,
    pub valid_target_types: Vec<TargetType>,
}

impl TargetingContext {
    pub fn new(
        kind: TargetingKind,
        normal_range: u32,
        valid_target_types: Vec<TargetType>,
    ) -> Self {
        TargetingContext {
            kind,
            normal_range,
            max_range: normal_range,
            valid_target_types,
        }
    }

    pub fn self_target() -> Self {
        TargetingContext {
            kind: TargetingKind::SelfTarget,
            normal_range: 0,
            max_range: 0,
            valid_target_types: vec![TargetType::Entity],
        }
    }
}
