use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::de::DeserializeOwned;

use crate::components::{
    actions::action::Action,
    background::Background,
    class::{Class, Subclass},
    id::{ActionId, BackgroundId, ClassId, IdProvider, ItemId, ResourceId, SpellId, SubclassId},
    items::inventory::ItemInstance,
    resource::ResourceDefinition,
    spells::spell::Spell,
};

#[derive(Debug, Clone)]
pub struct Registry<K, V> {
    pub entries: HashMap<K, V>,
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
    V: IdProvider<Id = K> + DeserializeOwned,
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

pub struct RegistrySet {
    pub actions: Registry<ActionId, Action>,
    pub spells: Registry<SpellId, Spell>,
    pub backgrounds: Registry<BackgroundId, Background>,
    pub classes: Registry<ClassId, Class>,
    pub subclasses: Registry<SubclassId, Subclass>,
    pub items: Registry<ItemId, ItemInstance>,
    pub resources: Registry<ResourceId, ResourceDefinition>,
}

impl RegistrySet {
    pub fn load_from_root_directory(
        root_directory: impl AsRef<Path>,
    ) -> Result<Self, RegistryError> {
        let root_directory = root_directory.as_ref();

        let actions_directory = root_directory.join("actions");
        let spells_directory = root_directory.join("spells");
        let backgrounds_directory = root_directory.join("backgrounds");
        let classes_directory = root_directory.join("classes");
        let subclasses_directory = root_directory.join("subclasses");
        let items_directory = root_directory.join("items");
        let resources_directory = root_directory.join("resources");

        Ok(Self {
            actions: Registry::load_from_directory(actions_directory)?,
            spells: Registry::load_from_directory(spells_directory)?,
            backgrounds: Registry::load_from_directory(backgrounds_directory)?,
            classes: Registry::load_from_directory(classes_directory)?,
            subclasses: Registry::load_from_directory(subclasses_directory)?,
            items: Registry::load_from_directory(items_directory)?,
            resources: Registry::load_from_directory(resources_directory)?,
        })
    }
}

static REGISTRIES: LazyLock<RegistrySet> = LazyLock::new(|| {
    // TODO: Make this configurable
    // TODO: Temporary workaround for getting the correct path in tests
    let registry_root: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/registries");

    RegistrySet::load_from_root_directory(registry_root).expect("Failed to load registries")
});

macro_rules! define_registry {
    ($registry_name:ident, $key_type:ty, $value_type:ty, $field:ident) => {
        pub struct $registry_name;

        impl $registry_name {
            pub fn get(key: &$key_type) -> Option<&'static $value_type> {
                REGISTRIES.$field.entries.get(key)
            }

            pub fn keys() -> impl Iterator<Item = &'static $key_type> + 'static {
                REGISTRIES.$field.entries.keys()
            }

            pub fn values() -> impl Iterator<Item = &'static $value_type> + 'static {
                REGISTRIES.$field.entries.values()
            }
        }
    };
}

define_registry!(ActionsRegistry, ActionId, Action, actions);
define_registry!(SpellsRegistry, SpellId, Spell, spells);
define_registry!(BackgroundsRegistry, BackgroundId, Background, backgrounds);
define_registry!(ClassesRegistry, ClassId, Class, classes);
define_registry!(SubclassesRegistry, SubclassId, Subclass, subclasses);
define_registry!(ResourcesRegistry, ResourceId, ResourceDefinition, resources);
define_registry!(ItemsRegistry, ItemId, ItemInstance, items);
