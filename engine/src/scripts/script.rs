use std::{
    fs::{self, DirEntry},
    str::FromStr,
};

use strum::EnumIter;

use crate::components::id::{IdProvider, ScriptId};

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
        let file_path = value.path();
        let file_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ScriptError::LoadError("Invalid file name".to_string()))?;
        let content = fs::read_to_string(&file_path).map_err(|e| {
            ScriptError::LoadError(format!("Failed to read script file {:?}: {}", file_path, e))
        })?;

        let language = ScriptLanguage::from_str(
            file_path
                .extension()
                .and_then(|s| s.to_str())
                .ok_or_else(|| ScriptError::LoadError("Missing file extension".to_string()))?,
        )?;

        let folder = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .ok_or_else(|| ScriptError::LoadError("Invalid parent folder".to_string()))?;
        // If folder is "spells" convert to singular "spell"
        let id = format!("script.{}.{}", folder.trim_end_matches('s'), file_name);
        let id = ScriptId::from_str(id);

        Ok(Script {
            id,
            file_path: file_path.to_string_lossy().to_string(),
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
