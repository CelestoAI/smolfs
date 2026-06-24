const assert = require("node:assert/strict");
const { chmodSync, mkdirSync, mkdtempSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { spawnSync } = require("node:child_process");

const root = mkdtempSync(join(tmpdir(), "smolfs-node-pack-"));
const project = join(root, "project");
const juicefs = join(root, "juicefs");

mkdirSync(project);

writeFileSync(
  juicefs,
  [
    "#!/bin/sh",
    "if [ \"$1\" = \"version\" ]; then",
    "  echo \"juicefs mock 1.0.0\"",
    "  exit 0",
    "fi",
    "exit 0",
    ""
  ].join("\n")
);
chmodSync(juicefs, 0o755);

run("npm", ["pack", "--pack-destination", root], process.cwd());
run("npm", ["init", "-y"], root);

const tarball = join(root, "celestoai-smolfs-0.1.0.tgz");
run("npm", ["install", tarball], project);

const smoke = [
  "const assert = require('node:assert/strict');",
  "const { SmolFS, doctor } = require('@celestoai/smolfs');",
  "assert.equal(typeof SmolFS, 'function');",
  "assert.equal(typeof doctor, 'function');",
  "assert.equal(doctor().juicefs.version, 'juicefs mock 1.0.0');",
  ""
].join("\n");

run(process.execPath, ["-e", smoke], project, {
  ...process.env,
  SMOLFS_HOME: join(root, "home"),
  SMOLFS_JUICEFS_BIN: juicefs
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
