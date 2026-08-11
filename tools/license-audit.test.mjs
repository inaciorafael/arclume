import assert from "node:assert/strict";
import test from "node:test";

import { auditInventory, evaluateLicense, markdownSummary } from "./license-audit.mjs";

test("evaluates SPDX AND, OR, parentheses and approved exceptions", () => {
  assert.equal(evaluateLicense("MIT OR GPL-3.0-only").allowed, true);
  assert.equal(evaluateLicense("MIT AND GPL-3.0-only").allowed, false);
  assert.equal(evaluateLicense("(MIT OR Apache-2.0) AND Unicode-3.0").allowed, true);
  assert.equal(evaluateLicense("Apache-2.0 WITH LLVM-exception").allowed, true);
});

test("normalizes known legacy slash expressions", () => {
  assert.equal(evaluateLicense("MIT/Apache-2.0").allowed, true);
  assert.equal(evaluateLicense("Unlicense/MIT").allowed, true);
});

test("rejects missing, unknown and mandatory strong-copyleft licenses", () => {
  assert.deepEqual(evaluateLicense(null), { allowed: false, unknown: ["<missing>"] });
  assert.equal(evaluateLicense("Unknown-1.0").allowed, false);
  assert.equal(evaluateLicense("AGPL-3.0-only").allowed, false);
});

test("audits both ecosystems and renders a deterministic disclaimer", () => {
  const inventory = {
    project: { name: "arclume", version: "0.1.0" },
    ecosystems: {
      npm: [{ name: "vue", version: "1", license: "MIT" }],
      cargo: [{ name: "demo", version: "2", license: "Apache-2.0 OR MIT" }],
    },
  };
  const audit = auditInventory(inventory);
  assert.equal(audit.ok, true);
  assert.equal(audit.packageCount, 2);
  assert.match(markdownSummary(inventory, audit), /not legal advice/);
});
