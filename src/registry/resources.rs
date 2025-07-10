use std::sync::LazyLock;

use crate::utils::id::ResourceId;

// TODO: Feels a bit over-engineered
fn resource_key(name: &str) -> ResourceId {
    ResourceId::from_str(format!("resource.{}", name))
}

macro_rules! resource {
    ($($name:ident),* $(,)?) => {
        $(
            pub static $name: LazyLock<ResourceId> = LazyLock::new(|| resource_key(&stringify!($name).to_lowercase()));
        )*
    };
}

resource!(
    // --- DEFAULT RESOURCES ---
    ACTION,
    BONUS_ACTION,
    REACTION,
    // --- CLASS RESOURCES ---
    // - (SHARED MARTIALS) -
    EXTRA_ATTACK,
    // - FIGHTER -
    // Action Surge can be used twice per short rest at level 17. The easiest
    // way to model the charges is to add it as a resource
    ACTION_SURGE,
    // Same as above
    SECOND_WIND,
);
