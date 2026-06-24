# SmolFS

SmolFS is a small developer-facing volume tool for agent workspaces. It wraps
JuiceFS with a Rust core, a polished CLI, and language bindings so agents can
work against a durable local-looking directory without each runtime learning
JuiceFS lifecycle details.

## Quick Start

```bash
cargo run -p smolfs-cli -- doctor
cargo run -p smolfs-cli -- init demo --dev
cargo run -p smolfs-cli -- mount demo ./workspace
echo hello > ./workspace/hello.txt
cargo run -p smolfs-cli -- unmount demo
cargo run -p smolfs-cli -- mount demo ./workspace
cat ./workspace/hello.txt
```

`--dev` uses JuiceFS with local SQLite metadata and local file storage under
`~/.smolfs/dev`.

## Python

```python
from smolfs import SmolFS

fs = SmolFS.from_env()
fs.init("demo", dev=True)
fs.mount("demo", "./workspace")
fs.flush("demo")
fs.unmount("demo")
```

