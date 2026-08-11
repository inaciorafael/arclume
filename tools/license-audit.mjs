import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PROJECT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const APPROVED = new Set([
  "0BSD", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "BSL-1.0", "CC0-1.0",
  "ISC", "MIT", "MIT-0", "MPL-2.0", "Unicode-3.0", "Unlicense", "Zlib",
]);
const APPROVED_EXCEPTIONS = new Set(["LLVM-exception"]);
const LEGACY = new Map([
  ["MIT/Apache-2.0", "MIT OR Apache-2.0"],
  ["MIT / Apache-2.0", "MIT OR Apache-2.0"],
  ["Apache-2.0/MIT", "Apache-2.0 OR MIT"],
  ["Apache-2.0 / MIT", "Apache-2.0 OR MIT"],
  ["BSD-3-Clause/MIT", "BSD-3-Clause OR MIT"],
  ["Unlicense/MIT", "Unlicense OR MIT"],
]);

function tokenize(expression) {
  const normalized = LEGACY.get(expression) ?? expression;
  return normalized.match(/\(|\)|\bAND\b|\bOR\b|\bWITH\b|[^\s()]+/g) ?? [];
}

export function evaluateLicense(expression) {
  if (!expression) return { allowed: false, unknown: ["<missing>"] };
  const tokens = tokenize(expression);
  const unknown = new Set();
  let index = 0;

  const primary = () => {
    if (tokens[index] === "(") {
      index += 1;
      const value = orExpression();
      if (tokens[index] !== ")") throw new Error(`unbalanced license expression: ${expression}`);
      index += 1;
      return value;
    }
    const license = tokens[index++];
    if (!license || ["AND", "OR", "WITH", ")"].includes(license)) {
      throw new Error(`invalid license expression: ${expression}`);
    }
    let allowed = APPROVED.has(license);
    if (!allowed) unknown.add(license);
    if (tokens[index] === "WITH") {
      index += 1;
      const exception = tokens[index++];
      if (!APPROVED_EXCEPTIONS.has(exception)) {
        unknown.add(exception ?? "<missing-exception>");
        allowed = false;
      }
    }
    return allowed;
  };
  const andExpression = () => {
    let value = primary();
    while (tokens[index] === "AND") {
      index += 1;
      const right = primary();
      value = value && right;
    }
    return value;
  };
  function orExpression() {
    let value = andExpression();
    while (tokens[index] === "OR") {
      index += 1;
      const right = andExpression();
      value = value || right;
    }
    return value;
  }

  const allowed = orExpression();
  if (index !== tokens.length) throw new Error(`unexpected token in license expression: ${expression}`);
  return { allowed, unknown: [...unknown].sort() };
}

export function auditInventory(inventory) {
  const packages = [
    ...inventory.ecosystems.npm.map((value) => ({ ecosystem: "npm", ...value })),
    ...inventory.ecosystems.cargo.map((value) => ({ ecosystem: "cargo", ...value })),
  ];
  const expressions = new Map();
  const rejected = [];
  for (const item of packages) {
    const evaluation = evaluateLicense(item.license);
    expressions.set(item.license ?? "<missing>", (expressions.get(item.license ?? "<missing>") ?? 0) + 1);
    if (!evaluation.allowed) rejected.push({
      ecosystem: item.ecosystem,
      name: item.name,
      version: item.version,
      license: item.license ?? null,
      unknown: evaluation.unknown,
    });
  }
  return {
    ok: rejected.length === 0,
    packageCount: packages.length,
    expressions: [...expressions].map(([license, count]) => ({ license, count })).sort((a, b) => a.license.localeCompare(b.license)),
    rejected,
  };
}

export function markdownSummary(inventory, audit) {
  const lines = [
    `# Third-party license summary — ${inventory.project.name} ${inventory.project.version}`,
    "",
    "> Generated technical inventory. This is not legal advice and does not replace required license texts or notices.",
    "",
    `Packages reviewed: ${audit.packageCount}`,
    "",
    "| Declared license expression | Packages |",
    "|---|---:|",
    ...audit.expressions.map(({ license, count }) => `| \`${license}\` | ${count} |`),
    "",
    `Technical policy result: **${audit.ok ? "PASS" : "REVIEW REQUIRED"}**`,
    "",
  ];
  return `${lines.join("\n")}\n`;
}

function main() {
  const inventory = JSON.parse(readFileSync(resolve(PROJECT_ROOT, "artifacts", "dependency-inventory.json"), "utf8"));
  const audit = auditInventory(inventory);
  const output = resolve(PROJECT_ROOT, "artifacts", "THIRD_PARTY_LICENSE_SUMMARY.md");
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, markdownSummary(inventory, audit), { flag: "w" });
  if (!audit.ok) {
    const details = audit.rejected.map((item) => `${item.ecosystem}:${item.name}@${item.version} (${item.license ?? "missing"})`).join("\n- ");
    throw new Error(`license audit requires review:\n- ${details}`);
  }
  process.stdout.write(`license audit passed for ${audit.packageCount} packages\n${output}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
