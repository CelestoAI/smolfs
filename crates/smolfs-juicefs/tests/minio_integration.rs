use std::env;

use smolfs_core::{Config, InitVolume, SmolFsHome};
use smolfs_juicefs::{SmolFs, install_managed_storage_backend};

struct MetadataBackend {
    name: &'static str,
    url: String,
}

fn integration_enabled() -> bool {
    env::var("SMOLFS_RUN_INTEGRATION").as_deref() == Ok("1")
}

#[test]
fn initializes_volume_against_minio_s3_bucket_with_metadata_backends() {
    if !integration_enabled() {
        eprintln!("skipping MinIO integration test; set SMOLFS_RUN_INTEGRATION=1");
        return;
    }

    let bucket = env::var("SMOLFS_TEST_S3_BUCKET")
        .unwrap_or_else(|_| "http://127.0.0.1:9000/smolfs-ci".to_string());

    let temp = tempfile::tempdir().expect("create temp SmolFS home");
    let home = SmolFsHome::new(temp.path());
    let metadata_backends = metadata_backends(temp.path());

    if let Ok(bin) = env::var("SMOLFS_STORAGE_BACKEND_BIN") {
        Config {
            storage_backend_bin: Some(bin.into()),
        }
        .save(&home)
        .expect("save storage backend path");
    } else {
        install_managed_storage_backend(&home).expect("install managed storage backend");
    }

    let fs = SmolFs::new(home).expect("construct SmolFS service");
    for metadata in metadata_backends {
        let volume_name = format!("minio-ci-{}-{}", metadata.name, std::process::id());

        let volume = fs
            .init(InitVolume {
                name: volume_name.clone(),
                dev: false,
                metadata_url: Some(metadata.url.clone()),
                store_url: None,
                storage: Some("s3".to_string()),
                bucket: Some(bucket.clone()),
            })
            .unwrap_or_else(|err| {
                panic!(
                    "initialize SmolFS volume backed by MinIO with {} metadata: {err}",
                    metadata.name
                )
            });

        assert_eq!(volume.name, volume_name);
        assert_eq!(volume.metadata_url, metadata.url);
        assert!(!volume.dev);
        assert_eq!(volume.storage, "s3");
        assert_eq!(volume.bucket, bucket);

        let status = fs
            .status(Some(&volume_name))
            .expect("read volume from registry");
        assert_eq!(status.volumes.len(), 1);
        assert_eq!(status.volumes[0].name, volume_name);
    }
}

fn metadata_backends(root: &std::path::Path) -> Vec<MetadataBackend> {
    let redis_url = env::var("SMOLFS_TEST_REDIS_METADATA_URL")
        .or_else(|_| env::var("SMOLFS_TEST_METADATA_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/1".to_string());
    let sqlite_path = root.join("metadata.sqlite3");
    let sqlite_url = env::var("SMOLFS_TEST_SQLITE_METADATA_URL")
        .unwrap_or_else(|_| format!("sqlite3://{}", sqlite_path.to_string_lossy()));

    vec![
        MetadataBackend {
            name: "redis",
            url: redis_url,
        },
        MetadataBackend {
            name: "sqlite",
            url: sqlite_url,
        },
    ]
}
