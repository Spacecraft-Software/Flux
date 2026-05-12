# Flux

**DNS Selector & Network Configurator** for Linux and BSD.

Flux detects the active DNS/NTP subsystem, applies encrypted DNS (DoT/DoH/DoQ/DNSCrypt), optionally configures NTP, and orchestrates VPN clients (Cloudflare WARP, AdGuard VPN CLI). Part of Project Steelbore.

## Quick Start

```sh
# Apply Cloudflare Family DNS over DoT
dns apply cloudflare family dot

# Same, machine-readable
dns apply cloudflare family dot --json

# Show current state
dns status --json

# List providers
dns list --providers --json

# Detect backend
dns detect --json
```

## Supported Operating Systems

- Linux: Arch, Debian/Ubuntu, Fedora, openSUSE, NixOS
- BSD: FreeBSD, OpenBSD, NetBSD

## Supported DNS Providers

| Provider | Plain | DoT | DoH | DoQ | DNSCrypt | VPN |
|----------|:-----:|:---:|:---:|:---:|:--------:|:---:|
| Google | ✓ | ✓ | ✓ | — | — | — |
| Cloudflare | ✓ | ✓ | ✓ | — | — | WARP |
| AdGuard | ✓ | ✓ | ✓ | ✓ | ✓ | AG VPN |
| Quad9 | ✓ | ✓ | ✓ | — | ✓ | — |
| OpenDNS | ✓ | — | ✓ | — | — | — |

## Build

```sh
cargo build --workspace --release
```

## Test

```sh
cargo test --workspace
```

## Project Posture

**Personal / Hobby.** Audience: maintainer's own use case. Pace: hobby pace, no service-level commitments. Warranty: none, provided AS IS. Liability: none, see `NOTICE.md`. License: GPL-3.0-or-later governs binding terms. Maintainer discretion applies to PR acceptance, scope, and roadmap.

## Maintainer

- **Mohamed Hammad** &lt;Mohamed.Hammad@Steelbore.com&gt;
- **Project URL:** https://Flux.Steelbore.com/
- **License:** GPL-3.0-or-later

*--- Forged in Steelbore ---*
