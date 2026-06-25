const assert = require("node:assert/strict");
const { chmodSync, mkdirSync, mkdtempSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { spawnSync } = require("node:child_process");
const { version } = require("../package.json");

const root = mkdtempSync(join(tmpdir(), "smolfs-node-pack-"));
const project = join(root, "project");
const storageBackend = join(root, "smolfs-storage");

mkdirSync(project);

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

run("npm", ["pack", "--pack-destination", root], process.cwd());
run("npm", ["init", "-y"], root);

const tarball = join(root, `celestoai-smolfs-${version}.tgz`);
run("npm", ["install", tarball], project);

const smoke = [
  "const assert = require('node:assert/strict');",
  "const { SmolFS, doctor } = require('@celestoai/smolfs');",
  "assert.equal(typeof SmolFS, 'function');",
  "assert.equal(typeof doctor, 'function');",
  "assert.equal(doctor().storageBackend.version, '1.0.0');",
  ""
].join("\n");

run(process.execPath, ["-e", smoke], project, {
  ...process.env,
  SMOLFS_HOME: join(root, "home"),
  SMOLFS_STORAGE_BACKEND_BIN: storageBackend
});

function run(command, args, cwd, env = process.env) {
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: "utf8"
  });

  assert.equal(
    result.status,
    0,
    [
      `${command} ${args.join(" ")} failed`,
      result.stdout,
      result.stderr
    ].join("\n")
  );
}
