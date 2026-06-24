use std::path::PathBuf;

use crate::error::{Result, SmolFsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreSpec {
    pub storage: String,
    pub bucket: String,
}

impl StoreSpec {
    pub fn new(storage: impl Into<String>, bucket: impl Into<String>) -> Self {
        Self {
            storage: storage.into(),
            bucket: bucket.into(),
        }
    }
}

pub fn parse_store_url(store: &str) -> Result<StoreSpec> {
    if let Some(path) = store.strip_prefix("file://") {
        return Ok(StoreSpec::new("file", path));
    }
    if store.starts_with("s3://") {
        return Ok(StoreSpec::new("s3", store));
    }
    if store.starts_with("gs://") {
        return Ok(StoreSpec::new("gs", store));
    }

    Err(SmolFsError::UnsupportedStoreUrl {
        store: store.to_string(),
    })
}

pub fn dev_store(root: PathBuf) -> StoreSpec {
    StoreSpec::new("file", root.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::parse_store_url;

    #[test]
    fn parses_known_store_urls() {
        assert_eq!(
            parse_store_url("file:///tmp/objects").unwrap().storage,
            "file"
        );
        assert_eq!(parse_store_url("s3://bucket/prefix").unwrap().storage, "s3");
        assert_eq!(parse_store_url("gs://bucket/prefix").unwrap().storage, "gs");
    }
}
