import assert from "node:assert/strict";
import { execFile as execFileCallback } from "node:child_process";
import { readFile, rm } from "node:fs/promises";
import { join, resolve } from "node:path";
import test from "node:test";
import { promisify } from "node:util";

import { createPlugin, validateFile, validateManifest } from "./plugin-cli.mjs";

const execFile = promisify(execFileCallback);

const validManifest = {
  id: "hello-world",
  name: "Hello World",
  version: "0.1.0",
  pluginApiVersion: 1,
  entrypoint: "builtin:hello-world",
  capabilities: [],
  contributes: { providers: ["greeting"] },
};

test("accepts the checked-in manifest contract", () => {
  assert.deepEqual(validateManifest(validManifest), []);
});

test("rejects unknown fields, capabilities and API versions", () => {
  const errors = validateManifest({
    ...validManifest,
    pluginApiVersion: 2,
    capabilities: ["process:execute"],
    surprise: true,
  });
  assert.ok(errors.some((error) => error.includes("unknown field")));
  assert.ok(errors.some((error) => error.includes("pluginApiVersion")));
  assert.ok(errors.some((error) => error.includes("unknown capabilities")));
});

test("scaffolds a valid plugin without overwriting", async () => {
  const id = `sdk-test-${process.pid}`;
  const pluginDirectory = resolve("plugins", id);
  try {
    await createPlugin(id);
    assert.deepEqual(await validateFile(join(pluginDirectory, "plugin.json")), []);
    assert.match(await readFile(join(pluginDirectory, "src", "provider.ts"), "utf8"), /defineProvider/);
    assert.match(await readFile(join(pluginDirectory, "tsconfig.json"), "utf8"), /plugin-sdk/);
    assert.match(await readFile(join(pluginDirectory, "package.json"), "utf8"), /typecheck/);
    try {
      await execFile(process.execPath, [
        resolve("node_modules/typescript/bin/tsc"),
        "--noEmit",
        "-p",
        join(pluginDirectory, "tsconfig.json"),
      ]);
    } catch (error) {
      throw new Error(`generated scaffold did not typecheck:\n${error.stdout ?? ""}${error.stderr ?? ""}`);
    }
    await assert.rejects(() => createPlugin(id), /refusing to overwrite/);
  } finally {
    await rm(pluginDirectory, { recursive: true, force: true });
  }
});
