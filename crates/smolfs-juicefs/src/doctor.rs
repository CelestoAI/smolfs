use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use smolfs_core::{
    BinaryReport, Config, DoctorReport, FuseReport, Result, SmolFsError, SmolFsHome,
};

use crate::command::CommandSpec;

pub fn doctor(home: &SmolFsHome) -> Result<DoctorReport> {
    home.ensure_layout()?;
    let config = Config::load(home)?;
    let path = detect_juicefs_bin(home, &config);
    let version = path.as_ref().and_then(|path| juicefs_version(path).ok());
    let managed_path = home.managed_juicefs_bin();
    Ok(DoctorReport {
        home: home.root().to_path_buf(),
        config: home.config_path(),
        juicefs: BinaryReport {
            found: path.is_some(),
            managed: path.as_ref().is_some_and(|path| path == &managed_path),
            path,
            version,
        },
        fuse: detect_fuse(),
    })
}

pub fn install_managed_juicefs(home: &SmolFsHome) -> Result<PathBuf> {
    home.ensure_layout()?;
    let existing = which_on_path("juicefs").ok_or(SmolFsError::MissingJuiceFsBinary)?;
    let target = home.managed_juicefs_bin();
    fs::copy(&existing, &target).map_err(|source| SmolFsError::IoAt {
        path: target.clone(),
        source,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&target)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&target, permissions)?;
    }

    let config = Config {
        juicefs_bin: Some(target.clone()),
    };
    config.save(home)?;
    Ok(target)
}

pub fn detect_juicefs_bin(home: &SmolFsHome, config: &Config) -> Option<PathBuf> {
    if let Some(path) = env::var_os("SMOLFS_JUICEFS_BIN").map(PathBuf::from) {
        if is_executable_file(&path) {
            return Some(path);
        }
    }

    if let Some(path) = &config.juicefs_bin {
        if is_executable_file(path) {
            return Some(path.clone());
        }
    }

    let managed = home.managed_juicefs_bin();
    if is_executable_file(&managed) {
        return Some(managed);
    }

    which_on_path("juicefs")
}

pub fn require_juicefs_bin(home: &SmolFsHome, config: &Config) -> Result<PathBuf> {
    detect_juicefs_bin(home, config).ok_or(SmolFsError::MissingJuiceFsBinary)
}

pub fn require_fuse() -> Result<()> {
    let report = detect_fuse();
    if report.found {
        Ok(())
    } else {
        Err(SmolFsError::FuseUnavailable {
            detail: report.detail,
            fix: report
                .fix
                .unwrap_or_else(|| "Install FUSE support for this platform.".to_string()),
        })
    }
}

pub fn detect_fuse() -> FuseReport {
    match std::env::consts::OS {
        "macos" => detect_macos_fuse(),
        "linux" => detect_linux_fuse(),
        other => FuseReport {
            found: false,
            detail: format!("FUSE detection is not implemented for {other}"),
            fix: Some("Install FUSE support for your OS and retry.".to_string()),
        },
    }
}

fn juicefs_version(path: &Path) -> Result<String> {
    let output = CommandSpec::new(path).arg("version").run()?;
    Ok(output.stdout.trim().to_string())
}

fn detect_macos_fuse() -> FuseReport {
    let macfuse_root = Path::new("/Library/Filesystems/macfuse.fs");
    let macfuse_helper = macfuse_root
        .join("Contents")
        .join("Resources")
        .join("mount_macfuse");

    if macfuse_helper.exists()
        || macfuse_root.exists()
        || which_on_path("mount_macfuse").is_some()
        || which_on_path("mount_fusefs").is_some()
    {
        return FuseReport {
            found: true,
            detail: "macFUSE mount helper found".to_string(),
            fix: None,
        };
    }

    FuseReport {
        found: false,
        detail: "macFUSE mount helper was not found".to_string(),
        fix: Some("Install macFUSE, then rerun `smolfs doctor`.".to_string()),
    }
}

fn detect_linux_fuse() -> FuseReport {
    let has_dev_fuse = Path::new("/dev/fuse").exists();
    let has_helper =
        which_on_path("fusermount3").is_some() || which_on_path("fusermount").is_some();

    if has_dev_fuse && has_helper {
        return FuseReport {
            found: true,
            detail: "/dev/fuse and fusermount helper found".to_string(),
            fix: None,
        };
    }

    let detail = match (has_dev_fuse, has_helper) {
        (false, false) => "/dev/fuse and fusermount helper were not found",
        (false, true) => "/dev/fuse was not found",
        (true, false) => "fusermount helper was not found",
        (true, true) => unreachable!(),
    };

    FuseReport {
        found: false,
        detail: detail.to_string(),
        fix: Some("Install fuse3 and ensure the current user can access /dev/fuse.".to_string()),
    }
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}
