import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { checksumLines, collectFiles } from "./artifact-checksums.mjs";

test("collects recursively and emits deterministic SHA-256 lines", () => {
  const directory = join(tmpdir(), `arclume-checksums-${process.pid}-${Date.now()}`);
  mkdirSync(join(directory, "nested"), { recursive: true });
  writeFileSync(join(directory, "b.bin"), "beta");
  writeFileSync(join(directory, "nested", "a.bin"), "alpha");
  try {
    const files = collectFiles(directory);
    const lines = checksumLines(files, directory);
    assert.deepEqual(lines, [
      "f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753  b.bin",
      "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8  nested/a.bin",
    ]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
