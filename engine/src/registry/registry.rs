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
    id::ItemId,
    items::inventory::{ItemContainer, ItemInstance},
};

#[derive(Debug, Clone)]
pub struct Registry<K, V> {
    pub entries: HashMap<K, V>,
}

pub trait RegistryEntry {
    type Id: Eq + Hash + Clone + Debug;

    fn id(&self) -> Self::Id;
}

#[derive(Debug)]
pub enum RegistryError {
    DuplicateIdError(String),
    LoadError(std::io::Error),
    JsonError(serde_json::Error),
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        RegistryError::LoadError(err)
    }
}

impl From<serde_json::Error> for RegistryError {
    fn from(err: serde_json::Error) -> Self {
        RegistryError::JsonError(err)
    }
}

impl<K, V> Registry<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: RegistryEntry<Id = K> + DeserializeOwned,
{
    pub fn load_from_directory(directory: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let mut entries = HashMap::new();

        println!("Loading registry from directory: {:?}", directory.as_ref());

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path: PathBuf = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let file_contents: String = fs::read_to_string(&path)?;
            let value: V = serde_json::from_str(&file_contents)?;

            let id: K = value.id().clone();

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

pub struct RegistrySet {
    // pub actions: HashMap<ActionId, Action>,
    // pub classes: HashMap<ClassId, Class>,
    pub items: Registry<ItemId, ItemInstance>,
    // pub spells: HashMap<SpellId, Spell>,
}

impl RegistrySet {
    pub fn load_from_root_directory(
        root_directory: impl AsRef<Path>,
    ) -> Result<Self, RegistryError> {
        let root_directory = root_directory.as_ref();

        // let classes_directory = root_directory.join("classes");
        // let spells_directory  = root_directory.join("spells");
        let items_directory = root_directory.join("items");

        // TODO: Some unit tests rely on the contents of the registries
        // TODO: Load order matters if there are dependencies between registries
        // OOOOORRRR since they're just usind IDs to reference each other, maybe it doesn't
        // Probably something like:
        // 1. Resources
        // 2. Effects
        // 3. Feats
        // 4. Actions
        // 5. Spells
        // 6. Items
        // 7. Classes
        // 8. Backgrounds
        // 9. Races

        Ok(Self {
            // classes: Registry::load_from_directory(classes_directory)?,
            // spells: Registry::load_from_directory(spells_directory)?,
            items: Registry::load_from_directory(items_directory)?,
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

pub struct ItemsRegistry;
pub struct SpellsRegistry;
pub struct ClassesRegistry;

pub fn registry() -> std::sync::RwLockReadGuard<'static, RegistrySet> {
    REGISTRIES.read().unwrap()
}

impl ItemsRegistry {
    pub fn get(id: &ItemId) -> Option<ItemInstance> {
        registry().items.entries.get(id).cloned()
    }
}
