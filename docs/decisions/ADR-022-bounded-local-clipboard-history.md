# ADR-022: Bounded local clipboard history

## Status

Accepted for phase 20.

## Decision

Arclume provides an opt-in clipboard history for text and images while the application is running. Data stays in a dedicated local SQLite database and is never sent to plugins or remote services.

Retention is enforced after every capture and settings change by three simultaneous limits: maximum item count, maximum age, and maximum stored payload bytes. Images are PNG-compressed before persistence; result lists query metadata only and load image bytes only for the selected preview. Raw images above 64 MB and individual encoded payloads above the configured disk limit are rejected.

The default is disabled because clipboard contents can contain credentials and other sensitive data. The first release does not capture arbitrary clipboard formats or reconstruct clipboard activity from before Arclume started.

## Consequences

- Memory use stays bounded because image payloads are not loaded with the history list.
- Disk growth is controlled independently from the main search index.
- Users can pause capture and clear all retained content.
- Clipboard monitoring uses a background thread to avoid desktop clipboard deadlocks on the UI thread.
