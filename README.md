# SmolFS

SmolFS is a small developer-facing volume tool for agent workspaces. It wraps
JuiceFS with a Rust core, a polished CLI, and language bindings so agents can
work against a durable local-looking directory without each runtime learning
JuiceFS lifecycle details.

## Quick Start

SmolFS needs JuiceFS plus FUSE support on the machine that mounts volumes.
Run `doctor` first; it reports the exact missing dependency and the next fix.

```bash
curl -fsSL https://raw.githubusercontent.com/CelestoAI/smolfs/main/scripts/install.sh | sh

smolfs doctor
smolfs init demo --dev
smolfs mount demo ./workspace
echo hello > ./workspace/hello.txt
smolfs unmount demo
smolfs mount demo ./workspace
cat ./workspace/hello.txt
```

`--dev` uses JuiceFS with local SQLite metadata and local file storage under
`~/.smolfs/dev`.

To install both the CLI and the Python SDK into the current Python project or
active virtualenv:

```bash
curl -fsSL https://raw.githubusercontent.com/CelestoAI/smolfs/main/scripts/install.sh | SMOLFS_INSTALL_PYTHON=1 sh
```

For CLI development from this checkout:

```bash
cargo build -p smolfs-cli
./target/debug/smolfs doctor
```

## Python SDK

The Python package is SDK-only. Install it with `uv`:

```bash
uv add smolfs
```

The shell installer can also add the SDK for you when `SMOLFS_INSTALL_PYTHON=1`
is set. It runs `uv add smolfs` from a directory with `pyproject.toml`, or
`uv pip install smolfs` inside an active virtualenv. Set
`SMOLFS_PYTHON_MODE=user` to use `uv pip install --user smolfs`.

For local development from this checkout:

```bash
uv run --isolated --with-editable ./bindings/python python -c "from smolfs import doctor; print(doctor())"
```

Use the SDK from any Python agent runner:

```python
from pathlib import Path

from smolfs import SmolFS, doctor

report = doctor()
if not report["juicefs"]["found"] or not report["fuse"]["found"]:
    raise RuntimeError(f"SmolFS is not ready: {report}")

fs = SmolFS.from_env()
volume = fs.ensure_volume("demo", dev=True)
mount = fs.mount(volume.name, "./workspace")

workspace = Path(mount.mountpoint)
(workspace / "hello.txt").write_text("hello from SmolFS\n")

try:
    fs.flush(volume.name)
finally:
    fs.unmount(volume.name)
```

Cloud volumes use the same API with explicit metadata and object storage:

```python
fs.ensure_volume(
    "agent-workspace",
    metadata="redis://localhost:6379/1",
    storage="s3",
    bucket="https://my-bucket.s3.us-east-2.amazonaws.com",
)
```

For S3-compatible services such as MinIO, pass the service bucket URL and provide
`ACCESS_KEY` and `SECRET_KEY` in the environment used by JuiceFS.

## Publishing the Python Package

Python packaging uses `uv` and `maturin`.

The GitHub workflow at `.github/workflows/publish-python.yml` builds wheels for
Linux and macOS, builds an sdist, and publishes to PyPI when a `v*` tag is
pushed.

Before the first release, configure PyPI Trusted Publishing:

1. Create or claim the `smolfs` project on PyPI.
2. Add a trusted publisher for repository `CelestoAI/smolfs`.
3. Set the workflow name to `publish-python.yml`.
4. Set the environment name to `pypi`.

Release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Publishing the CLI

The `smolfs` command is built from the Rust CLI crate. The GitHub workflow at
`.github/workflows/publish-cli.yml` builds Linux and macOS release binaries,
smoke-tests `smolfs --help`, and attaches tarballs to `v*` GitHub releases.

## Next Steps

- Replace the current managed JuiceFS copy flow with real cross-platform
  JuiceFS downloads in `smolfs doctor --install`.
- Add a Node.js SDK using the same Rust core through `napi-rs`.
- Add type stubs for the Python package.
- Add a Linux CI job that mounts a local dev volume when FUSE is available.
- Add release notes and a changelog before the first non-draft release.
