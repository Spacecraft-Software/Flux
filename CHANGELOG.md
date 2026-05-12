# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-12

### Added

- **DNS backend adapters** for Linux and BSD:
  - `systemd-resolved` — native D-Bus/CLI integration
  - `NetworkManager` — `nmcli` dispatch
  - `resolvectl` — fallback for systemd-based systems
  - `resolv.conf` — direct file manipulation with backup/restore
  - NixOS — declarative `configuration.nix` expression generation
  - FreeBSD / OpenBSD / NetBSD — `resolv.conf`-style with scoped elevation
  - Stub resolver — DoQ via `unbound` and DNSCrypt via `dnscrypt-proxy` for protocols unsupported by the detected backend
- **Protocol coverage**: Plain DNS, DoT, DoH, DoQ, DNSCrypt, WARP
- **Provider registry** with 5 providers: Cloudflare, Google, AdGuard, Quad9, OpenDNS
- **Tier system**: Standard, malware, family, unfiltered, secured, ECS variants per provider
- **Scoped privilege elevation**: parent never runs as root; only config-write child elevates via `sudo`/`pkexec`/`doas` using argv arrays (no shell interpolation)
- **Backup & restore**: timestamped snapshots with `YYYY-MM-DDTHH:MM:SSZ_<backend>.bak` naming
- **VPN orchestration**: `VpnProvider` trait with WARP and AdGuard VPN CLI adapters
- **NTP configuration**: adapters for `systemd-timesyncd`, `chrony`, `ntpd`, `openntpd`, and NixOS
- **Interactive TUI**: ratatui-based provider/protocol/tier selection (gated behind `tui` feature, enabled by default)
- **MCP server**: Model Context Protocol server surface for agentic integration (`dns mcp`)
- **Schema introspection**: `dns schema` emits structured command definitions for LLM function calling
- **Agent-friendly output**: JSON envelope with metadata, `--fields` trimming, `--format` variants (json, jsonl, yaml, csv, human, explore), AI-agent auto-detection via `AI_AGENT`/`AGENT`/`CI` env vars
- **Security hardening**:
  - Control character rejection on all string inputs
  - Prompt-injection detection (12 pattern case-insensitive scan)
  - Path traversal prevention with canonicalization and allow-listing
  - Numeric bounds checking generic helper
  - RELRO/NX/ASLR linker hardening flags
- **Compliance**: SPDX headers on all source files, `jiff` for UTC timestamps, `cargo-deny` license/advisory gating
- **CI/CD**: GitHub Actions workflow with build, test, clippy, fmt, deny, and audit gates

[0.1.0]: https://github.com/steelbore/flux/releases/tag/v0.1.0
