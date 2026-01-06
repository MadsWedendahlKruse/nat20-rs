#[macro_export]
macro_rules! from_world {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty,
            )*
        }
    ) => {
        use hecs::{World, Entity};
        use crate::systems;

        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field : $ty,
            )*
        }

        impl $name {
            pub fn from_world(world: &World, entity: Entity) -> Self {
                Self {
                    $(
                        $field: systems::helpers::get_component_clone(world, entity),
                    )*
                }
            }
        }
    }
}
