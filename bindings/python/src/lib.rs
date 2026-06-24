use std::path::PathBuf;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use smolfs_core::{InitVolume, MountVolume};
use smolfs_juicefs::SmolFs;

pyo3::create_exception!(smolfs, SmolFSError, PyException);

#[pyclass(name = "SmolFS")]
struct PySmolFs {
    inner: SmolFs,
}

#[pyclass]
#[derive(Clone)]
struct VolumeInfo {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    metadata_url: String,
    #[pyo3(get)]
    storage: String,
    #[pyo3(get)]
    bucket: String,
    #[pyo3(get)]
    dev: bool,
    #[pyo3(get)]
    mountpoint: Option<String>,
}

#[pyclass]
#[derive(Clone)]
struct MountInfo {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    mountpoint: String,
}

#[pyclass]
#[derive(Clone)]
struct Status {
    #[pyo3(get)]
    volumes: Vec<VolumeInfo>,
}

#[pymethods]
impl PySmolFs {
    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        Ok(Self {
            inner: SmolFs::from_env().map_err(to_py_err)?,
        })
    }

    fn doctor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let report = self.inner.doctor().map_err(to_py_err)?;
        let dict = PyDict::new(py);
        dict.set_item("home", report.home.display().to_string())?;
        dict.set_item("config", report.config.display().to_string())?;
        let juicefs = PyDict::new(py);
        juicefs.set_item("found", report.juicefs.found)?;
        juicefs.set_item(
            "path",
            report
                .juicefs
                .path
                .as_ref()
                .map(|path| path.display().to_string()),
        )?;
        juicefs.set_item("version", report.juicefs.version)?;
        juicefs.set_item("managed", report.juicefs.managed)?;
        dict.set_item("juicefs", juicefs)?;
        let fuse = PyDict::new(py);
        fuse.set_item("found", report.fuse.found)?;
        fuse.set_item("detail", report.fuse.detail)?;
        fuse.set_item("fix", report.fuse.fix)?;
        dict.set_item("fuse", fuse)?;
        Ok(dict)
    }

    #[pyo3(signature = (name, *, dev=false, metadata=None, store=None, storage=None, bucket=None))]
    fn init(
        &self,
        name: String,
        dev: bool,
        metadata: Option<String>,
        store: Option<String>,
        storage: Option<String>,
        bucket: Option<String>,
    ) -> PyResult<VolumeInfo> {
        self.inner
            .init(InitVolume {
                name,
                dev,
                metadata_url: metadata,
                store_url: store,
                storage,
                bucket,
            })
            .map(VolumeInfo::from)
            .map_err(to_py_err)
    }

    #[pyo3(signature = (name, *, dev=false, metadata=None, store=None, storage=None, bucket=None))]
    fn ensure_volume(
        &self,
        name: String,
        dev: bool,
        metadata: Option<String>,
        store: Option<String>,
        storage: Option<String>,
        bucket: Option<String>,
    ) -> PyResult<VolumeInfo> {
        self.inner
            .ensure_volume(InitVolume {
                name,
                dev,
                metadata_url: metadata,
                store_url: store,
                storage,
                bucket,
            })
            .map(VolumeInfo::from)
            .map_err(to_py_err)
    }

    #[pyo3(signature = (name, path, *, foreground=false, check_storage=false))]
    fn mount(
        &self,
        name: String,
        path: String,
        foreground: bool,
        check_storage: bool,
    ) -> PyResult<MountInfo> {
        self.inner
            .mount(MountVolume {
                name,
                path: PathBuf::from(path),
                foreground,
                check_storage,
            })
            .map(MountInfo::from)
            .map_err(to_py_err)
    }

    fn flush(&self, name: String) -> PyResult<()> {
        self.inner.flush(&name).map_err(to_py_err)
    }

    #[pyo3(signature = (name, *, force=false))]
    fn unmount(&self, name: String, force: bool) -> PyResult<()> {
        self.inner.unmount(&name, force).map_err(to_py_err)
    }

    #[pyo3(signature = (name=None))]
    fn status(&self, name: Option<String>) -> PyResult<Status> {
        self.inner
            .status(name.as_deref())
            .map(|status| Status {
                volumes: status.volumes.into_iter().map(VolumeInfo::from).collect(),
            })
            .map_err(to_py_err)
    }
}

impl From<smolfs_core::VolumeInfo> for VolumeInfo {
    fn from(value: smolfs_core::VolumeInfo) -> Self {
        Self {
            name: value.name,
            metadata_url: value.metadata_url,
            storage: value.storage,
            bucket: value.bucket,
            dev: value.dev,
            mountpoint: value.mountpoint.map(|path| path.display().to_string()),
        }
    }
}

impl From<smolfs_core::MountInfo> for MountInfo {
    fn from(value: smolfs_core::MountInfo) -> Self {
        Self {
            name: value.name,
            mountpoint: value.mountpoint.display().to_string(),
        }
    }
}

fn to_py_err(err: smolfs_core::SmolFsError) -> PyErr {
    SmolFSError::new_err(err.to_string())
}

#[pymodule]
fn _native(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("SmolFSError", py.get_type::<SmolFSError>())?;
    module.add_class::<PySmolFs>()?;
    module.add_class::<VolumeInfo>()?;
    module.add_class::<MountInfo>()?;
    module.add_class::<Status>()?;
    Ok(())
}
