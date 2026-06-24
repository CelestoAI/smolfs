use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SmolFsError};
use crate::paths::SmolFsHome;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub juicefs_bin: Option<PathBuf>,
}

impl Config {
    pub fn load(home: &SmolFsHome) -> Result<Self> {
        let path = home.config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|source| SmolFsError::IoAt { path, source })?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self, home: &SmolFsHome) -> Result<()> {
        home.ensure_layout()?;
        let path = home.config_path();
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents).map_err(|source| SmolFsError::IoAt { path, source })
    }
}
