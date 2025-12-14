use std::{
    collections::HashMap,
    fmt::{self, Debug},
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::de::DeserializeOwned;

use tracing::{error, info};

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

static REGISTRIES: LazyLock<RegistrySet> =
    LazyLock::new(
        || match RegistrySet::load_from_root_directory(&*REGISTRY_ROOT) {
            Ok(set) => set,
            Err(error) => {
                error!(path = ?&*REGISTRY_ROOT, %error, "Failed to load registries");
                panic!("Failed to load registries");
            }
        },
    );

#[derive(Debug, Clone)]
pub struct Registry<K, V> {
    pub entries: HashMap<K, V>,
}

#[derive(Debug)]
pub enum RegistryError {
    ReadDirectory {
        directory: PathBuf,
        message: String,
    },
    ReadDirectoryEntry {
        directory: PathBuf,
        message: String,
    },
    ReadFile {
        path: PathBuf,
        message: String,
    },
    DeserializeJson {
        path: PathBuf,
        message: String,
    },
    DuplicateId {
        id_debug: String,
        first_path: PathBuf,
        second_path: PathBuf,
    },
    Many(Vec<RegistryError>),
}

impl RegistryError {
    pub fn push_into(self, errors: &mut Vec<RegistryError>) {
        match self {
            RegistryError::Many(mut inner) => errors.append(&mut inner),
            other => errors.push(other),
        }
    }

    pub fn many_if_needed(errors: Vec<RegistryError>) -> Result<(), RegistryError> {
        if errors.is_empty() {
            Ok(())
        } else {
            Err(RegistryError::Many(errors))
        }
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::ReadDirectory { directory, message } => {
                write!(f, "Failed to read directory {:?}: {}", directory, message)
            }
            RegistryError::ReadDirectoryEntry { directory, message } => {
                write!(
                    f,
                    "Failed to read directory entry in {:?}: {}",
                    directory, message
                )
            }
            RegistryError::ReadFile { path, message } => {
                write!(f, "Failed to read file {:?}: {}", path, message)
            }
            RegistryError::DeserializeJson { path, message } => {
                write!(f, "Failed to deserialize JSON {:?}: {}", path, message)
            }
            RegistryError::DuplicateId {
                id_debug,
                first_path,
                second_path,
            } => {
                write!(
                    f,
                    "Duplicate id {}:\n  first:  {:?}\n  second: {:?}",
                    id_debug, first_path, second_path
                )
            }
            RegistryError::Many(errors) => {
                writeln!(f, "{} registry error(s):", errors.len())?;
                for (index, error) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}", index + 1, error)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RegistryError {}

impl<K, V> Registry<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: IdProvider<Id = K> + DeserializeOwned,
{
    pub fn load_from_directory(directory: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let directory = directory.as_ref();

        let mut entries: HashMap<K, V> = HashMap::new();
        let mut id_to_path: HashMap<K, PathBuf> = HashMap::new();
        let mut errors: Vec<RegistryError> = Vec::new();

        Self::load_directory_recursive(directory, &mut entries, &mut id_to_path, &mut errors);

        if errors.is_empty() {
            Ok(Self { entries })
        } else {
            Err(RegistryError::Many(errors))
        }
    }

    fn load_directory_recursive(
        directory: &Path,
        entries: &mut HashMap<K, V>,
        id_to_path: &mut HashMap<K, PathBuf>,
        errors: &mut Vec<RegistryError>,
    ) {
        let read_dir_iter = match fs::read_dir(directory) {
            Ok(iter) => iter,
            Err(error) => {
                errors.push(RegistryError::ReadDirectory {
                    directory: directory.to_path_buf(),
                    message: error.to_string(),
                });
                return;
            }
        };

        for entry_result in read_dir_iter {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(error) => {
                    errors.push(RegistryError::ReadDirectoryEntry {
                        directory: directory.to_path_buf(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            let path = entry.path();

            if path.is_dir() {
                Self::load_directory_recursive(&path, entries, id_to_path, errors);
                continue;
            }

            if path.extension().is_none()
                || path.extension().and_then(|ext| ext.to_str()) != Some("json")
            {
                info!("Skipping non-json file in registry: {:?}", path);
                continue;
            }

            let value = match Self::load_file(&path) {
                Ok(value) => value,
                Err(error) => {
                    error!(%error, "Failed to load registry entry");
                    error.push_into(errors);
                    continue;
                }
            };

            let id = value.id().clone();

            if let Some(first_path) = id_to_path.get(&id).cloned() {
                errors.push(RegistryError::DuplicateId {
                    id_debug: format!("{:?}", id),
                    first_path,
                    second_path: path.clone(),
                });
                // Decide policy:
                // - Keep first: do not overwrite
                // - Keep last: overwrite
                // Recommended for deterministic behavior: keep first.
                continue;
            }

            id_to_path.insert(id.clone(), path.clone());
            entries.insert(id, value);
        }
    }

    fn load_file(path: &Path) -> Result<V, RegistryError> {
        let file_contents = fs::read_to_string(path).map_err(|error| RegistryError::ReadFile {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

        serde_json::from_str::<V>(&file_contents).map_err(|error| RegistryError::DeserializeJson {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
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

        let all_directories: Vec<&Path> = vec![
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

        let mut errors: Vec<RegistryError> = Vec::new();

        // Load scripts first (but do not fail early).
        let scripts_map = Self::load_scripts_from_directories(&all_directories, &mut errors);

        // Helper to load a registry and collect errors.
        fn load_registry<K, V>(
            directory: &Path,
            errors: &mut Vec<RegistryError>,
        ) -> Option<Registry<K, V>>
        where
            K: Eq + Hash + Clone + Debug,
            V: IdProvider<Id = K> + DeserializeOwned,
        {
            if !directory.exists() {
                // Decide policy: missing directory might be okay.
                // If not okay, push an error here.
                return Some(Registry {
                    entries: HashMap::new(),
                });
            }

            match Registry::<K, V>::load_from_directory(directory) {
                Ok(registry) => Some(registry),
                Err(error) => {
                    error.push_into(errors);
                    None
                }
            }
        }

        let actions = load_registry::<ActionId, Action>(&actions_directory, &mut errors);
        let backgrounds =
            load_registry::<BackgroundId, Background>(&backgrounds_directory, &mut errors);
        let classes = load_registry::<ClassId, Class>(&classes_directory, &mut errors);
        let effects = load_registry::<EffectId, Effect>(&effects_directory, &mut errors);
        let factions = load_registry::<FactionId, Faction>(&factions_directory, &mut errors);
        let feats = load_registry::<FeatId, Feat>(&feats_directory, &mut errors);
        let items = load_registry::<ItemId, ItemInstance>(&items_directory, &mut errors);
        let resources =
            load_registry::<ResourceId, ResourceDefinition>(&resources_directory, &mut errors);
        let species = load_registry::<SpeciesId, Species>(&species_directory, &mut errors);
        let spells = load_registry::<SpellId, Spell>(&spells_directory, &mut errors);
        let subclasses = load_registry::<SubclassId, Subclass>(&subclasses_directory, &mut errors);
        let subspecies =
            load_registry::<SubspeciesId, Subspecies>(&subspecies_directory, &mut errors);

        // If anything failed, report all collected diagnostics once.
        if !errors.is_empty() {
            return Err(RegistryError::Many(errors));
        }

        Ok(Self {
            actions: actions.expect("validated"),
            backgrounds: backgrounds.expect("validated"),
            classes: classes.expect("validated"),
            effects: effects.expect("validated"),
            factions: factions.expect("validated"),
            feats: feats.expect("validated"),
            items: items.expect("validated"),
            resources: resources.expect("validated"),
            scripts: Registry {
                entries: scripts_map,
            },
            species: species.expect("validated"),
            spells: spells.expect("validated"),
            subclasses: subclasses.expect("validated"),
            subspecies: subspecies.expect("validated"),
        })
    }

    // assuming Script has: id: ScriptId, and Script::try_from(entry) -> Result<Script, ScriptError>
    fn load_scripts_from_directories(
        directories: &[&Path],
        errors: &mut Vec<RegistryError>,
    ) -> HashMap<ScriptId, Script> {
        let mut scripts: HashMap<ScriptId, Script> = HashMap::new();
        let mut script_id_to_path: HashMap<ScriptId, PathBuf> = HashMap::new();

        for directory in directories {
            if !directory.exists() {
                continue;
            }

            let mut stack = vec![directory.to_path_buf()];
            while let Some(dir) = stack.pop() {
                let read_dir_iter = match fs::read_dir(&dir) {
                    Ok(iter) => iter,
                    Err(error) => {
                        errors.push(RegistryError::ReadDirectory {
                            directory: dir.clone(),
                            message: error.to_string(),
                        });
                        continue;
                    }
                };

                for entry_result in read_dir_iter {
                    let entry = match entry_result {
                        Ok(entry) => entry,
                        Err(error) => {
                            errors.push(RegistryError::ReadDirectoryEntry {
                                directory: dir.clone(),
                                message: error.to_string(),
                            });
                            continue;
                        }
                    };

                    let path = entry.path();

                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }

                    if path.extension().is_none()
                        || path.extension().and_then(|ext| ext.to_str()) == Some("json")
                    {
                        continue;
                    }

                    match Script::try_from(entry) {
                        Ok(script) => {
                            let id = script.id.clone();

                            if let Some(first_path) = script_id_to_path.get(&id).cloned() {
                                errors.push(RegistryError::DuplicateId {
                                    id_debug: format!("{:?}", id),
                                    first_path,
                                    second_path: path.clone(),
                                });
                                continue;
                            }

                            script_id_to_path.insert(id.clone(), path.clone());
                            scripts.insert(id, script);
                        }
                        Err(script_error) => {
                            // Map to a structured error; you can add a dedicated ScriptError variant if you prefer.
                            errors.push(RegistryError::ReadFile {
                                path: path.clone(),
                                message: format!("Failed to load script: {:?}", script_error),
                            });
                        }
                    }
                }
            }
        }

        scripts
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
