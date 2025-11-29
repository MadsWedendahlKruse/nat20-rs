use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
};

use serde::de::DeserializeOwned;

use crate::components::{
    background::Background,
    class::{Class, Subclass},
    id::{BackgroundId, ClassId, ItemId, ResourceId, SubclassId},
    items::inventory::{ItemContainer, ItemInstance},
    resource::ResourceDefinition,
};

#[derive(Debug, Clone)]
pub struct Registry<K, V> {
    pub entries: HashMap<K, V>,
}

pub trait RegistryEntry {
    type Id: Eq + Hash + Clone + Debug;

    fn id(&self) -> Self::Id;
}

macro_rules! impl_registry_entry {
    ($type:ty, $id_type:ty, $field:ident) => {
        impl RegistryEntry for $type {
            type Id = $id_type;

            fn id(&self) -> &Self::Id {
                &self.$field
            }
        }
    };
}

#[derive(Debug)]
pub enum RegistryError {
    DuplicateIdError(String),
    LoadError(std::io::Error),
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        RegistryError::LoadError(err)
    }
}

impl<K, V> Registry<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: RegistryEntry<Id = K> + DeserializeOwned,
{
    pub fn load_from_directory(directory: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let mut entries = HashMap::new();

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path: PathBuf = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let file_contents: String = fs::read_to_string(&path)?;
            let serde_result = serde_json::from_str::<V>(&file_contents);
            if let Err(e) = serde_result {
                eprintln!("Failed to deserialize file {:?}: {}", path, e.to_string());
                continue;
            }
            let value = serde_result.unwrap();

            let id = value.id().clone();

            if let Some(_) = entries.insert(id.clone(), value) {
                return Err(RegistryError::DuplicateIdError(format!(
                    "Duplicate ID found: {:?} in file {:?}",
                    id, path
                )));
            }
        }

        Ok(Self { entries })
    }
}

// TODO: Not sure where this belongs best
impl RegistryEntry for ItemInstance {
    type Id = ItemId;

    fn id(&self) -> Self::Id {
        self.item().id.clone()
    }
}

impl RegistryEntry for ResourceDefinition {
    type Id = ResourceId;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl RegistryEntry for Class {
    type Id = ClassId;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl RegistryEntry for Subclass {
    type Id = SubclassId;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl RegistryEntry for Background {
    type Id = BackgroundId;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

pub struct RegistrySet {
    // pub actions: Registry<ActionId, Action>,
    pub backgrounds: Registry<BackgroundId, Background>,
    pub classes: Registry<ClassId, Class>,
    pub subclasses: Registry<SubclassId, Subclass>,
    pub items: Registry<ItemId, ItemInstance>,
    // pub spells: Registry<SpellId, Spell>,
    pub resources: Registry<ResourceId, ResourceDefinition>,
}

impl RegistrySet {
    pub fn load_from_root_directory(
        root_directory: impl AsRef<Path>,
    ) -> Result<Self, RegistryError> {
        let root_directory = root_directory.as_ref();

        let backgrounds_directory = root_directory.join("backgrounds");
        let classes_directory = root_directory.join("classes");
        let subclasses_directory = root_directory.join("subclasses");
        // let spells_directory  = root_directory.join("spells");
        let items_directory = root_directory.join("items");
        let resources_directory = root_directory.join("resources");

        Ok(Self {
            backgrounds: Registry::load_from_directory(backgrounds_directory)?,
            classes: Registry::load_from_directory(classes_directory)?,
            subclasses: Registry::load_from_directory(subclasses_directory)?,
            // spells: Registry::load_from_directory(spells_directory)?,
            items: Registry::load_from_directory(items_directory)?,
            resources: Registry::load_from_directory(resources_directory)?,
        })
    }
}

static REGISTRIES: LazyLock<RwLock<RegistrySet>> = LazyLock::new(|| {
    // TODO: Make this configurable
    // TODO: Temporary workaround for getting the correct path in tests
    let registry_root: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/registries");

    let set =
        RegistrySet::load_from_root_directory(registry_root).expect("Failed to load registries");

    RwLock::new(set)
});

// pub struct ItemsRegistry;
// pub struct SpellsRegistry;
// pub struct ClassesRegistry;
// pub struct SubclassesRegistry;
// pub struct ResourcesRegistry;
// pub struct BackgroundsRegistry;

pub fn registry() -> std::sync::RwLockReadGuard<'static, RegistrySet> {
    REGISTRIES.read().unwrap()
}

// TODO: Right now it's convenient to just clone everything, but we might want to
// consider the performance implications of this later on.

macro_rules! define_registry {
    ($registry_name:ident, $key_type:ty, $value_type:ty, $field:ident) => {
        pub struct $registry_name;

        impl $registry_name {
            pub fn get(key: &$key_type) -> Option<$value_type> {
                registry().$field.entries.get(key).cloned()
            }

            pub fn keys() -> Vec<$key_type> {
                registry().$field.entries.keys().cloned().collect()
            }
        }
    };
}

define_registry!(ItemsRegistry, ItemId, ItemInstance, items);
define_registry!(ResourcesRegistry, ResourceId, ResourceDefinition, resources);
define_registry!(ClassesRegistry, ClassId, Class, classes);
define_registry!(SubclassesRegistry, SubclassId, Subclass, subclasses);
define_registry!(BackgroundsRegistry, BackgroundId, Background, backgrounds);
