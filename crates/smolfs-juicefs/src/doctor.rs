use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use smolfs_core::{
    BinaryReport, Config, DoctorReport, MountSupportReport, Result, SmolFsError, SmolFsHome,
};

use crate::command::CommandSpec;

const STORAGE_BACKEND_VERSION: &str = "1.3.1";
const STORAGE_BACKEND_BASE_URL: &str = "https://d.juicefs.com/juicefs/releases/download";

pub fn doctor(home: &SmolFsHome) -> Result<DoctorReport> {
    home.ensure_layout()?;
    let config = Config::load(home)?;
    let path = detect_storage_backend_bin(home, &config);
    let version = path
        .as_ref()
        .and_then(|path| storage_backend_version(path).ok());
    let managed_path = home.managed_storage_backend_bin();
    Ok(DoctorReport {
        home: home.root().to_path_buf(),
        config: home.config_path(),
        storage_backend: BinaryReport {
            found: path.is_some(),
            managed: path.as_ref().is_some_and(|path| path == &managed_path),
            path,
            version,
        },
        mount_support: detect_mount_support(),
    })
}

pub fn install_managed_storage_backend(home: &SmolFsHome) -> Result<PathBuf> {
    home.ensure_layout()?;
    let target = home.managed_storage_backend_bin();

    if is_executable_file(&target) {
        save_storage_backend_config(home, &target)?;
        return Ok(target);
    }

    if let Some(existing) = which_on_path("juicefs") {
        install_existing_storage_backend(home, &existing, &target)?;
        return Ok(target);
    }

    download_storage_backend(home, &target)?;
    Ok(target)
}

pub fn install_managed_juicefs(home: &SmolFsHome) -> Result<PathBuf> {
    install_managed_storage_backend(home)
}

fn install_existing_storage_backend(
    home: &SmolFsHome,
    existing: &Path,
    target: &Path,
) -> Result<()> {
    fs::copy(existing, target).map_err(|source| SmolFsError::IoAt {
        path: target.to_path_buf(),
        source,
    })?;
    make_executable(target)?;
    save_storage_backend_config(home, target)
}

fn download_storage_backend(home: &SmolFsHome, target: &Path) -> Result<()> {
    let (os, arch) = storage_backend_platform()?;
    let version = env::var("SMOLFS_STORAGE_BACKEND_VERSION")
        .ok()
        .map(|value| value.trim_start_matches('v').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| STORAGE_BACKEND_VERSION.to_string());
    let base_url = env::var("SMOLFS_STORAGE_BACKEND_BASE_URL")
        .unwrap_or_else(|_| STORAGE_BACKEND_BASE_URL.to_string());
    let file_name = format!("juicefs-{version}-{os}-{arch}.tar.gz");
    let release_url = format!("{base_url}/v{version}/{file_name}");
    let checksums_url = format!("{base_url}/v{version}/checksums.txt");
    let tmp = temp_install_dir()?;
    let archive = tmp.join(&file_name);
    let checksums = tmp.join("checksums.txt");

    let result = (|| {
        download_file(&release_url, &archive)?;
        download_file(&checksums_url, &checksums)?;
        verify_checksum(&archive, &checksums, &file_name)?;
        run_command(
            Command::new("tar")
                .arg("-xzf")
                .arg(&archive)
                .arg("-C")
                .arg(&tmp),
            "extract storage backend archive",
        )?;
        let extracted = tmp.join("juicefs");
        fs::copy(&extracted, target).map_err(|source| SmolFsError::IoAt {
            path: target.to_path_buf(),
            source,
        })?;
        make_executable(target)?;
        save_storage_backend_config(home, target)
    })();

    let _ = fs::remove_dir_all(&tmp);
    result
}

fn save_storage_backend_config(home: &SmolFsHome, target: &Path) -> Result<()> {
    let config = Config {
        storage_backend_bin: Some(target.to_path_buf()),
    };
    config.save(home)
}

fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn temp_install_dir() -> Result<PathBuf> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = env::temp_dir().join(format!(
        "smolfs-storage-install-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&path).map_err(|source| SmolFsError::IoAt {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

fn storage_backend_platform() -> Result<(&'static str, &'static str)> {
    let os = match env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        other => {
            return Err(SmolFsError::UnsupportedStorageBackendPlatform {
                platform: other.to_string(),
            });
        }
    };
    let arch = match env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => {
            return Err(SmolFsError::UnsupportedStorageBackendPlatform {
                platform: format!("{os}/{other}"),
            });
        }
    };
    Ok((os, arch))
}

fn download_file(url: &str, path: &Path) -> Result<()> {
    if let Some(curl) = which_on_path("curl") {
        return run_command(
            Command::new(curl).arg("-fsSL").arg(url).arg("-o").arg(path),
            "download storage backend",
        );
    }
    if let Some(wget) = which_on_path("wget") {
        return run_command(
            Command::new(wget).arg("-q").arg("-O").arg(path).arg(url),
            "download storage backend",
        );
    }
    Err(SmolFsError::StorageBackendInstallFailed {
        reason: "curl or wget is required".to_string(),
    })
}

