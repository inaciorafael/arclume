# ADR-004: Application discovery

Status: accepted at architecture level.

Use a shared `AppProvider` contract with separate Windows, macOS and Linux adapters. A generic PATH scan is rejected because installed applications, display names, icons and launch semantics differ materially across operating systems.

Each adapter owns identity normalization, discovery, icon references and launch metadata. Platform implementations require platform-specific fixtures and manual acceptance tests.
