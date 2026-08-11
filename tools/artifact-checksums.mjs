import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROJECT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export function collectFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) files.push(...collectFiles(path));
    else if (entry.isFile()) files.push(path);
  }
  return files.sort((left, right) => left.localeCompare(right));
}

export function checksumLines(files, root = PROJECT_ROOT) {
  return files.map((path) => {
    const digest = createHash("sha256").update(readFileSync(path)).digest("hex");
    const name = relative(root, path).replaceAll("\\", "/");
    return `${digest}  ${name}`;
  });
}

function option(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function main() {
  const directoryValue = option("--directory");
  const outputValue = option("--output") ?? "artifacts/SHA256SUMS";
  if (!directoryValue) throw new Error("--directory requires a bundle directory");
  const directory = resolve(PROJECT_ROOT, directoryValue);
  if (!existsSync(directory) || !statSync(directory).isDirectory()) {
    throw new Error(`bundle directory does not exist: ${directory}`);
  }
  const files = collectFiles(directory);
  if (!files.length) throw new Error(`bundle directory has no files: ${directory}`);
  const output = resolve(PROJECT_ROOT, outputValue);
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${checksumLines(files).join("\n")}\n`, { flag: "w" });
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
