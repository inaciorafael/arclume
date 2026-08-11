# Security policy

Do not report vulnerabilities in public issues. Until a private disclosure channel exists, do not distribute production builds or untrusted plugins.

Security boundaries:

- Vue is untrusted presentation code and cannot grant capabilities.
- Rust validates every privileged action and plugin permission.
- Tauri capabilities follow least privilege.
- Clipboard and history remain local and opt-in.
- Plugins never receive unrestricted filesystem, network, process or clipboard access.

Search and clipboard history are stored locally in separate SQLite databases and can be cleared from the launcher. Clipboard capture is opt-in and bounded by item count, retention age, and stored bytes. The only plugin execution is the embedded, capability-free hello-world POC in an isolated short-lived process; arbitrary third-party entrypoints remain disabled. There is no remote synchronization, analytics upload, or unrestricted command execution.

Phase 9 explicitly defers remote plugin distribution until package signing, digest verification, revocation, permission grants, auditing and a private vulnerability disclosure process exist.

Package dry runs are unsigned and retained as CI artifacts for seven days. They are build evidence, not trusted production releases. Public release remains blocked on platform signing, clean-machine acceptance, and a private disclosure channel.