fn verify_checksum(archive: &Path, checksums: &Path, file_name: &str) -> Result<()> {
    let checksums_text = fs::read_to_string(checksums).map_err(|source| SmolFsError::IoAt {
        path: checksums.to_path_buf(),
        source,
    })?;
    let expected = checksums_text
        .lines()
        .find(|line| {
            line.split_whitespace()
                .any(|part| part.ends_with(file_name))
        })
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| SmolFsError::StorageBackendInstallFailed {
            reason: "downloaded checksum file did not include the storage backend archive"
                .to_string(),
        })?;

    let output = if let Some(shasum) = which_on_path("shasum") {
        Command::new(shasum)
            .arg("-a")
            .arg("256")
            .arg(archive)
            .output()?
    } else if let Some(sha256sum) = which_on_path("sha256sum") {
        Command::new(sha256sum).arg(archive).output()?
    } else {
        return Err(SmolFsError::StorageBackendInstallFailed {
            reason: "shasum or sha256sum is required to verify the download".to_string(),
        });
    };

    if !output.status.success() {
        return Err(SmolFsError::StorageBackendInstallFailed {
            reason: "checksum verification command failed".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = stdout.split_whitespace().next().unwrap_or_default();
    if actual != expected {
        return Err(SmolFsError::StorageBackendInstallFailed {
            reason: "downloaded storage backend checksum did not match".to_string(),
        });
    }
    Ok(())
}

fn run_command(command: &mut Command, action: &str) -> Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reason = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(action)
        .to_string();
    Err(SmolFsError::StorageBackendInstallFailed { reason })
}

pub fn detect_storage_backend_bin(home: &SmolFsHome, config: &Config) -> Option<PathBuf> {
    if let Some(path) = env::var_os("SMOLFS_STORAGE_BACKEND_BIN").map(PathBuf::from) {
        if is_executable_file(&path) {
            return Some(path);
        }
    }

    if let Some(path) = env::var_os("SMOLFS_JUICEFS_BIN").map(PathBuf::from) {
        if is_executable_file(&path) {
            return Some(path);
        }
    }

    if let Some(path) = &config.storage_backend_bin {
        if is_executable_file(path) {
            return Some(path.clone());
        }
    }

    let managed = home.managed_storage_backend_bin();
    if is_executable_file(&managed) {
        return Some(managed);
    }

    let legacy_managed = home.legacy_managed_juicefs_bin();
    if is_executable_file(&legacy_managed) {
        return Some(legacy_managed);
    }

    which_on_path("juicefs")
}

pub fn require_storage_backend_bin(home: &SmolFsHome, config: &Config) -> Result<PathBuf> {
    detect_storage_backend_bin(home, config).ok_or(SmolFsError::MissingStorageBackend)
}

pub fn require_mount_support() -> Result<()> {
    let report = detect_mount_support();
    if report.found {
        Ok(())
    } else {
        Err(SmolFsError::MountSupportUnavailable {
            detail: report.detail,
            fix: report
                .fix
                .unwrap_or_else(|| "Install local mount support for this platform.".to_string()),
        })
    }
}

pub fn detect_mount_support() -> MountSupportReport {
    match std::env::consts::OS {
        "macos" => detect_macos_mount_support(),
        "linux" => detect_linux_mount_support(),
        other => MountSupportReport {
            found: false,
            detail: format!("mount support detection is not implemented for {other}"),
            fix: Some("Install local mount support for your OS and retry.".to_string()),
        },
    }
}

fn storage_backend_version(path: &Path) -> Result<String> {
    let output = CommandSpec::new(path).arg("version").run()?;
    Ok(extract_version(output.stdout.trim()))
}

fn extract_version(raw: &str) -> String {
    raw.split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find(|part| part.contains('.') && part.chars().any(|ch| ch.is_ascii_digit()))
        .unwrap_or(raw)
        .to_string()
}

fn detect_macos_mount_support() -> MountSupportReport {
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
        return MountSupportReport {
            found: true,
            detail: "macOS mount helper found".to_string(),
            fix: None,
        };
    }

    MountSupportReport {
        found: false,
        detail: "macOS mount helper was not found".to_string(),
        fix: Some(
            "Install macOS FUSE-compatible mount support, then rerun `smolfs doctor`.".to_string(),
        ),
    }
}

fn detect_linux_mount_support() -> MountSupportReport {
    let has_dev_fuse = Path::new("/dev/fuse").exists();
    let has_helper =
        which_on_path("fusermount3").is_some() || which_on_path("fusermount").is_some();

    if has_dev_fuse && has_helper {
        return MountSupportReport {
            found: true,
            detail: "Linux mount device and helper found".to_string(),
            fix: None,
        };
    }

    let detail = match (has_dev_fuse, has_helper) {
        (false, false) => "Linux mount device and helper were not found",
        (false, true) => "Linux mount device was not found",
        (true, false) => "Linux mount helper was not found",
        (true, true) => unreachable!(),
    };

    MountSupportReport {
        found: false,
        detail: detail.to_string(),
        fix: Some(
            "Install fuse3 and ensure the current user can access the Linux mount device."
                .to_string(),
        ),
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
