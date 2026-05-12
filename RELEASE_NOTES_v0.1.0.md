# Flux v0.1.0 — DNS Selector & Network Configurator

## Overview

Flux is a DNS selector and network configurator CLI for Linux and BSD. It detects your system's DNS subsystem, applies encrypted DNS (DoT/DoH/DoQ/DNSCrypt), optionally configures NTP, and orchestrates VPN clients — all from a single, auditable tool.

## Quick Start

```bash
# Detect your system
dns detect

# Apply Cloudflare DNS over HTTPS (default tier)
dns apply cloudflare doh

# Apply Cloudflare DNS with malware filtering
dns apply cloudflare family doh

# List available providers and protocols
dns list --providers
dns list --protocols --provider cloudflare

# Back up current DNS config before changing
dns backup

# Verify resolution is working
dns verify

# Restore from last backup
dns restore
```

## What's New in v0.1.0

### DNS Backends (9 adapters)

| Backend | Protocols | Notes |
|---------|-----------|-------|
| `systemd-resolved` | Plain, DoT, DoH | Native D-Bus/CLI |
| `NetworkManager` | Plain, DoT, DoH | `nmcli` dispatch |
| `resolvectl` | Plain, DoT, DoH | systemd fallback |
| `resolv.conf` | Plain | Direct file manipulation |
| NixOS | All | Declarative `configuration.nix` expressions |
| FreeBSD | Plain | `resolv.conf`-style |
| OpenBSD | Plain | `resolv.conf`-style |
| NetBSD | Plain | `resolv.conf`-style |
| Stub resolver | DoQ, DNSCrypt | Auto-fallback via `unbound` / `dnscrypt-proxy` |

### Protocol Coverage

- **Plain** — standard UDP/TCP DNS
- **DoT** — DNS over TLS (port 853)
- **DoH** — DNS over HTTPS
- **DoQ** — DNS over QUIC (stub resolver fallback)
- **DNSCrypt** — encrypted DNS (stub resolver fallback)
- **WARP** — Cloudflare's WireGuard-based resolver

### Supported Providers

| Provider | Protocols | Tiers |
|----------|-----------|-------|
| Cloudflare | Plain, DoT, DoH | standard, malware, family |
| Google | Plain, DoT, DoH | standard |
| AdGuard | Plain, DoT, DoH, DoQ, DNSCrypt | standard, family, unfiltered |
| Quad9 | Plain, DoT, DoH, DNSCrypt | secured, ECS, unsecured |
| OpenDNS | Plain, DoH | standard, family |

### Security Features

- **Scoped elevation**: parent process never runs as root; only the config-writing child elevates via `sudo`/`pkexec`/`doas` using argv arrays — no shell interpolation
- **Input sanitization**: control character rejection, prompt-injection detection (12 patterns), path traversal prevention with allow-listing
- **Binary hardening**: RELRO, NOW, NX stack linker flags
- **Audit trail**: timestamped UTC backups with `YYYY-MM-DDTHH:MM:SSZ_<backend>.bak` naming

### VPN & NTP

- **VPN orchestration**: WARP (`warp-cli`) and AdGuard VPN CLI adapters with `--vpn` flag
- **NTP configuration**: `systemd-timesyncd`, `chrony`, `ntpd`, `openntpd`, and NixOS adapters via `dns ntp`

### Agent & Automation Support

- **JSON envelope**: every command emits structured output with metadata (tool, version, timestamp, command line)
- **Output formats**: `json`, `jsonl`, `yaml`, `csv`, `human`, `explore`
- **Field trimming**: `--fields` for token-budget-conscious agent consumers
- **MCP server**: `dns mcp` exposes a Model Context Protocol surface for LLM agent integration
- **Schema introspection**: `dns schema` emits command definitions for LLM function calling
- **Auto-detection**: `AI_AGENT`, `AGENT`, `CI`, `CLAUDECODE`, `CURSOR_AGENT`, `GEMINI_CLI` env vars force JSON output

### Interactive TUI

Run `dns` without arguments in an interactive terminal to launch the ratatui-based provider/protocol/tier selector.

> The TUI feature is enabled by default. Build with `--no-default-features` for a headless binary (~2.8 MB vs ~3.1 MB).

## Installation

### From Source

```bash
git clone https://github.com/steelbore/flux.git
cd flux
cargo build --release
sudo cp target/release/dns /usr/local/bin/
```

### Prerequisites

- Rust 1.85+ (stable)
- Linux or BSD target
- `sudo`, `pkexec`, or `doas` for privileged config writes

## CI & Compliance

- 101 tests (95 unit + 6 integration)
- `cargo clippy` with `-D warnings`
- `cargo fmt --check`
- `cargo deny check licenses` (OSI/FSF allow-list)
- `cargo audit` (RUSTSEC advisory gating)
- SPDX license headers on all source files
- GPL-3.0-or-later

## Links

- Repository: https://github.com/steelbore/flux
- Website: https://Flux.Steelbore.com/
- PRD: [`Flux_PRD_v0.1.0.md`](./Flux_PRD_v0.1.0.md)
- Changelog: [`CHANGELOG.md`](./CHANGELOG.md)

## Maintainers

Mohamed Hammad <Mohamed.Hammad@Steelbore.com>
