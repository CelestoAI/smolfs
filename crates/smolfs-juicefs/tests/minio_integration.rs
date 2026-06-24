use std::env;

use smolfs_core::{Config, InitVolume, SmolFsHome};
use smolfs_juicefs::{SmolFs, install_managed_juicefs};

fn integration_enabled() -> bool {
    env::var("SMOLFS_RUN_INTEGRATION").as_deref() == Ok("1")
}

#[test]
fn initializes_volume_against_minio_s3_bucket() {
    if !integration_enabled() {
        eprintln!("skipping MinIO integration test; set SMOLFS_RUN_INTEGRATION=1");
        return;
    }

    let metadata_url = env::var("SMOLFS_TEST_METADATA_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/1".to_string());
    let bucket = env::var("SMOLFS_TEST_S3_BUCKET")
        .unwrap_or_else(|_| "http://127.0.0.1:9000/smolfs-ci".to_string());

    let temp = tempfile::tempdir().expect("create temp SmolFS home");
    let home = SmolFsHome::new(temp.path());

    if let Ok(bin) = env::var("SMOLFS_JUICEFS_BIN") {
        Config {
            juicefs_bin: Some(bin.into()),
        }
        .save(&home)
        .expect("save JuiceFS path");
    } else {
        install_managed_juicefs(&home).expect("install managed JuiceFS from PATH");
    }

    let fs = SmolFs::new(home).expect("construct SmolFS service");
    let volume_name = format!("minio-ci-{}", std::process::id());

    let volume = fs
        .init(InitVolume {
            name: volume_name.clone(),
            dev: false,
            metadata_url: Some(metadata_url),
            store_url: None,
            storage: Some("s3".to_string()),
            bucket: Some(bucket.clone()),
        })
        .expect("initialize JuiceFS volume backed by MinIO");

    assert_eq!(volume.name, volume_name);
    assert!(!volume.dev);
    assert_eq!(volume.storage, "s3");
    assert_eq!(volume.bucket, bucket);

    let status = fs
        .status(Some(&volume_name))
        .expect("read volume from registry");
    assert_eq!(status.volumes.len(), 1);
    assert_eq!(status.volumes[0].name, volume_name);
}
