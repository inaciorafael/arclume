import assert from "node:assert/strict";
import test from "node:test";

import { buildInventory, cargoChecksums, npmPackages, sha256 } from "./dependency-inventory.mjs";

test("normalizes and sorts npm lock packages", () => {
  const packages = npmPackages({ packages: {
    "": { name: "app", version: "1.0.0" },
    "node_modules/zeta": { version: "2.0.0", license: "MIT", integrity: "sha512-z" },
    "node_modules/@scope/alpha": { version: "1.0.0", dev: true },
  } });
  assert.deepEqual(packages.map(({ name }) => name), ["@scope/alpha", "zeta"]);
  assert.equal(packages[0].development, true);
});

test("extracts checksums from Cargo.lock", () => {
  const checksums = cargoChecksums(`version = 4\n[[package]]\nname = "demo"\nversion = "1.2.3"\nchecksum = "abc"\n`);
  assert.equal(checksums.get("demo@1.2.3"), "abc");
});

test("builds a deterministic inventory without timestamps", () => {
  const packageLockContent = JSON.stringify({ name: "app", version: "1.0.0", packages: {} });
  const cargoLockContent = "version = 4\n";
  const metadata = { packages: [] };
  const first = buildInventory({ packageLockContent, cargoLockContent, metadata });
  const second = buildInventory({ packageLockContent, cargoLockContent, metadata });
  assert.deepEqual(first, second);
  assert.equal(first.locks.npmSha256, sha256(packageLockContent));
  assert.equal("generatedAt" in first, false);
});
