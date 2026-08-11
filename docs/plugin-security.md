# Plugin security

Threats include malicious packages, compromised updates, infinite loops, excessive memory, filesystem exfiltration, process execution and confused-deputy attacks.

Required controls are deny-by-default capabilities, per-call validation, timeouts, structured IPC, size limits, origin metadata, auditable grants and process termination. Manifest declarations are requests, not authorization. Native plugins are outside the initial public model because an in-process native library defeats meaningful isolation.

The Phase 6 POC enforces process isolation, a 100 ms timeout, structured JSON with unknown-field rejection, request/response/result size limits and no capabilities. Origin grants, auditing, third-party package verification and a general permission broker remain hardening work; therefore arbitrary plugins cannot be installed or loaded yet. Phase 9 also requires signed catalog metadata, artifact digests, key rotation and revocation before remote distribution.
