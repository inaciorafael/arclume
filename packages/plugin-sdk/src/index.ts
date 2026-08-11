export const PLUGIN_API_VERSION = 1 as const;

export const PLUGIN_LIMITS = Object.freeze({
  queryBytes: 512,
  requestBytes: 16 * 1024,
  responseBytes: 32 * 1024,
  resultsPerCall: 8,
  deadlineMs: 100,
});

export type PluginCapability =
  | "clipboard:read"
  | "clipboard:write"
  | "filesystem:read"
  | "network:fetch";

export interface PluginManifestV1 {
  id: string;
  name: string;
  version: string;
  pluginApiVersion: typeof PLUGIN_API_VERSION;
  entrypoint: string;
  capabilities: PluginCapability[];
  contributes: {
    providers: string[];
  };
}

export interface ProviderRequestV1 {
  pluginApiVersion: typeof PLUGIN_API_VERSION;
  requestId: number;
  query: string;
}

export interface ProviderResultV1 {
  id: string;
  title: string;
  subtitle: string;
  score: number;
}

export interface ProviderResponseV1 {
  pluginApiVersion: typeof PLUGIN_API_VERSION;
  requestId: number;
  results: ProviderResultV1[];
  diagnostic: string | null;
}

export type ProviderV1 = (
  request: Readonly<ProviderRequestV1>,
) => ProviderResultV1[] | Promise<ProviderResultV1[]>;

export function defineProvider(provider: ProviderV1): ProviderV1 {
  return provider;
}

export function createResponse(
  request: Readonly<ProviderRequestV1>,
  results: readonly ProviderResultV1[],
  diagnostic: string | null = null,
): ProviderResponseV1 {
  if (request.pluginApiVersion !== PLUGIN_API_VERSION) {
    throw new Error(`Unsupported plugin API version: ${request.pluginApiVersion}`);
  }
  if (!Number.isSafeInteger(request.requestId) || request.requestId < 0) {
    throw new Error("requestId must be a non-negative safe integer");
  }
  if (new TextEncoder().encode(request.query).byteLength > PLUGIN_LIMITS.queryBytes) {
    throw new Error(`query may contain at most ${PLUGIN_LIMITS.queryBytes} UTF-8 bytes`);
  }
  if (results.length > PLUGIN_LIMITS.resultsPerCall) {
    throw new Error(`A provider may return at most ${PLUGIN_LIMITS.resultsPerCall} results`);
  }
  const normalizedResults = results.map(validateResult);
  const response = {
    pluginApiVersion: PLUGIN_API_VERSION,
    requestId: request.requestId,
    results: normalizedResults,
    diagnostic,
  };
  if (new TextEncoder().encode(JSON.stringify(response)).byteLength > PLUGIN_LIMITS.responseBytes) {
    throw new Error(`response may contain at most ${PLUGIN_LIMITS.responseBytes} UTF-8 bytes`);
  }
  return response;
}

function validateResult(result: ProviderResultV1): ProviderResultV1 {
  if (!result.id.startsWith("plugin:") || result.id.length > 160) {
    throw new Error("Plugin result IDs must start with 'plugin:' and contain at most 160 characters");
  }
  if (!result.title.trim() || result.title.length > 160 || result.subtitle.length > 240) {
    throw new Error("Plugin result text violates the SDK limits");
  }
  if (!Number.isSafeInteger(result.score) || result.score < 0 || result.score > 10_000) {
    throw new Error("Plugin result score must be an integer between 0 and 10000");
  }
  return { ...result };
}
