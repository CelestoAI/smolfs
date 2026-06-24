use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SmolFsError {
    #[error("invalid volume name {name:?}; use only letters, numbers, '.', '_' and '-'")]
    InvalidVolumeName { name: String },

    #[error("volume {name:?} already exists")]
    VolumeExists { name: String },

    #[error("volume {name:?} does not exist")]
    VolumeNotFound { name: String },

    #[error(
        "missing JuiceFS binary; run `smolfs doctor --install`, set SMOLFS_JUICEFS_BIN, or install `juicefs` on PATH"
    )]
    MissingJuiceFsBinary,

    #[error(
        "unsupported store URL {store:?}; use file://, s3://, gs://, or pass --storage and --bucket"
    )]
    UnsupportedStoreUrl { store: String },

    #[error("missing metadata URL; pass --metadata or use --dev")]
    MissingMetadataUrl,

    #[error("missing object storage config; pass --store or --storage and --bucket")]
    MissingStore,

    #[error("mountpoint {path} exists and is not a directory")]
    MountpointNotDirectory { path: PathBuf },

    #[error("volume {name:?} is not mounted")]
    VolumeNotMounted { name: String },

    #[error("FUSE is unavailable: {detail}\nFix: {fix}")]
    FuseUnavailable { detail: String, fix: String },

    #[error("command failed: {program} {args}\nexit: {status}\nstdout: {stdout}\nstderr: {stderr}")]
    CommandFailed {
        program: String,
        args: String,
        status: String,
        stdout: String,
        stderr: String,
    },

    #[error("I/O error at {path}: {source}")]
    IoAt {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, SmolFsError>;
