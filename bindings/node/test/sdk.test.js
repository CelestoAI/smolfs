const assert = require("node:assert/strict");
const { chmodSync, mkdtempSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = require("node:test");

const { SmolFS, doctor } = require("../index.js");

function configureSandbox() {
  const root = mkdtempSync(join(tmpdir(), "smolfs-node-test-"));
  const storageBackend = join(root, "smolfs-storage");

  writeFileSync(
    storageBackend,
    [
      "#!/bin/sh",
      "if [ \"$1\" = \"version\" ]; then",
      "  echo \"storage backend mock 1.0.0\"",
      "  exit 0",
      "fi",
      "exit 0",
      ""
    ].join("\n")
  );
  chmodSync(storageBackend, 0o755);

  process.env.SMOLFS_HOME = join(root, "home");
  process.env.SMOLFS_STORAGE_BACKEND_BIN = storageBackend;
  delete process.env.SMOLFS_JUICEFS_BIN;

  return root;
}

test("doctor reports the configured storage backend", () => {
  configureSandbox();

  const report = doctor();

  assert.equal(report.storageBackend.found, true);
  assert.equal(report.storageBackend.version, "1.0.0");
  assert.match(report.home, /smolfs-node-test-/);
});

test("SmolFS can ensure and inspect a dev volume", () => {
  configureSandbox();

  const fs = SmolFS.fromEnv();
  const volume = fs.ensureVolume({ name: "demo", dev: true });

  assert.equal(volume.name, "demo");
  assert.equal(volume.dev, true);
  assert.match(volume.metadataUrl, /^sqlite3:\/\//);
  assert.equal(volume.storage, "file");

  const sameVolume = fs.ensureVolume({ name: "demo", dev: true });
  assert.equal(sameVolume.name, "demo");

  const status = fs.status();
  assert.deepEqual(status.volumes.map((item) => item.name), ["demo"]);
});

test("SmolFS surfaces Rust validation errors", () => {
  configureSandbox();

  const fs = new SmolFS();

  assert.throws(
    () => fs.ensureVolume({ name: "bad name", dev: true }),
    /invalid volume name/
  );
});

test("SmolFS does not expose command arguments in native errors", () => {
  const root = configureSandbox();
  const storageBackend = join(root, "smolfs-storage");

  writeFileSync(
    storageBackend,
    [
      "#!/bin/sh",
      "if [ \"$1\" = \"version\" ]; then",
      "  echo \"storage backend mock 1.0.0\"",
      "  exit 0",
      "fi",
      "echo \"stderr: $*\" >&2",
      "exit 2",
      ""
    ].join("\n")
  );
  chmodSync(storageBackend, 0o755);

  const fs = SmolFS.fromEnv();
  const metadata = "redis://:supersecret@localhost:6379/1";

  assert.throws(
    () =>
      fs.ensureVolume({
        name: "cloud",
        metadata,
        storage: "s3",
        bucket: "https://example-bucket.s3.amazonaws.com"
      }),
    (error) => {
      assert.match(error.message, /SmolFS storage backend/);
      assert.doesNotMatch(error.message, /supersecret/);
      assert.doesNotMatch(error.message, /redis:\/\//);
      assert.doesNotMatch(error.message, /stderr:/);
      assert.doesNotMatch(error.message, /juicefs/i);
      return true;
    }
  );
});
