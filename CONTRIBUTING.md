# Contributing to Flux

## Contribution Scope

Flux welcomes contributions that align with the project's goals: reliable DNS/NTP/VPN configuration for Linux and BSD systems. All contributions are subject to maintainer discretion.

## Pull Request Acceptance

- PRs must include tests for new functionality.
- PRs must pass `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
- PRs must be signed off per the DCO (Developer Certificate of Origin).
- Maintainer reserves the right to reject PRs that do not align with project scope or roadmap.

## Developer Certificate of Origin

By contributing to this project, you agree to the DCO v1.1:

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.
```

## Security Reporting

Report security issues directly to Mohamed.Hammad@SpacecraftSoftware.org. Do not open public issues for security vulnerabilities.

## License of Contributions

All contributions are licensed under GPL-3.0-or-later.

## Code Style

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- SPDX header on every `.rs` file

*--- Forged in Spacecraft Software ---*
