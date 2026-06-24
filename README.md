<div align="center">

# SmolFS

#### Durable workspace folders for AI agents.

<img src="https://ik.imagekit.io/gradsflow/celestoai/logo/celesto%20cover%20low_vFigbRaJI.png">

[![CI](https://github.com/CelestoAI/smolfs/actions/workflows/ci.yml/badge.svg)](https://github.com/CelestoAI/smolfs/actions/workflows/ci.yml)
[![Publish CLI](https://github.com/CelestoAI/smolfs/actions/workflows/publish-cli.yml/badge.svg)](https://github.com/CelestoAI/smolfs/actions/workflows/publish-cli.yml)
[![Publish Python Package](https://github.com/CelestoAI/smolfs/actions/workflows/publish-python.yml/badge.svg)](https://github.com/CelestoAI/smolfs/actions/workflows/publish-python.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-orange.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![Python 3.9+](https://img.shields.io/badge/python-3.9+-orange.svg)](https://www.python.org/downloads/)

[Quickstart](#quickstart) | [Lifecycle](#cli-lifecycle) | [Python SDK](#python-sdk) | [TypeScript SDK](#typescript-sdk) | [Development](#development) | [Releases](#releases)

</div>

---

SmolFS gives agents a workspace folder that can survive after the agent process
stops. You mount it like a normal directory, write files into it, unmount it
when the job is done, and mount it again later from another process.

SmolFS uses JuiceFS for the filesystem work. SmolFS owns the simpler developer
experience around creating volumes, checking the machine, mounting, flushing,
unmounting, and inspecting status.

<br>

<table>
<tr>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/folder-sync.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>Durable workspaces</strong></p>
<p>Agents can keep files across short-lived runtimes without each runtime managing JuiceFS directly.</p>
<p><a href="#cli-lifecycle">Read more -&gt;</a></p>
</td>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/hard-drive.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>Local dev mode</strong></p>
<p>Use <code>--dev</code> for a local-only volume backed by SQLite metadata and local object files.</p>
<p><a href="#create-a-local-volume">Read more -&gt;</a></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/cloud.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>Cloud volumes</strong></p>
<p>Use Redis plus S3-compatible object storage when the same workspace needs to outlive one machine.</p>
<p><a href="#create-a-cloud-volume">Read more -&gt;</a></p>
</td>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/terminal.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>One CLI lifecycle</strong></p>
<p>Run <code>doctor</code>, <code>init</code>, <code>mount</code>, <code>flush</code>, <code>status</code>, and <code>unmount</code> from one command.</p>
<p><a href="#commands">Read more -&gt;</a></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/code-2.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>Thin SDKs</strong></p>
<p>Python and TypeScript bindings call the same Rust core so agent tools can use SmolFS without shelling out.</p>
<p><a href="#python-sdk">Read more -&gt;</a></p>
</td>
<td width="50%" valign="top">
<p><img src="https://api.iconify.design/lucide/shield-check.svg?color=%236e7681" width="24" height="24" align="absmiddle" alt=""> <strong>Explicit configuration</strong></p>
<p>Cloud metadata, buckets, and credentials stay explicit so durable agent data is easier to audit.</p>
<p><a href="#security-and-reliability">Read more -&gt;</a></p>
</td>
</tr>
</table>

## Use Cases

- **Keep agent work across turns.** Mount the same workspace again after an
  agent process exits.
- **Share a workspace across runtimes.** Put metadata in Redis and file contents
  in S3-compatible storage.
- **Test locally before using cloud storage.** Start with `--dev`, then switch
  to explicit metadata and object storage settings.
- **Wrap storage in agent tooling.** Use the Python or TypeScript SDK from an
  agent runner instead of teaching every agent process about JuiceFS.

## Quickstart

Install the SmolFS CLI:

```bash
curl -fsSL https://raw.githubusercontent.com/CelestoAI/smolfs/main/scripts/install.sh | sh
```

The installer downloads the latest CLI release asset for your platform. If no
release asset exists yet, use the source checkout flow in [Development](#development).

Check the machine and try a local volume:

```bash
smolfs doctor
smolfs init demo --dev
smolfs mount demo ./workspace
echo hello > ./workspace/hello.txt
smolfs flush demo
smolfs unmount demo
smolfs mount demo ./workspace
cat ./workspace/hello.txt
```

SmolFS needs JuiceFS plus FUSE support on the machine that mounts volumes. FUSE
is operating system support that lets an app provide a folder your tools can
read and write.

<details>
<summary>Install the Python SDK with the CLI</summary>

```bash
curl -fsSL https://raw.githubusercontent.com/CelestoAI/smolfs/main/scripts/install.sh | SMOLFS_INSTALL_PYTHON=1 sh
```

The installer runs `uv add smolfs` from a directory with `pyproject.toml`, or
`uv pip install smolfs` inside an active virtualenv. Set
`SMOLFS_PYTHON_MODE=user` to use `uv pip install --user smolfs`.

</details>

## CLI Lifecycle

The normal SmolFS flow is: check the machine, create a volume, mount it, use the
directory, flush important writes, unmount it, and mount it again when you need
the same files later.

### Check the Machine

Run `doctor` before creating or mounting volumes:

```bash
smolfs doctor
```

`doctor` checks whether SmolFS can find JuiceFS and whether the machine can use
FUSE.

Useful options:

- `smolfs doctor --install` copies a discovered JuiceFS binary into SmolFS'
  managed bin directory.
- `smolfs doctor --json` prints the same report as JSON for scripts.

SmolFS looks for its home directory in `SMOLFS_HOME`. If it is not set, SmolFS
uses `~/.smolfs`. The home directory stores SmolFS config, volume records, logs,
managed binaries, and local dev-volume data.

### Create a Local Volume

A volume is the named workspace that SmolFS can mount later.

```bash
smolfs init demo --dev
```

`--dev` creates a local-only volume. It uses SQLite for metadata and local files
for object data under the SmolFS home directory. This is the simplest path for
trying SmolFS on one machine.

### Create a Cloud Volume

Cloud volumes need explicit metadata and object storage settings. Metadata is
where JuiceFS stores the file tree. Object storage is where file contents live.

```bash
smolfs init agent-workspace \
  --metadata redis://localhost:6379/1 \
  --storage s3 \
  --bucket https://my-bucket.s3.us-east-2.amazonaws.com
```

You can pass object storage in either of these forms:

- `--store s3://bucket/prefix`, `--store gs://bucket/prefix`, or
  `--store file:///path/to/objects`.
- `--storage TYPE --bucket BUCKET`, which is useful for S3-compatible services
  that expect an endpoint-style bucket URL.

For Cloudflare R2 or another S3-compatible service, keep credentials in the
environment used by JuiceFS. Do not put access keys in command arguments or logs.

```bash
set -a
. ./.env
set +a

export SMOLFS_HOME=/tmp/smolfs-r2-home
VOL="r2demo-$(date +%Y%m%d%H%M%S)"

smolfs init "$VOL" \
  --metadata "$SMOLFS_R2_METADATA" \
  --storage s3 \
  --bucket "$SMOLFS_R2_BUCKET_URL"
```

### Mount and Use the Volume

Mounting makes the volume appear as a normal local directory:

```bash
smolfs mount demo ./workspace
echo hello > ./workspace/hello.txt
cat ./workspace/hello.txt
```

SmolFS creates the mount directory if it does not exist. After the mount
succeeds, programs can read and write files through that directory.

Useful options:

- `--check-storage` asks JuiceFS to test object storage access before the mount
  completes.
- `--foreground` runs JuiceFS in the foreground instead of starting a background
  mount process.

### Flush, Inspect, and Unmount

Run `flush` when you want a best-effort check that recent writes have reached
the mounted filesystem:

```bash
smolfs flush demo
```

Run `status` to see known volumes:

```bash
smolfs status
smolfs status demo
smolfs status --json
```

Unmount when the job is done:

```bash
smolfs unmount demo
```

`unmount` asks JuiceFS to flush before detaching the mountpoint. Use
`smolfs umount demo` if you prefer the shorter alias. Add `--force` when the
mountpoint is busy and you want JuiceFS to force the unmount.

After unmounting, you can mount the same volume again and read the files:

```bash
smolfs mount demo ./workspace
cat ./workspace/hello.txt
```

### Commands

| Command | What it does |
| --- | --- |
| `smolfs doctor` | Checks JuiceFS, FUSE, and local SmolFS setup. |
| `smolfs init NAME --dev` | Creates a local development volume. |
| `smolfs init NAME --metadata URL --storage TYPE --bucket BUCKET` | Creates a cloud volume with explicit metadata and object storage. |
| `smolfs mount NAME PATH` | Mounts a volume at a local directory. |
| `smolfs flush NAME` | Probes the mounted volume and syncs a small file through it. |
| `smolfs status [NAME]` | Shows known volumes and current mountpoints. |
| `smolfs unmount NAME` | Unmounts a mounted volume and asks JuiceFS to flush. |
| `smolfs umount NAME` | Alias for `smolfs unmount NAME`. |

Every command has its own help page:

```bash
smolfs help
smolfs init --help
```

## Python SDK

The Python package is SDK-only. Install it with `uv`:

```bash
uv add smolfs
```

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

For S3-compatible services such as MinIO or Cloudflare R2, pass the service
bucket URL and provide `ACCESS_KEY` and `SECRET_KEY` in the environment used by
JuiceFS.

## TypeScript SDK

The TypeScript package is a native Node.js binding over the same Rust core. The
npm package is not published yet; for now, use the local checkout flow below.

For local development from this checkout, use Node.js 18 or newer:

```bash
cd bindings/node
npm ci
npm run build:debug
npm test
```

Use the SDK from a Node.js agent runner:

```ts
import { SmolFS, doctor } from "@celestoai/smolfs";
import { writeFile } from "node:fs/promises";
import { join } from "node:path";

const report = doctor();
if (!report.juicefs.found || !report.fuse.found) {
  throw new Error(`SmolFS is not ready: ${JSON.stringify(report)}`);
}

const fs = SmolFS.fromEnv();
const volume = fs.ensureVolume({ name: "demo", dev: true });
const mount = fs.mount({ name: volume.name, path: "./workspace" });

try {
  await writeFile(join(mount.mountpoint, "hello.txt"), "hello from SmolFS\n");
  fs.flush(volume.name);
} finally {
  fs.unmount(volume.name);
}
```

Cloud volumes use the same options object:

```ts
fs.ensureVolume({
  name: "agent-workspace",
  metadata: "redis://localhost:6379/1",
  storage: "s3",
  bucket: "https://my-bucket.s3.us-east-2.amazonaws.com"
});
```

## Development

Work from a source checkout when you are changing SmolFS itself or when a CLI
release asset has not been published yet.

Build and check the CLI:

```bash
cargo build -p smolfs-cli
./target/debug/smolfs doctor
```

Run the normal quality checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Run the R2-style lifecycle from this checkout:

```bash
cargo build -p smolfs-cli

set -a
. ./.env
set +a

export SMOLFS_HOME=/tmp/smolfs-r2-home
VOL="r2demo-$(date +%Y%m%d%H%M%S)"
MOUNT="/tmp/smolfs-r2-workspace"

./target/debug/smolfs init "$VOL" \
  --metadata "$SMOLFS_R2_METADATA" \
  --storage s3 \
  --bucket "$SMOLFS_R2_BUCKET_URL"

./target/debug/smolfs mount "$VOL" "$MOUNT"
echo "hello from smolfs r2" > "$MOUNT/hello.txt"
./target/debug/smolfs flush "$VOL"
./target/debug/smolfs unmount "$VOL"
./target/debug/smolfs mount "$VOL" "$MOUNT"
cat "$MOUNT/hello.txt"
```

Run the MinIO integration test path when JuiceFS, Redis, and a MinIO bucket are
available:

```bash
SMOLFS_RUN_INTEGRATION=1 cargo test -p smolfs-juicefs --test minio_integration -- --nocapture
```

Build the Python wheel:

```bash
uvx maturin build --manifest-path bindings/python/Cargo.toml --interpreter python
```

Develop the Python binding locally:

```bash
uvx maturin develop --manifest-path bindings/python/Cargo.toml
```

Test the TypeScript SDK:

```bash
cd bindings/node
npm ci
npm test
```

## Project Layout

| Path | Purpose |
| --- | --- |
| `crates/smolfs-core/` | Shared models, config, registry, paths, validation, and errors. |
| `crates/smolfs-juicefs/` | JuiceFS command wrapper, doctor checks, service layer, and integration tests. |
| `crates/smolfs-cli/` | User-facing CLI. |
| `bindings/python/` | Python SDK built from the Rust core with PyO3 and maturin. |
| `bindings/node/` | TypeScript SDK built from the Rust core with napi-rs. |
| `.github/workflows/` | CI and package publishing workflows. |

## Security and Reliability

SmolFS stores agent workspace data outside the sandbox lifecycle. Treat it like
durable infrastructure.

- Do not log credentials, S3 access keys, Redis URLs with secrets, or mount
  tokens.
- Prefer explicit object-store configuration over hidden global state.
- Make mount and unmount behavior idempotent where possible.
- Fail loudly on missing JuiceFS, missing metadata URLs, missing object-store
  config, or missing FUSE support.
- Avoid changes that weaken persistence guarantees without calling them out.

## Releases

The `smolfs` command is built from the Rust CLI crate. The GitHub workflow at
`.github/workflows/publish-cli.yml` builds Linux and macOS release binaries for
x86_64 and arm64 targets, smoke-tests `smolfs --help`, and attaches tarballs to
`v*` GitHub releases.

Python packaging uses `uv` and `maturin`. The GitHub workflow at
`.github/workflows/publish-python.yml` builds wheels for Linux and macOS, builds
an sdist, and publishes to PyPI when a `v*` tag is pushed.

Before the first Python release, configure PyPI Trusted Publishing:

1. Create or claim the `smolfs` project on PyPI.
2. Add a trusted publisher for repository `CelestoAI/smolfs`.
3. Set the workflow name to `publish-python.yml`.
4. Set the environment name to `pypi`.

Release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Roadmap

- Replace the current managed JuiceFS copy flow with real cross-platform
  JuiceFS downloads in `smolfs doctor --install`.
- Add npm publishing with prebuilt TypeScript SDK native artifacts.
- Add type stubs for the Python package.
- Add a Linux CI job that mounts a local dev volume when FUSE is available.
- Add release notes and a changelog before the first non-draft release.

## License

Apache 2.0.

---

<div align="center">
Built in London by <a href="https://celesto.ai">Celesto AI</a>
</div>
