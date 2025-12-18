use std::{
    fs::{self, DirEntry},
    str::FromStr,
};

use strum::EnumIter;

use crate::{
    components::id::{IdProvider, ScriptId},
    registry::registry::REGISTRIES_FOLDER,
};

#[derive(Debug)]
pub enum ScriptError {
    LoadError(String),
    RuntimeError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum ScriptLanguage {
    // Lua,
    Rhai,
}

impl ScriptLanguage {
    pub fn file_extension(&self) -> &str {
        match self {
            // ScriptLanguage::Lua => "lua",
            ScriptLanguage::Rhai => "rhai",
        }
    }
}

impl FromStr for ScriptLanguage {
    type Err = ScriptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // "lua" => Ok(ScriptLanguage::Lua),
            "rhai" => Ok(ScriptLanguage::Rhai),
            _ => Err(ScriptError::LoadError(format!(
                "Unknown script language: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    pub id: ScriptId,
    pub file_path: String,
    pub content: String,
    pub language: ScriptLanguage,
}

impl TryFrom<DirEntry> for Script {
    type Error = ScriptError;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let full_file_path = value.path();
        let file_name = full_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ScriptError::LoadError("Invalid file name".to_string()))?;
        let content = fs::read_to_string(&full_file_path).map_err(|e| {
            ScriptError::LoadError(format!(
                "Failed to read script file {:?}: {}",
                full_file_path, e
            ))
        })?;

        let language = ScriptLanguage::from_str(
            full_file_path
                .extension()
                .and_then(|s| s.to_str())
                .ok_or_else(|| ScriptError::LoadError("Missing file extension".to_string()))?,
        )?;

        // Keep visiting parent folders until we reach the registry root
        let mut script_id = file_name.to_string();
        let mut file_path = full_file_path.clone();
        while let Some(parent) = file_path.parent() {
            if let Some(folder_name) = parent.file_name().and_then(|s| s.to_str()) {
                if folder_name == REGISTRIES_FOLDER {
                    break;
                }
                // Convert plural folder names to singular for script IDs
                let folder_name = folder_name.trim_end_matches('s');
                script_id = format!("{}.{}", folder_name, script_id);
            }
            file_path = parent.to_path_buf();
        }
        let id = ScriptId::new("nat20_rs", format!("script.{}", script_id));

        Ok(Script {
            id,
            file_path: full_file_path.to_string_lossy().to_string(),
            content,
            language,
        })
    }
}

impl IdProvider for Script {
    type Id = ScriptId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
