import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROJECT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DEFAULT_OUTPUT = resolve(PROJECT_ROOT, "artifacts", "dependency-inventory.json");

export function sha256(content) {
  return createHash("sha256").update(content).digest("hex");
}

export function npmPackages(lock) {
  const packages = new Map();
  for (const [path, value] of Object.entries(lock.packages ?? {})) {
    if (!path || !value.version) continue;
    const name = path.split("node_modules/").at(-1);
    const key = `${name}@${value.version}`;
    packages.set(key, {
      name,
      version: value.version,
      license: value.license ?? null,
      integrity: value.integrity ?? null,
      development: value.dev === true,
      optional: value.optional === true,
    });
  }
  return [...packages.values()].sort(comparePackage);
}

export function cargoChecksums(lockContent) {
  const checksums = new Map();
  for (const block of lockContent.split("[[package]]").slice(1)) {
    const name = block.match(/^\s*name = "([^"]+)"/m)?.[1];
    const version = block.match(/^\s*version = "([^"]+)"/m)?.[1];
    const checksum = block.match(/^\s*checksum = "([^"]+)"/m)?.[1] ?? null;
    if (name && version) checksums.set(`${name}@${version}`, checksum);
  }
  return checksums;
}

export function rustPackages(metadata, checksums) {
  return metadata.packages
    .filter((value) => value.source)
    .map((value) => ({
      name: value.name,
      version: value.version,
      license: value.license ?? null,
      source: value.source,
      checksum: checksums.get(`${value.name}@${value.version}`) ?? null,
    }))
    .sort(comparePackage);
}

function comparePackage(left, right) {
  return left.name.localeCompare(right.name) || left.version.localeCompare(right.version);
}

function cargoMetadata() {
  const result = spawnSync(
    "rustup",
    ["run", "1.97.1", "cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", "src-tauri/Cargo.toml"],
    { cwd: PROJECT_ROOT, encoding: "utf8", windowsHide: true, maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.status !== 0) {
    const detail = result.error?.message ?? (result.stderr.trim() || `exit code ${result.status}`);
    throw new Error(`cargo metadata failed: ${detail}`);
  }
  return JSON.parse(result.stdout);
}

export function buildInventory({ packageLockContent, cargoLockContent, metadata }) {
  const packageLock = JSON.parse(packageLockContent);
  return {
    schemaVersion: 1,
    project: { name: packageLock.name, version: packageLock.version },
    locks: {
      npmSha256: sha256(packageLockContent),
      cargoSha256: sha256(cargoLockContent),
    },
    ecosystems: {
      npm: npmPackages(packageLock),
      cargo: rustPackages(metadata, cargoChecksums(cargoLockContent)),
    },
  };
}

function main() {
  const outputIndex = process.argv.indexOf("--output");
  const output = outputIndex >= 0 ? resolve(PROJECT_ROOT, process.argv[outputIndex + 1]) : DEFAULT_OUTPUT;
  if (outputIndex >= 0 && !process.argv[outputIndex + 1]) throw new Error("--output requires a path");
  const packageLockContent = readFileSync(resolve(PROJECT_ROOT, "package-lock.json"), "utf8");
  const cargoLockContent = readFileSync(resolve(PROJECT_ROOT, "src-tauri", "Cargo.lock"), "utf8");
  const inventory = buildInventory({ packageLockContent, cargoLockContent, metadata: cargoMetadata() });
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(inventory, null, 2)}\n`, { flag: "w" });
  process.stdout.write(`${output}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
