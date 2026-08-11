import assert from "node:assert/strict";
import test from "node:test";

import { cargoLockPackage, cargoPackage, validateReleaseState } from "./release-preflight.mjs";

function validState() {
  return {
    packageJson: { name: "arclume", version: "0.1.0" },
    packageLock: { name: "arclume", version: "0.1.0", packages: { "": { version: "0.1.0" } } },
    cargoPackage: { name: "arclume", version: "0.1.0" },
    cargoLockVersion: "0.1.0",
    tauri: { productName: "Arclume", version: "0.1.0", identifier: "com.arclume.launcher" },
    inventory: { project: { name: "arclume", version: "0.1.0" }, locks: { npmSha256: "npm", cargoSha256: "cargo" } },
    packageLockHash: "npm",
    cargoLockHash: "cargo",
  };
}

test("parses only the Cargo package section and application lock entry", () => {
  const manifest = `[package]\nname = "arclume"\nversion = "0.1.0"\n\n[dependencies]\nversion = "9.9.9"\n`;
  const lock = `[[package]]\nname = "dependency"\nversion = "9.9.9"\n\n[[package]]\nname = "arclume"\nversion = "0.1.0"\n`;
  assert.deepEqual(cargoPackage(manifest), { name: "arclume", version: "0.1.0" });
  assert.equal(cargoLockPackage(lock, "arclume"), "0.1.0");
});

test("accepts a synchronized release state", () => {
  assert.deepEqual(validateReleaseState(validState()), { ok: true, version: "0.1.0", errors: [] });
});

test("reports every mismatch instead of failing on the first", () => {
  const state = validState();
  state.packageLock.version = "0.2.0";
  state.tauri.version = "0.3.0";
  state.inventory.locks.cargoSha256 = "stale";
  const result = validateReleaseState(state);
  assert.equal(result.ok, false);
  assert.equal(result.errors.length, 3);
  assert(result.errors.some((error) => error.includes("package-lock.json version")));
  assert(result.errors.some((error) => error.includes("Tauri version")));
  assert(result.errors.some((error) => error.includes("inventory Cargo lock hash")));
});

test("rejects non-SemVer application versions", () => {
  const state = validState();
  state.packageJson.version = "next";
  const result = validateReleaseState(state);
  assert(result.errors.some((error) => error.includes("valid SemVer")));
});
