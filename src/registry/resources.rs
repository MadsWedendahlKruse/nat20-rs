use std::sync::LazyLock;

use crate::utils::id::ResourceId;

// TODO: Feels a bit over-engineered
fn resource_key(name: &str) -> ResourceId {
    ResourceId::from_str(format!("resource.{}", name))
}

macro_rules! resource {
    ($($name:ident),* $(,)?) => {
        $(
            pub static $name: LazyLock<ResourceId> = LazyLock::new(|| resource_key(&stringify!($name).to_lowercase().replace('_', ".")));
        )*
    };
}

resource!(ACTION, BONUS_ACTION, REACTION);
