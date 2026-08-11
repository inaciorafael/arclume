import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { sha256 } from "./dependency-inventory.mjs";

const PROJECT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

export function cargoPackage(manifest) {
  const packageHeader = manifest.search(/^\[package\]\s*$/m);
  if (packageHeader < 0) return { name: undefined, version: undefined };
  const body = manifest.slice(packageHeader).replace(/^\[package\]\s*$/m, "");
  const nextSection = body.search(/^\[/m);
  const section = nextSection >= 0 ? body.slice(0, nextSection) : body;
  const field = (name) => section.match(new RegExp(`^${name}\\s*=\\s*"([^"]+)"`, "m"))?.[1];
  return { name: field("name"), version: field("version") };
}

export function cargoLockPackage(lock, packageName) {
  for (const block of lock.split("[[package]]").slice(1)) {
    const name = block.match(/^\s*name = "([^"]+)"/m)?.[1];
    if (name === packageName) return block.match(/^\s*version = "([^"]+)"/m)?.[1];
  }
  return undefined;
}

export function validateReleaseState(state) {
  const errors = [];
  const expectedName = "arclume";
  const expectedProduct = "Arclume";
  const expectedIdentifier = "com.arclume.launcher";
  const version = state.packageJson.version;
  const same = (label, actual, expected) => {
    if (actual !== expected) errors.push(`${label} is ${JSON.stringify(actual)}, expected ${JSON.stringify(expected)}`);
  };

  same("package.json name", state.packageJson.name, expectedName);
  if (!SEMVER.test(version ?? "")) errors.push(`package.json version is not valid SemVer: ${JSON.stringify(version)}`);
  same("package-lock.json name", state.packageLock.name, expectedName);
  same("package-lock.json version", state.packageLock.version, version);
  same("package-lock root version", state.packageLock.packages?.[""]?.version, version);
  same("Cargo package name", state.cargoPackage.name, expectedName);
  same("Cargo.toml version", state.cargoPackage.version, version);
  same("Cargo.lock application version", state.cargoLockVersion, version);
  same("Tauri product name", state.tauri.productName, expectedProduct);
  same("Tauri version", state.tauri.version, version);
  same("Tauri identifier", state.tauri.identifier, expectedIdentifier);
  same("inventory project name", state.inventory.project?.name, expectedName);
  same("inventory project version", state.inventory.project?.version, version);
  same("inventory npm lock hash", state.inventory.locks?.npmSha256, state.packageLockHash);
  same("inventory Cargo lock hash", state.inventory.locks?.cargoSha256, state.cargoLockHash);

  return { ok: errors.length === 0, version, errors };
}

function main() {
  const packageJson = JSON.parse(readFileSync(resolve(PROJECT_ROOT, "package.json"), "utf8"));
  const packageLockContent = readFileSync(resolve(PROJECT_ROOT, "package-lock.json"), "utf8");
  const packageLock = JSON.parse(packageLockContent);
  const cargoManifest = readFileSync(resolve(PROJECT_ROOT, "src-tauri", "Cargo.toml"), "utf8");
  const cargoLockContent = readFileSync(resolve(PROJECT_ROOT, "src-tauri", "Cargo.lock"), "utf8");
  const tauri = JSON.parse(readFileSync(resolve(PROJECT_ROOT, "src-tauri", "tauri.conf.json"), "utf8"));
  const inventory = JSON.parse(readFileSync(resolve(PROJECT_ROOT, "artifacts", "dependency-inventory.json"), "utf8"));
  const parsedCargoPackage = cargoPackage(cargoManifest);
  const result = validateReleaseState({
    packageJson,
    packageLock,
    cargoPackage: parsedCargoPackage,
    cargoLockVersion: cargoLockPackage(cargoLockContent, parsedCargoPackage.name),
    tauri,
    inventory,
    packageLockHash: sha256(packageLockContent),
    cargoLockHash: sha256(cargoLockContent),
  });
  if (!result.ok) throw new Error(`release preflight failed:\n- ${result.errors.join("\n- ")}`);
  process.stdout.write(`release preflight passed for Arclume ${result.version}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
