---
name: flux
description: DNS selector and network configurator CLI for Linux and BSD
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@Steelbore.com>
website: https://Flux.Steelbore.com/
---

# Flux Skill

## Capability Surface

The `dns` CLI provides the following capabilities:

### DNS Management
- `dns apply <provider> [tier] <protocol>` — Apply DNS configuration
- `dns status` — Show current DNS state
- `dns detect` — Detect active DNS backend
- `dns verify` — Test DNS resolution
- `dns backup` — Create backup
- `dns restore` — Restore from backup

### Provider Registry
- `dns list --providers` — List DNS providers
- `dns list --tiers -p <slug>` — List tiers for provider
- `dns list --protocols -p <slug> [-t <tier>]` — List valid protocols

### NTP Configuration
- `dns ntp --provider <slug>` — Configure NTP independently
- `--ntp` flag on `dns apply`

### VPN Orchestration
- `dns vpn connect -p <provider>` — Connect to VPN
- `dns vpn disconnect -p <provider>` — Disconnect from VPN
- `dns vpn status` — Show VPN state

### Agent Surface
- `dns schema` — JSON Schema Draft 2020-12
- `dns describe` — Human + machine manifest
- `dns mcp` — MCP server (stdio)

## Output Modes

- `--json` / `--format json` — JSON envelope
- `--format jsonl` — Streaming JSON Lines
- `--format yaml` — YAML
- `--format csv` — CSV
- `--format explore` / no subcommand — TUI

## Environment Variables

- `AI_AGENT=1` — Force JSON, no color, no TUI
- `AGENT=1` — Same as AI_AGENT
- `CI=true` — JSON mode
- `NO_COLOR` — Disable ANSI color
- `FORCE_COLOR` — Force ANSI color

## Exit Codes

| Code | Meaning |
|:----:|---------|
| 0 | Success |
| 1 | General failure |
| 2 | Usage error |
| 3 | Detection failed |
| 4 | Elevation failed |
| 5 | Conflict |
| 6 | Apply failure |
| 7 | Verification failure |
| 8 | VPN error |
| 9 | Registry fetch error (v0.2+) |
