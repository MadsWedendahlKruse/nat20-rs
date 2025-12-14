use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::de::DeserializeOwned;

use crate::{
    components::{
        actions::action::Action,
        background::Background,
        class::{Class, Subclass},
        effects::effects::Effect,
        faction::Faction,
        feat::Feat,
        id::{
            ActionId, BackgroundId, ClassId, EffectId, FactionId, FeatId, IdProvider, ItemId,
            ResourceId, ScriptId, SpeciesId, SpellId, SubclassId, SubspeciesId,
        },
        items::inventory::ItemInstance,
        resource::ResourceDefinition,
        species::{Species, Subspecies},
        spells::spell::Spell,
    },
    scripts::script::Script,
};

pub static REGISTRIES_FOLDER: &str = "registries";

// TODO: Make this configurable?
pub static REGISTRY_ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../assets/{}", REGISTRIES_FOLDER))
});

static REGISTRIES: LazyLock<RegistrySet> = LazyLock::new(|| {
    RegistrySet::load_from_root_directory(&*REGISTRY_ROOT).expect("Failed to load registries")
});

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
        println!("Loading registry from directory: {:?}", directory.as_ref());
        Self::load_directory_recursive(directory.as_ref(), &mut entries)?;
        Ok(Self { entries })
    }

    fn load_directory_recursive(
        directory: &Path,
        entries: &mut HashMap<K, V>,
    ) -> Result<(), RegistryError> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::load_directory_recursive(&path, entries)?;
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            print!("Loading file: {:?}", path);
            let value = Self::load_file(&path)?;
            let id = value.id().clone();
            print!("\rLoaded entry: {:?} from file {:?}\n", id, path);

            if let Some(_) = entries.insert(id.clone(), value) {
                return Err(RegistryError::DuplicateIdError(format!(
                    "Duplicate ID found: {:?} in file {:?}",
                    id, path
                )));
            }
        }

        Ok(())
    }

    fn load_file(path: &Path) -> Result<V, RegistryError> {
        let file_contents = fs::read_to_string(path)?;
        let value = serde_json::from_str::<V>(&file_contents).map_err(|e| {
            RegistryError::LoadError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to deserialize file {:?}: {}", path, e),
            ))
        })?;

        Ok(value)
    }
}

pub struct RegistrySet {
    pub actions: Registry<ActionId, Action>,
    pub backgrounds: Registry<BackgroundId, Background>,
    pub classes: Registry<ClassId, Class>,
    pub effects: Registry<EffectId, Effect>,
    pub factions: Registry<FactionId, Faction>,
    pub feats: Registry<FeatId, Feat>,
    pub items: Registry<ItemId, ItemInstance>,
    pub resources: Registry<ResourceId, ResourceDefinition>,
    pub scripts: Registry<ScriptId, Script>,
    pub species: Registry<SpeciesId, Species>,
    pub spells: Registry<SpellId, Spell>,
    pub subclasses: Registry<SubclassId, Subclass>,
    pub subspecies: Registry<SubspeciesId, Subspecies>,
}

impl RegistrySet {
    pub fn load_from_root_directory(
        root_directory: impl AsRef<Path>,
    ) -> Result<Self, RegistryError> {
        let root_directory = root_directory.as_ref();

        let actions_directory = root_directory.join("actions");
        let backgrounds_directory = root_directory.join("backgrounds");
        let classes_directory = root_directory.join("classes");
        let effects_directory = root_directory.join("effects");
        let factions_directory = root_directory.join("factions");
        let feats_directory = root_directory.join("feats");
        let items_directory = root_directory.join("items");
        let resources_directory = root_directory.join("resources");
        let species_directory = root_directory.join("species");
        let spells_directory = root_directory.join("spells");
        let subclasses_directory = root_directory.join("subclasses");
        let subspecies_directory = root_directory.join("subspecies");

        // Scripts can be in all directories, so we load them separately
        let all_directories = vec![
            actions_directory.as_path(),
            backgrounds_directory.as_path(),
            classes_directory.as_path(),
            effects_directory.as_path(),
            factions_directory.as_path(),
            feats_directory.as_path(),
            items_directory.as_path(),
            resources_directory.as_path(),
            species_directory.as_path(),
            spells_directory.as_path(),
            subclasses_directory.as_path(),
            subspecies_directory.as_path(),
        ];

        let mut scripts = HashMap::new();
        for directory in all_directories {
            if !directory.exists() {
                continue;
            }

            let mut stack = vec![directory.to_path_buf()];
            while let Some(dir) = stack.pop() {
                println!("Stack: {:?}", stack);
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }

                    if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                        continue;
                    }

                    match Script::try_from(entry) {
                        Ok(script) => {
                            let id = script.id.clone();
                            println!("Loaded script: {:?}", id);
                            if scripts.insert(id.clone(), script).is_some() {
                                return Err(RegistryError::DuplicateIdError(format!(
                                    "Duplicate Script ID found: {:?}",
                                    id
                                )));
                            }
                        }
                        Err(err) => eprintln!("Failed to load script: {:?}", err),
                    }
                }
            }
        }

        println!("Loaded {} scripts", scripts.len());

        Ok(Self {
            actions: Registry::load_from_directory(actions_directory)?,
            backgrounds: Registry::load_from_directory(backgrounds_directory)?,
            classes: Registry::load_from_directory(classes_directory)?,
            effects: Registry::load_from_directory(effects_directory)?,
            factions: Registry::load_from_directory(factions_directory)?,
            feats: Registry::load_from_directory(feats_directory)?,
            items: Registry::load_from_directory(items_directory)?,
            resources: Registry::load_from_directory(resources_directory)?,
            scripts: Registry { entries: scripts },
            species: Registry::load_from_directory(species_directory)?,
            spells: Registry::load_from_directory(spells_directory)?,
            subclasses: Registry::load_from_directory(subclasses_directory)?,
            subspecies: Registry::load_from_directory(subspecies_directory)?,
        })
    }
}

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
define_registry!(BackgroundsRegistry, BackgroundId, Background, backgrounds);
define_registry!(ClassesRegistry, ClassId, Class, classes);
define_registry!(EffectsRegistry, EffectId, Effect, effects);
define_registry!(FactionsRegistry, FactionId, Faction, factions);
define_registry!(FeatsRegistry, FeatId, Feat, feats);
define_registry!(ItemsRegistry, ItemId, ItemInstance, items);
define_registry!(ResourcesRegistry, ResourceId, ResourceDefinition, resources);
define_registry!(ScriptsRegistry, ScriptId, Script, scripts);
define_registry!(SpeciesRegistry, SpeciesId, Species, species);
define_registry!(SpellsRegistry, SpellId, Spell, spells);
define_registry!(SubclassesRegistry, SubclassId, Subclass, subclasses);
define_registry!(SubspeciesRegistry, SubspeciesId, Subspecies, subspecies);
