#!/usr/bin/env node
import { existsSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { pathToFileURL } from "node:url";

const API_VERSION = 1;
const CAPABILITIES = new Set([
  "clipboard:read",
  "clipboard:write",
  "filesystem:read",
  "network:fetch",
]);
const ID_PATTERN = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/;
const SEMVER_PATTERN = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/;
const ROOT_KEYS = ["id", "name", "version", "pluginApiVersion", "entrypoint", "capabilities", "contributes"];

export function validateManifest(manifest) {
  const errors = [];
  if (!isRecord(manifest)) return ["manifest must be a JSON object"];
  rejectUnknown(manifest, ROOT_KEYS, "manifest", errors);
  if (typeof manifest.id !== "string" || !ID_PATTERN.test(manifest.id) || manifest.id.length > 64) {
    errors.push("id must be a kebab-case identifier with at most 64 characters");
  }
  if (typeof manifest.name !== "string" || !manifest.name.trim() || manifest.name.length > 80) {
    errors.push("name must contain 1 to 80 characters");
  }
  if (typeof manifest.version !== "string" || !SEMVER_PATTERN.test(manifest.version)) {
    errors.push("version must be semantic versioning, for example 0.1.0");
  }
  if (manifest.pluginApiVersion !== API_VERSION) {
    errors.push(`pluginApiVersion must be ${API_VERSION}`);
  }
  if (typeof manifest.entrypoint !== "string" || !manifest.entrypoint || manifest.entrypoint.length > 160) {
    errors.push("entrypoint must contain 1 to 160 characters");
  }
  if (!Array.isArray(manifest.capabilities)) {
    errors.push("capabilities must be an array");
  } else {
    const unknown = manifest.capabilities.filter((item) => !CAPABILITIES.has(item));
    if (unknown.length) errors.push(`unknown capabilities: ${unknown.join(", ")}`);
    if (new Set(manifest.capabilities).size !== manifest.capabilities.length) errors.push("capabilities must be unique");
  }
  if (!isRecord(manifest.contributes)) {
    errors.push("contributes must be an object");
  } else {
    rejectUnknown(manifest.contributes, ["providers"], "contributes", errors);
    const providers = manifest.contributes.providers;
    if (!Array.isArray(providers) || providers.length < 1 || providers.length > 8) {
      errors.push("contributes.providers must contain 1 to 8 providers");
    } else if (providers.some((provider) => typeof provider !== "string" || !ID_PATTERN.test(provider))) {
      errors.push("provider IDs must use kebab-case");
    } else if (new Set(providers).size !== providers.length) {
      errors.push("provider IDs must be unique");
    }
  }
  return errors;
}

export async function validateFile(path) {
  let manifest;
  try {
    manifest = JSON.parse(await readFile(path, "utf8"));
  } catch (error) {
    return [`cannot read valid JSON: ${error.message}`];
  }
  return validateManifest(manifest);
}

export async function createPlugin(id, baseDirectory = "plugins") {
  if (!ID_PATTERN.test(id) || id.length > 64) throw new Error("plugin ID must use kebab-case");
  const directory = resolve(baseDirectory, id);
  if (existsSync(directory)) throw new Error(`refusing to overwrite existing directory: ${directory}`);
  await mkdir(join(directory, "src"), { recursive: true });
  const manifest = {
    id,
    name: titleCase(id),
    version: "0.1.0",
    pluginApiVersion: API_VERSION,
    entrypoint: `process:dist/${id}`,
    capabilities: [],
    contributes: { providers: ["search"] },
  };
  await writeFile(join(directory, "plugin.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  await writeFile(join(directory, "src", "provider.ts"), providerTemplate(id), "utf8");
  await writeFile(join(directory, "package.json"), packageTemplate(id), "utf8");
  await writeFile(join(directory, "tsconfig.json"), tsconfigTemplate(), "utf8");
  await writeFile(join(directory, "README.md"), readmeTemplate(id), "utf8");
  return directory;
}

async function main([command, argument]) {
  if (command === "validate") {
    if (!argument) throw new Error("usage: npm run plugin:validate -- <path-to-plugin.json>");
    const path = resolve(argument);
    const errors = await validateFile(path);
    if (errors.length) {
      for (const error of errors) console.error(`- ${error}`);
      process.exitCode = 1;
      return;
    }
    console.log(`valid plugin manifest (API v${API_VERSION}): ${path}`);
    return;
  }
  if (command === "create") {
    if (!argument) throw new Error("usage: npm run plugin:create -- <kebab-case-id>");
    console.log(`created plugin scaffold: ${await createPlugin(argument)}`);
    return;
  }
  throw new Error("usage: plugin-cli <validate|create> <argument>");
}

function rejectUnknown(value, allowed, label, errors) {
  for (const key of Object.keys(value)) if (!allowed.includes(key)) errors.push(`${label} contains unknown field: ${key}`);
}

function isRecord(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function titleCase(id) {
  return id.split("-").map((part) => part[0].toUpperCase() + part.slice(1)).join(" ");
}

function providerTemplate(id) {
  return `import { defineProvider } from "@arclume/plugin-sdk";\n\nexport default defineProvider((request) => {\n  return request.query.includes("${id}")\n    ? [{ id: "plugin:${id}:result", title: "${titleCase(id)}", subtitle: "Plugin result", score: 7000 }]\n    : [];\n});\n`;
}

function packageTemplate(id) {
  return `${JSON.stringify({ name: `@arclume-plugin/${id}`, version: "0.1.0", private: true, type: "module", scripts: { typecheck: "tsc --noEmit -p tsconfig.json" } }, null, 2)}\n`;
}

function tsconfigTemplate() {
  return `${JSON.stringify({ compilerOptions: { strict: true, target: "ES2022", module: "ESNext", moduleResolution: "Bundler", noEmit: true, paths: { "@arclume/plugin-sdk": ["../../packages/plugin-sdk/src/index.ts"] } }, include: ["src/**/*.ts"] }, null, 2)}\n`;
}

function readmeTemplate(id) {
  return `# ${titleCase(id)}\n\nGenerated against Arclume Plugin API v${API_VERSION}. This scaffold is for contract development only: Arclume does not load third-party entrypoints yet.\n`;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
