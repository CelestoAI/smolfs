use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Result, SmolFsError};

#[derive(Debug, Clone)]
pub struct SmolFsHome {
    root: PathBuf,
}

impl SmolFsHome {
    pub fn from_env() -> Result<Self> {
        if let Some(value) = env::var_os("SMOLFS_HOME") {
            return Ok(Self {
                root: PathBuf::from(value),
            });
        }

        let home = dirs::home_dir().ok_or_else(|| {
            SmolFsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not determine home directory",
            ))
        })?;

        Ok(Self {
            root: home.join(".smolfs"),
        })
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn volumes_dir(&self) -> PathBuf {
        self.root.join("volumes")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.root.join("bin")
    }

    pub fn managed_juicefs_bin(&self) -> PathBuf {
        self.bin_dir().join("juicefs")
    }

    pub fn dev_dir(&self) -> PathBuf {
        self.root.join("dev")
    }

    pub fn lock_path(&self) -> PathBuf {
        self.root.join("lock")
    }

    pub fn volume_path(&self, name: &str) -> PathBuf {
        self.volumes_dir().join(format!("{name}.toml"))
    }

    pub fn volume_log_path(&self, name: &str) -> PathBuf {
        self.logs_dir().join(format!("{name}.log"))
    }

    pub fn ensure_layout(&self) -> Result<()> {
        for path in [
            self.root(),
            &self.volumes_dir(),
            &self.logs_dir(),
            &self.bin_dir(),
            &self.dev_dir(),
        ] {
            std::fs::create_dir_all(path).map_err(|source| SmolFsError::IoAt {
                path: path.to_path_buf(),
                source,
            })?;
        }
        Ok(())
    }
}
