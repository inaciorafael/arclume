# Plugin SDK and developer workflow

Phase 7 provides a private, pre-release TypeScript contract package at `packages/plugin-sdk`, a JSON Schema and a dependency-free Node CLI. This makes API v1 reproducible without enabling third-party execution prematurely.

## Versions

`@arclume/plugin-sdk` uses semantic package versions. The manifest and IPC use the integer `pluginApiVersion`.

- additive SDK fixes may change the package patch/minor version without changing the API integer;
- a breaking manifest or IPC change requires a new `pluginApiVersion`;
- the host accepts only API v1 and rejects unknown fields;
- the SDK is private until the runtime and permission broker are accepted.

## Create a scaffold

```shell
npm run plugin:create -- my-provider
```

This creates `plugins/my-provider/plugin.json`, `src/provider.ts`, `package.json`, `tsconfig.json` and a README. Existing directories are never overwritten. From the repository root, run `npx tsc --noEmit -p plugins/my-provider/tsconfig.json` to typecheck the scaffold against the local SDK.

The generated `process:dist/...` entrypoint is descriptive metadata only in Phase 7. The desktop host does not load it. Only the built-in hello-world fixture runs today.

## Validate a manifest

```shell
npm run plugin:validate -- plugins/my-provider/plugin.json
```

The CLI checks identifiers, semantic version, API compatibility, known fields, declared capabilities and provider contributions. The canonical machine-readable shape is `packages/plugin-sdk/plugin.schema.json`.

## Validate the toolchain

```shell
npm run plugin:test
npm run sdk:typecheck
npm run plugin:validate -- plugins/hello-world/plugin.json
```

## Minimal provider

```ts
import { defineProvider } from "@arclume/plugin-sdk";

export default defineProvider((request) =>
  request.query === "hello"
    ? [{
        id: "plugin:my-provider:greeting",
        title: "Hello",
        subtitle: "From my provider",
        score: 7_000,
      }]
    : [],
);
```

Providers cannot directly mutate launcher state. Future privileged host calls will require both a manifest declaration and an explicit user grant; neither condition alone authorizes access.

## Current compatibility boundary

API v1 currently covers manifests, provider requests, provider results and responses. Actions, previews, settings, cancellation messages, capability grants, package signatures and installation are not part of v1 yet.
