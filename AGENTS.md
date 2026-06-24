# SmolFS Context

SmolFS is a developer-facing filesystem layer for AI agent workspaces. It wraps
JuiceFS with a Rust core, a CLI, and language bindings so agents can use a
durable, local-looking directory backed by object storage.

## Project Overview

SmolFS is designed for agent volumes: persistent workspaces that can be mounted
inside short-lived or long-running agent runtimes without each runtime needing
to manage JuiceFS directly.

- **Purpose:** Make object-storage-backed filesystems easy to create, mount, and
  use from agent tooling.
- **Technology stack:**
  - **Core language:** Rust
  - **Filesystem backend:** JuiceFS
  - **Metadata stores:** SQLite for local/dev use, Redis for shared/cloud use
  - **Object storage:** S3-compatible stores, tested with MinIO in CI
  - **Python packaging:** `uv` plus `maturin`
  - **TypeScript packaging:** Node.js native bindings via `napi-rs`
- **Current scope:** Focus on SmolFS developer experience. Do not assume SmolVM
  production integration is in scope unless the task explicitly asks for it.

## Building and Running

### Prerequisites

- Rust toolchain matching the workspace `rust-version`
- JuiceFS available on `PATH` for integration tests and local mounts
- FUSE support on machines that mount volumes
- Docker for the MinIO and Redis integration test path

### Key Commands

- **Format:** `cargo fmt --all -- --check`
- **Lint:** `cargo clippy --workspace -- -D warnings`
- **Unit tests:** `cargo test --workspace`
- **Integration tests:** set `SMOLFS_RUN_INTEGRATION=1` and run
  `cargo test -p smolfs-juicefs --test minio_integration -- --nocapture`
- **Build Python wheel:** `uvx maturin build --manifest-path bindings/python/Cargo.toml --interpreter python`
- **Develop Python binding locally:** `uvx maturin develop --manifest-path bindings/python/Cargo.toml`
- **Test TypeScript SDK:** `cd bindings/node && npm ci && npm test`

## Project Structure

- `crates/smolfs-core/`: Shared models, config, registry, paths, validation, and errors.
- `crates/smolfs-juicefs/`: JuiceFS command wrapper, doctor checks, service layer, and integration tests.
- `crates/smolfs-cli/`: User-facing CLI.
- `bindings/python/`: Python SDK built from the Rust core with PyO3 and maturin.
- `bindings/node/`: TypeScript SDK built from the Rust core with napi-rs.
- `.github/workflows/`: CI and Python package publishing workflows.

## Development Conventions

- Keep the Rust core as the source of truth. Bindings should stay thin.
- Prefer explicit, typed APIs over stringly helper layers.
- Keep the V0 surface lean: initialize volumes, mount, flush, unmount, inspect
  status, and run doctor checks.
- Preserve the distinction between dev volumes and cloud volumes:
  - Dev volumes use SQLite metadata and local file storage.
  - Cloud volumes use explicit metadata and object storage settings.
- Tests should cover both Redis and SQLite metadata when touching JuiceFS or
  object-store behavior.
- Use `uv`/`uvx`, not raw `pip`, for Python packaging and local SDK workflows.
- Keep GitHub Actions dependencies pinned to immutable commit SHAs or image
  digests when editing workflows.

## Security and Reliability for Agents

SmolFS stores agent workspace data outside the sandbox lifecycle. Treat the
filesystem boundary as durable infrastructure:

- Do not log credentials, S3 access keys, Redis URLs with secrets, or mount
  tokens.
- Prefer explicit object-store configuration over hidden global state.
- Make mount and unmount behavior idempotent where possible.
- Fail loudly on missing JuiceFS, missing metadata URLs, missing object-store
  config, or missing FUSE support.
- Avoid changes that weaken persistence guarantees without calling them out.

## Core Writing Principles

- Follow progressive disclosure of complexity.
- Lead with outcomes, not implementation details.
- The first paragraph of every page must be plain English with no jargon.
- Assume the reader may be a beginner engineer or even a non-developer.
- Do not assume prior knowledge.
- Explain what the user can do and why it matters before explaining how it works.
- Do not introduce a new concept unless the page truly needs it.
- If you must use a technical term, explain it immediately in simple language.
- Prefer short, concrete sentences over dense explanations.
