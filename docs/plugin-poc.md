# Isolated plugin POC

Phase 6 introduces one deliberately narrow provider: `hello-world`. Searching for `hello`, `hello world`, `ola` or `olá` returns a structured plugin result.

## Isolation boundary

The desktop process starts a second Arclume process with `--plugin-host`, sends one JSON request over stdin and reads one JSON response from stdout. The host exits after the call. A call has a 100 ms deadline; the parent kills the host on expiry.

This POC does not load arbitrary native libraries or execute manifest-provided paths. The embedded provider has no granted capabilities. Its checked-in manifest is compiled into and validated by the host.

## Enforced limits

- plugin API version: `1`;
- query: 512 bytes;
- request: 16 KiB;
- response: 32 KiB;
- results per call: 8;
- provider deadline: 100 ms;
- unknown JSON fields: rejected;
- mismatched request ID or API version: rejected.

Failures are isolated: malformed output, timeout, process failure or contract mismatch produces no plugin results and does not fail the launcher search.

## Deliberate limitations

This is not yet a public plugin SDK. It has one built-in provider, no installer, no third-party code loading, no persistent host and no capability broker calls. Phase 7 must benchmark the process protocol against WASM/WASI before choosing the public runtime and versioning policy.

## Acceptance

1. Build and unit tests pass.
2. A direct host request returns a valid `plugin:hello-world:greeting` result.
3. The desktop search displays the result for `hello`.
4. Host latency stays below the 100 ms deadline on the development machine.

## Evidence

On the Phase 6 Windows development machine, a valid direct request to the debug host completed in **45.79 ms**, including process startup. The measurement is evidence for this machine only; release builds and macOS/Linux still require acceptance runs.
