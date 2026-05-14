<!--
  Spacecraft Software Document — GFM Companion
  Source format: .docx (MS Office; secondary per spacecraft-document-format §1)
  Sibling: Flux_PRD_v0.1.0.docx (same basename, same directory)
  Palette: Spacecraft Software Standard v1.2 §9 (Void Navy + 5 accents)
  Typography: Share Tech Mono (headings) + Inconsolata (body) — Standard v1.2 §10
  License: GPL-3.0-or-later (per project root; SPDX header rule does not apply to documents — Standard v1.2 §4)
  Authority: If this .md and the .docx disagree, this .md wins on regeneration (skill §2).
-->

# FLUX — DNS Selector & Network Configurator

**Product Requirements Document**
A Spacecraft Software

| Field | Value |
|-------|-------|
| Product Version | 0.1.0 (MVP) |
| PRD Revision | 2026-05-12 |
| Standard | The Spacecraft Software Standard v1.2 (2026-05-11) |
| License | GPL-3.0-or-later |
| Maintainer | Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt; |
| Project URL | https://Flux.SpacecraftSoftware.org/ |

---

## Table of Contents

1. [Preamble & Compliance Statement](#1--preamble--compliance-statement)
   - 1.1 [Naming Justification (Standard v1.2)](#11--naming-justification-standard-v12)
   - 1.2 [Scope Beyond DNS](#12--scope-beyond-dns)
   - 1.3 [Project Posture](#13--project-posture)
2. [Problem Statement](#2--problem-statement)
3. [Architecture Overview](#3--architecture-overview)
4. [MVP Scope (v0.1.0)](#4--mvp-scope-v010)
5. [DNS Provider Registry](#5--dns-provider-registry)
6. [NTP / Time Server Configuration](#6--ntp--time-server-configuration)
7. [System Detection Engine](#7--system-detection-engine)
8. [OS Support Matrix](#8--os-support-matrix)
9. [CLI Interface Specification](#9--cli-interface-specification)
10. [Agent-Facing UX](#10--agent-facing-ux)
11. [TUI Interface Specification](#11--tui-interface-specification)
12. [Security, PQC & Spacecraft Software Compliance](#12--security-pqc--spacecraft-software-compliance)
13. [Attribution & Maintainership](#13--attribution--maintainership)
14. [Future Scope](#14--future-scope)
15. [Compliance Audit Gate](#15--compliance-audit-gate)
16. [Normative References](#16--normative-references)

---

## 1 — Preamble & Compliance Statement

**FLUX** is a DNS selector and network configurator built under the Spacecraft Software project umbrella. This PRD governs the design, scope, and engineering requirements for Flux v0.1.0 (MVP). All requirements herein comply with:

- **The Spacecraft Software Standard v1.2** (2026-05-11) — the master engineering standard.
- **Spacecraft Software Dual-Mode Self-Documenting CLI Framework (SFRS v1.0.0)** — structural CLI rules.
- **Spacecraft Software Agentic CLI Standard v1.0.0** — agent-facing UX layer.
- **Spacecraft Software Document Format** — ODF/MS Office authoring rules with mandatory GFM Markdown companion.

Where this PRD is silent, those standards prevail. Where they conflict, the Spacecraft Software Standard (master) takes precedence.

The CLI binary is named `dns`, providing a memorable command-line entry point. **Flux** is the project codename used in documentation, repository, and packaging. A `flux` shell alias is installed for discoverability.

### 1.1 — Naming Justification (Standard v1.2)

Standard v1.2 §2 replaced the legacy metallurgical naming convention with an **Aerospace, Sci-Fi & AI** naming convention. The name **Flux** satisfies the v1.2 convention through its established aerospace and astronomy meanings:

- **Solar flux** — the radiant energy received from the Sun, measured in watts per square metre. Fundamental to spacecraft thermal design and astrophysical measurement.
- **Magnetic flux** — central to magnetoplasmadynamic and ion-propulsion systems used in deep-space missions.
- **Photon / particle flux** — the directed flow of quanta through a region of space; the natural metaphor for routing DNS query traffic.

Flux is listed in the Standard's §13.1 subdomain table as an active project. The aerospace meaning supersedes (without displacing) any prior metallurgical reading. The name passes the v1.2 test — it would sit naturally on the hull of a spacecraft.

### 1.2 — Scope Beyond DNS

Beyond DNS configuration, Flux provides optional orchestration for two related network services:

- **NTP / Time Server Configuration** — optionally configure the system's NTP source alongside DNS (e.g., `time.cloudflare.com` paired with Cloudflare DNS). Accurate time is a TLS validation prerequisite for DoT/DoH/DoQ.
- **CLI VPN Orchestration** — optionally set up Cloudflare WARP (`warp-cli`) and AdGuard VPN CLI (`adguardvpn-cli`).

Both features are opt-in. No state mutation occurs without explicit user confirmation.

### 1.3 — Project Posture

Per Standard v1.2 §5, Flux's default posture is **Personal / Hobby**. Audience: maintainer's own use case. Pace: hobby pace, no service-level commitments. Warranty: none, provided AS IS. Liability: none, see `NOTICE.md`. License: GPL-3.0-or-later governs binding terms. Maintainer discretion (§5.4) applies to PR acceptance, scope, and roadmap. The general-use carve-out (§5.3) is **not** declared for v0.1.0; reconsidered post-MVP based on adoption.

Required posture files (§5.2) ship at the repository root: `README.md`, `NOTICE.md`, `CONTRIBUTING.md`, `LICENSE`.

---

## 2 — Problem Statement

Configuring DNS on Unix-like systems is unreasonably fragmented. A single system may have its DNS managed by any combination of `/etc/resolv.conf`, `systemd-resolved`, NetworkManager (`nmcli`), `resolvectl`, distribution-specific declarative configuration (NixOS), or BSD resolvconf variants. Each mechanism has different file locations, syntaxes, precedence rules, and persistence behaviours. Switching to a privacy-respecting, encrypted DNS provider typically requires the user to:

1. Identify which DNS management subsystem is active on their distribution.
2. Look up the provider's IP addresses, DoH URLs, or DoT hostnames.
3. Determine which protocol is supported by the active subsystem.
4. Edit configuration files or run commands with root privileges.
5. Verify the change took effect and persists across reboots.

Separately, NTP server selection and VPN tunnel management face similar fragmentation. Flux unifies DNS, NTP, and VPN orchestration behind one intelligent command-line tool with dual human / agent output modes.

---

## 3 — Architecture Overview

Flux is structured as a layered system with clean separation between user interface, orchestration logic, and system-level backends.

### 3.1 — Layer Model

| Layer | Responsibility | MVP Scope |
|-------|----------------|-----------|
| Interface Layer | TUI (primary), CLI (dual-mode), GUI (future) | TUI + CLI |
| Agent Surface | JSON envelope, schema/describe, tips-thinking errors, MCP server | Full SFRS |
| Orchestration Layer | Provider registry, protocol negotiation, backup/restore, NTP, VPN | Full |
| Detection Engine | Probe active DNS/NTP subsystem; deterministic backend selection | Full |
| Backend Adapters | Write configs for resolved, nmcli, resolv.conf, NixOS, BSDs | Linux + BSD |
| Privilege Layer | Auto-elevate via sudo/pkexec/doas; scoped child process | All three |

### 3.2 — Core Principles

- **Detect, don't assume.** Flux probes the running system before making changes. No hardcoded configuration paths.
- **Backup before mutate.** Every `dns apply` creates a UTC-timestamped backup. `dns restore` reverts.
- **Validate after apply.** Post-apply verification query confirms the new resolver is reachable.
- **Fail loudly, with runnable hints.** Errors emit structured JSON to stderr with a `hint` field that contains the exact next command (tips-thinking, §9.7).
- **Dual-mode by default.** Every data-returning command supports human and agent output (JSON envelope, §9.6) selected automatically by the output-mode cascade (§9.5).
- **Idempotent.** Same invocation twice, same result. `dns apply` produces the same end state regardless of starting state.

---

## 4 — MVP Scope (v0.1.0)

### 4.1 — In Scope

- TUI interface with full Spacecraft Software v1.2 theming (Void Navy background, six-token palette, Share Tech Mono + Inconsolata, Vim + CUA keybindings).
- CLI binary `dns` with POSIX-compliant flag grammar, positional shorthand, full SFRS §3 global flags, `--json`, `--format`, `--fields`, and structured errors.
- `dns schema` and `dns describe` subcommands for self-documentation (SFRS §2 Rule 4).
- `dns mcp` — lazy-loading MCP server surface (SFRS §2 Rule 8).
- Five DNS providers: Google, Cloudflare, AdGuard, Quad9, OpenDNS. Six protocol families: Plain, DoT, DoH, DoQ, DNSCrypt, WARP.
- Linux: Arch, Debian/Ubuntu, Fedora, openSUSE, NixOS.
- BSDs (priority): FreeBSD, OpenBSD, NetBSD.
- Optional NTP and VPN orchestration (Cloudflare WARP, AdGuard VPN CLI).
- Backup/restore with UTC ISO 8601 timestamps (§12.6).
- Post-apply verification with millisecond RTT readout.
- Repository context files: `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, `CONTRIBUTING.md`, `NOTICE.md`, `README.md`, `LICENSE`.

### 4.2 — Out of Scope (Future Releases)

- GUI interface (planned: GTK4 or Iced; Material Design + Spacecraft Software palette).
- Windows 10/11 support (PowerShell/netsh backend).
- macOS support.
- DNS benchmark / latency comparison.
- Fetchable provider registry (v0.2 — §5.8).
- Runtime DNSCrypt stamp fetching (v0.2/v0.3).
- Custom user-defined providers, profiles, scheduled switching.

---

## 5 — DNS Provider Registry

Flux v0.1.0 ships with a compile-time provider registry. The fetchable model in §5.8 lands in v0.2.

### 5.1 — Provider × Protocol Compatibility Matrix

| Provider | Plain | DoT | DoH | DoQ | DNSCrypt | VPN |
|----------|:-----:|:---:|:---:|:---:|:--------:|:---:|
| Google | ✓ | ✓ | ✓ | — | — | — |
| Cloudflare | ✓ | ✓ | ✓ | — | — | WARP |
| Cloudflare (Malware) | ✓ | ✓ | ✓ | — | — | WARP |
| Cloudflare (Family) | ✓ | ✓ | ✓ | — | — | WARP |
| AdGuard (Default) | ✓ | ✓ | ✓ | ✓ | ✓ | AG VPN† |
| AdGuard (Family) | ✓ | ✓ | ✓ | ✓ | ✓ | AG VPN† |
| AdGuard (Unfiltered) | ✓ | ✓ | ✓ | ✓ | ✓ | AG VPN† |
| Quad9 (Secured) | ✓ | ✓ | ✓ | — | ✓ | — |
| Quad9 (Secured+ECS) | ✓ | ✓ | ✓ | — | ✓ | — |
| Quad9 (Unsecured) | ✓ | ✓ | ✓ | — | ✓ | — |
| OpenDNS (Standard) | ✓ | — | ✓ | — | — | — |
| OpenDNS (FamilyShield) | ✓ | — | ✓ | — | — | — |

† AdGuard VPN CLI — separate VPN product. Orchestrated, not bundled. See §5.7.

### 5.2 — Google Public DNS

| Property | Primary | Secondary |
|----------|---------|-----------|
| IPv4 | `8.8.8.8` | `8.8.4.4` |
| IPv6 | `2001:4860:4860::8888` | `2001:4860:4860::8844` |
| DoT Host | `dns.google` | `dns.google` |
| DoH URL | `https://dns.google/dns-query` | `https://dns.google/dns-query` |

Plain DNS, DoT (port 853), DoH. Companion NTP: `time.google.com`.

### 5.3 — Cloudflare DNS

| Tier | IPv4 (Pri / Sec) | IPv6 (Pri / Sec) | DoT Hostname | DoH URL |
|------|------------------|------------------|--------------|---------|
| Standard | `1.1.1.1` / `1.0.0.1` | `2606:4700:4700::1111` / `::1001` | `one.one.one.one` | `https://cloudflare-dns.com/dns-query` |
| Malware | `1.1.1.2` / `1.0.0.2` | `::1112` / `::1002` | `security.cloudflare-dns.com` | `https://security.cloudflare-dns.com/dns-query` |
| Family | `1.1.1.3` / `1.0.0.3` | `::1113` / `::1003` | `family.cloudflare-dns.com` | `https://family.cloudflare-dns.com/dns-query` |

Companion NTP: `time.cloudflare.com`. WARP integration: §5.7.1.

### 5.4 — AdGuard DNS

The only provider supporting all six DNS protocol families.

| Tier | IPv4 (Pri / Sec) | IPv6 (Pri / Sec) | DoH URL | DoT / DoQ Host |
|------|------------------|------------------|---------|----------------|
| Default | `94.140.14.14` / `94.140.15.15` | `2a10:50c0::ad1:ff` / `::ad2:ff` | `https://dns.adguard-dns.com/dns-query` | `dns.adguard-dns.com` |
| Family | `94.140.14.15` / `94.140.15.16` | `::bad1:ff` / `::bad2:ff` | `https://family.adguard-dns.com/dns-query` | `family.adguard-dns.com` |
| Unfiltered | `94.140.14.140` / `94.140.14.141` | `::1:ff` / `::2:ff` | `https://unfiltered.adguard-dns.com/dns-query` | `unfiltered.adguard-dns.com` |

DoQ on port 853 (`quic://`). DNSCrypt stamps compiled in v0.1.0; fetchable in v0.2 (§5.8). AdGuard VPN CLI: §5.7.2.

### 5.5 — Quad9

| Tier | IPv4 (Pri / Sec) | IPv6 (Pri / Sec) | DoH URL | DoT Host |
|------|------------------|------------------|---------|----------|
| Secured | `9.9.9.9` / `149.112.112.112` | `2620:fe::fe` / `2620:fe::9` | `https://dns.quad9.net/dns-query` | `dns.quad9.net` |
| Secured+ECS | `9.9.9.11` / `149.112.112.11` | `2620:fe::11` / `2620:fe::fe:11` | `https://dns11.quad9.net/dns-query` | `dns11.quad9.net` |
| Unsecured | `9.9.9.10` / `149.112.112.10` | `2620:fe::10` / `2620:fe::fe:10` | `https://dns10.quad9.net/dns-query` | `dns10.quad9.net` |

DNSCrypt stamps published at `quad9.net/quad9-resolvers.toml`.

### 5.6 — OpenDNS (Cisco)

| Tier | IPv4 (Pri / Sec) | IPv6 (Pri / Sec) | DoH URL |
|------|------------------|------------------|---------|
| Standard | `208.67.222.222` / `208.67.220.220` | `2620:119:35::35` / `2620:119:53::53` | `https://doh.opendns.com/dns-query` |
| FamilyShield | `208.67.222.123` / `208.67.220.123` | `2620:119:35::123` / `2620:119:53::123` | `https://doh.familyshield.opendns.com/dns-query` |

### 5.7 — CLI VPN Orchestration

VPN clients are external tools that Flux orchestrates. Flux never links, bundles, or reimplements VPN logic.

#### 5.7.1 — Cloudflare WARP

- Requires `warp-cli` from the `cloudflare-warp` package.
- Flux wraps `warp-cli register`, `connect`, `disconnect`, `status`, `set-license` (for WARP+).

#### 5.7.2 — AdGuard VPN CLI

- Requires `adguardvpn-cli`. Not FOSS — Flux orchestrates only, never bundles or links.
- Flux wraps `login`, `connect [--location <city>]`, `disconnect`, `status`. Supports HTTP2/QUIC protocol choice and TUN/SOCKS5 mode.

#### 5.7.3 — Future VPN Providers

The VPN trait surface is extensible. Future adapters: WireGuard native CLI, Mullvad VPN CLI, ProtonVPN CLI.

### 5.8 — Fetchable Provider Registry (v0.2 / v0.3)

- **Registry file:** signed TOML at `https://raw.githubusercontent.com/Spacecraft-Software/flux/main/registry.toml`, versioned, GPG-signed.
- **Update:** `dns update-registry` fetches, verifies signature, stores at `~/.local/share/flux/registry.toml`. Compiled-in registry is fallback.
- **DNSCrypt stamps:** fetched on first user-initiated run (v0.2/v0.3) rather than build time.
- **Privacy:** user-initiated only. No automatic background fetching. Consistent with PFA policy (§12.4).

---

## 6 — NTP / Time Server Configuration

Accurate time is a security dependency for TLS validation (DoT/DoH/DoQ). Flux optionally configures NTP alongside DNS.

| Provider | Recommended NTP | Notes |
|----------|-----------------|-------|
| Cloudflare | `time.cloudflare.com` | Anycast NTP, same infrastructure as 1.1.1.1 |
| Google | `time.google.com` | Leap-second smearing |
| AdGuard | `pool.ntp.org` | No public NTP from AdGuard |
| Quad9 | `pool.ntp.org` | No public NTP from Quad9 |
| OpenDNS | `pool.ntp.org` | No public NTP from OpenDNS |

**Backends:** `systemd-timesyncd`, `chrony`, `ntpd`, `openntpd` (common on BSDs), NixOS declarative. Detection follows the same priority approach as DNS detection.

**CLI:** `--ntp` flag on `dns apply` (opt-in). Standalone `dns ntp -p <provider>` subcommand for NTP-only configuration.

---

## 7 — System Detection Engine

### 7.1 — Detection Priority Order

| Priority | Backend | Detection Method | Configuration Target |
|:--------:|---------|------------------|----------------------|
| 1 | NixOS declarative | `/etc/NIXOS` marker file | Generate Nix expression for configuration.nix |
| 2 | BSD | `uname -s` reports FreeBSD/OpenBSD/NetBSD | `resolv.conf` + `unbound`/`unwind`/`local_unbound` |
| 3 | systemd-resolved | `systemctl is-active systemd-resolved` | `/etc/systemd/resolved.conf.d/flux.conf` |
| 4 | NetworkManager | `systemctl is-active NetworkManager` + `nmcli` | `nmcli connection modify` |
| 5 | resolvectl | `resolvectl status` exits 0 | `resolvectl dns / dot / etc.` |
| 6 | /etc/resolv.conf | File exists, not symlinked to `127.0.0.53` | Direct write with immutable check |

### 7.2 — BSD Support (Priority Requirement)

- **FreeBSD:** `local_unbound(8)` configured as forwarder for encrypted DNS. NTP via `ntpd`/`openntpd`. Elevation via `doas` or `sudo`.
- **OpenBSD:** `unwind(8)` or `unbound(8)` for encrypted DNS. `openntpd` (ships by default). Elevation via `doas` (native).
- **NetBSD:** `unbound` from pkgsrc. `ntpd`. Elevation via `sudo` or `doas`.

On BSDs requiring a local forwarder for encrypted DNS, Flux configures the forwarder and then points `/etc/resolv.conf` at `127.0.0.1`.

### 7.3 — NixOS Special Handling

NixOS is declarative. Flux generates a Nix expression fragment for the user to insert into `configuration.nix`; no imperative writes. Output to stdout with optional clipboard copy.

### 7.4 — Protocol → Backend Constraints

| Protocol | resolv.conf | systemd-resolved | nmcli | NixOS | BSD |
|----------|:-----------:|:----------------:|:-----:|:-----:|:---:|
| Plain | ✓ | ✓ | ✓ | ✓ | ✓ |
| DoT | — | ✓ | — | ✓ | ✓ |
| DoH | — | ✓ (v250+) | — | ✓ | ✓ |
| DoQ | — | Via stub | — | Via stub | Via stub |
| DNSCrypt | — | Via stub | — | Via stub | Via stub |
| WARP | warp-cli | warp-cli | warp-cli | warp-cli | warp-cli |

For DoQ/DNSCrypt without native backend support, Flux recommends and optionally installs `dnsproxy` or `dnscrypt-proxy` as a local forwarder.

---

## 8 — OS Support Matrix

### 8.1 — Phase 1: Linux + BSD (v0.1.0)

| OS | Primary Backend | Package Manager | Elevation | WARP Source |
|----|-----------------|-----------------|-----------|-------------|
| Arch Linux | systemd-resolved | pacman | sudo | AUR (`cloudflare-warp-bin`) |
| Debian/Ubuntu | systemd-resolved / NM | apt | sudo/pkexec | Cloudflare APT repo |
| Fedora | systemd-resolved / NM | dnf | sudo/pkexec | Cloudflare RPM repo |
| openSUSE | NetworkManager | zypper | sudo | Cloudflare RPM repo |
| NixOS | Declarative Nix | nix | sudo | nixpkgs (`cloudflare-warp`) |
| FreeBSD | resolv.conf + `local_unbound` | pkg | doas/sudo | N/A |
| OpenBSD | resolv.conf + `unwind` | pkg_add | doas | N/A |
| NetBSD | resolv.conf + `unbound` | pkgsrc | sudo/doas | N/A |

### 8.2 — Phase 2: Windows + macOS (Future)

Windows 10 (Build 19628+) / 11 with native DoH; PowerShell + netsh backend. macOS via `scutil` and configuration profiles. Both out of scope for v0.1.0.

---

## 9 — CLI Interface Specification

The Flux CLI is a Dual-Mode Self-Documenting CLI per the Spacecraft Software SFRS v1.0.0. It serves two co-equal readers: humans in interactive terminals and AI agents paying for tokens. Both modes are tuned independently; neither subsidizes the other.

### 9.1 — Command Grammar & POSIX Compliance

The CLI adheres to POSIX utility conventions (IEEE Std 1003.1). Two equivalent grammars are supported:

**Canonical (flagged):**

```sh
dns <subcommand> [options] [operands]
```

**Positional shorthand (apply only):**

```sh
dns apply <provider> [tier] <protocol>
```

POSIX permits operands after options. The shorthand maps strictly: first operand = provider, optional second = tier, third = protocol. Forms are interchangeable and produce identical internal state.

```sh
dns apply cloudflare family dot
dns apply -p cloudflare -t family -P dot
dns apply --provider cloudflare --tier family --protocol dot
```

### 9.2 — Subcommands

| Subcommand | Purpose | Verb Class |
|------------|---------|------------|
| `apply` | Apply DNS (+ optional NTP/VPN) configuration; idempotent. | apply |
| `status` | Show current DNS / NTP / VPN state. | get |
| `list` | List providers / tiers / protocols / VPN clients. | list |
| `restore` | Revert to most recent backup. | apply |
| `verify` | Test current DNS resolution. | get |
| `detect` | Display detected DNS/NTP backend info. | describe |
| `backup` | Create a backup of current state. | create |
| `ntp` | Configure NTP independently. | apply |
| `vpn` | Manage VPN (connect / disconnect / status). | apply |
| `schema` | Emit JSON Schema (Draft 2020-12) for the entire CLI. | describe |
| `describe` | Emit human + machine manifest of the CLI surface. | describe |
| `mcp` | Launch the MCP server (stdio transport). | (server) |
| `update-registry` | Fetch latest provider registry (v0.2+). | sync |

Standard verbs follow SFRS §2 Rule 7: `list`, `get`, `create`, `update`, `delete`, `apply`, `sync`, `describe`, `schema`.

### 9.3 — Global Flags (SFRS §3)

Every Spacecraft Software CLI accepts these flags with identical semantics. Divergence is a BLOCKER.

| Flag | Effect |
|------|--------|
| `--json` | Alias for `--format json`. |
| `--format <fmt>` | One of: `json`, `jsonl`, `yaml`, `csv`, `explore`. |
| `--fields <list>` | Comma-separated field selection. Reduces token cost for agents. |
| `--dry-run` | Emit action plan as JSON; no side effects. Required on every write command. |
| `-v` / `--verbose` | Diagnostic output to stderr (never stdout). |
| `-q` / `--quiet` | Suppress non-error stderr. |
| `--no-color` | Disable ANSI color. Equivalent to `--color=never`. |
| `--color <when>` | `never` / `always` / `auto`. |
| `-h` / `--help` | Help text with ≥2 examples per subcommand; one demonstrating `--json`. |
| `-V` / `--version` | Version + build info. Includes maintainer line and project URL (§13). |
| `--absolute-time` | Disable relative-time rendering in human mode. Data is always UTC ISO 8601 with Z. |
| `-0` / `--print0` | NUL-delimited output for `xargs -0` piping. |
| `--yes` / `--force` | Skip confirmation in non-TTY mode (required for destructive ops). |

### 9.4 — Apply Subcommand Flags

| Short | Long | Values | Description |
|-------|------|--------|-------------|
| `-p` | `--provider` | google, cloudflare, adguard, quad9, opendns | DNS provider. |
| `-t` | `--tier` | standard, malware, family, unfiltered, ecs, unsecured, secured | Filtering tier. |
| `-P` | `--protocol` | plain, dot, doh, doq, dnscrypt, warp | Transport protocol. |
| `-4` | `--ipv4-only` | — | Use only IPv4 addresses. |
| `-6` | `--ipv6-only` | — | Use only IPv6 addresses. |
|  | `--ntp` | — | Also configure NTP. |
|  | `--vpn` | warp, adguard | Also set up VPN client. |
|  | `--no-backup` | — | Skip backup creation. |
|  | `--no-verify` | — | Skip post-apply verification. |

### 9.5 — Output Mode Detection Cascade

Per SFRS §5, the first matching condition wins:

1. **Explicit flag.** `--format <fmt>` or `--json` forces that mode unconditionally.
2. **Agent env var.** `AI_AGENT=1`, `AGENT=1`, or `CI=true` → JSON mode, no color, no TUI, non-interactive (`--yes` implicit), minimal verbosity.
3. **TTY.** `isatty(stdout) == true` → human mode with color.
4. **Non-TTY.** stdout piped/redirected → JSON mode.
5. **Fallback.** → human mode.

`CLAUDECODE`, `CURSOR_AGENT`, `GEMINI_CLI` are **informational only** — they appear in `metadata.invoking_agent` but never override output format. `TERM=dumb` suppresses color and TUI but not necessarily JSON mode.

**Explore mode constraint:** if `AI_AGENT=1` is set and `--format explore` is requested, the TUI MUST NOT activate — fall back to JSON and warn on stderr. Never trap an agent in an interactive render loop.

### 9.6 — JSON Output Envelope

Every `--json` response is a single valid JSON document. snake_case property names; no trailing commas, comments, or BOM; no ANSI escapes; no log lines interleaved.

```json
{
  "metadata": {
    "tool": "flux",
    "version": "0.1.0",
    "command": "dns apply",
    "timestamp": "2026-05-12T14:30:00Z",
    "invoking_agent": "claude-code",
    "maintainer": "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
    "website": "https://Flux.SpacecraftSoftware.org/"
  },
  "data": {
    "provider": "cloudflare",
    "tier": "family",
    "protocol": "dot"
  }
}
```

**Hard rules:** stdout = data only. stderr = everything else (progress, logs, banners, errors). UTF-8 without BOM. UTC ISO 8601 timestamps with mandatory `Z` suffix. Null fields omitted (`serde(skip_serializing_if = "Option::is_none")`).

### 9.7 — Structured Errors & Tips-Thinking

Every non-zero exit in machine mode emits a structured error to stderr. The `hint` field is a **runnable command**, never prose.

```json
{
  "error": {
    "code": "DETECTION_FAILED",
    "exit_code": 3,
    "message": "No supported DNS subsystem detected on this system",
    "hint": "dns detect --json",
    "timestamp": "2026-05-12T14:30:00Z",
    "command": "dns apply cloudflare family dot",
    "docs_url": "https://Flux.SpacecraftSoftware.org/errors/DETECTION_FAILED"
  }
}
```

**Tips-thinking rule:** "Run `dns detect --json`" is the hint. "Use the detect command to see backends" is **wrong** — the agent must be able to *execute* the hint without further inference.

### 9.8 — Exit Codes (Canonical Map)

Per SFRS §4. Codes 0–5 are reserved by the canonical map; 6–125 are tool-specific and MUST be enumerated in `dns schema` output.

| Code | Meaning | Class | Tips-thinking hint |
|:----:|---------|-------|--------------------|
| 0 | Success | Canonical | — |
| 1 | General failure | Canonical | `dns --help <subcommand>` |
| 2 | Usage error (bad arguments) | Canonical | `dns <subcommand> --help` |
| 3 | Resource not found (`DETECTION_FAILED`) | Canonical | `dns detect --json` |
| 4 | Permission denied (`ELEVATION_FAILED`) | Canonical | `sudo dns apply ...` |
| 5 | Conflict (DNS already managed) | Canonical | `dns status --json` |
| 6 | Apply failure (write failed) | Tool-specific | `dns restore && dns detect` |
| 7 | Verification failure (DNS query failed) | Tool-specific | `dns restore` |
| 8 | VPN error (`warp-cli` / `adguardvpn-cli`) | Tool-specific | `dns vpn status --json` |
| 9 | Registry fetch error (v0.2+) | Tool-specific | `dns update-registry --verbose` |
| 126 | Command not executable | POSIX | `ls -l $(which dns)` |
| 127 | Command not found | POSIX | `which dns` |
| 128+N | Fatal signal N | POSIX | — |

### 9.9 — Usage Examples

```sh
# Apply Cloudflare Family DNS over DoT
dns apply cloudflare family dot

# Same, machine-readable
dns apply cloudflare family dot --json

# Streaming list of providers (jsonl, one record per line)
dns list --providers --format jsonl

# Reduce payload to just the fields an agent needs
dns list --providers --json --fields slug,protocols

# Emit JSON Schema for LLM function-calling
dns schema --json

# Apply with NTP + WARP, IPv4 only
dns apply cloudflare standard dot --ntp --vpn warp -4

# Dry-run shows action plan as JSON
dns apply quad9 doh --dry-run --json

# Launch the MCP server (stdio transport for Claude Code)
dns mcp
```

---

## 10 — Agent-Facing UX

Per the Spacecraft Software Agentic CLI Standard v1.0.0, Flux is designed for two co-equal readers from day one.

### 10.1 — Two-Readers Model

| Reader | Optimizes for | Reads via |
|--------|---------------|-----------|
| Human in a terminal | Discoverability, forgiveness, visual scanning | TTY with color, tables, TUI |
| AI agent paying tokens | Predictability, schema stability, token frugality | Pipes, JSON, schema, MCP |

The two are orthogonal, not opposite. The same command renders twice. When they trade off, agent-mode optimizes for the agent's constraints, and human-mode optimizes for the human's constraints.

### 10.2 — Repository Context Files

Context files are runtime configuration for agents, not documentation. These four ship at the repository root before any business logic is written:

| File | Primary Consumer | Contents |
|------|------------------|----------|
| `AGENTS.md` | Generic agents (Codex, Cursor, Aider, OpenCode) | Coding conventions, build/test commands, repo invariants, forbidden patterns |
| `CLAUDE.md` | Claude Code | AGENTS.md content + Claude-specific skills, MCP servers, tool preferences |
| `SKILL.md` | Spacecraft Software Skills + CLI-Anything | YAML frontmatter + capability surface of the CLI |
| `CONTRIBUTING.md` | Human contributors | Onboarding, dev env, PR conventions, sign-off, security reporting |

Plus the posture files (§1.3): `README.md`, `NOTICE.md`, `LICENSE`.

### 10.3 — Agent Environment Detection

| Variable | Format | Color | TUI | Interactivity | Verbosity |
|----------|--------|-------|-----|---------------|-----------|
| `AI_AGENT=1` | json | off | suppressed | non-interactive (implicit `--yes`) | minimal (failures only) |
| `AGENT=1` | json | off | suppressed | non-interactive | minimal |
| `CI=true` | json | off | suppressed | non-interactive | normal |
| `CLAUDECODE=1` | info | (other) | (other) | (other) | (other) |
| `CURSOR_AGENT=1` | info | (other) | (other) | (other) | (other) |
| `GEMINI_CLI=1` | info | (other) | (other) | (other) | (other) |
| `TERM=dumb` | (other) | off | suppressed | (other) | (other) |

### 10.4 — MCP Surface (dns mcp)

Per SFRS §2 Rule 8, Flux exposes an MCP (Model Context Protocol) server for the agent function-calling surface. The MCP server uses **lazy schema loading** per Agentic CLI Standard §6.

- **tools/list response:** names + one-line descriptions + capability tags (`read`, `write`, `destructive`). Full schemas are NOT included.
- **tools/get response:** loaded only when the agent requests a specific tool. Full input/output JSON Schema returned.
- **Transport:** stdio by default (`dns mcp`). Streamable-HTTP optional via `dns mcp --transport http --port <p>`.
- **Lazy loading rationale:** naive MCP servers dump 50–150 KB of schema on connection. Lazy loading mirrors Claude Code's deferred injection and Cursor's tool-search (reported 46.9% token reduction).

### 10.5 — Token-Economy Hygiene

- `--fields` honored on every list/get command. Default JSON output is compact (no pretty-printing) when stdout is non-TTY.
- `--format jsonl` available for streaming list commands.
- Error hints are runnable commands (§9.7), not prose.
- `dns schema` emits Anthropic-format JSON Schema by default. Drops directly into Claude function-calling without translation.
- Timestamps as ISO 8601 strings (`"2026-05-12T14:30:00Z"`, 22 chars) not object encodings.
- Null fields omitted in JSON output via `serde(skip_serializing_if = "Option::is_none")`.

---

## 11 — TUI Interface Specification

The TUI is launched via `dns` (no subcommand) or `dns --format explore`. Recommended library: `ratatui` + `crossterm`.

### 11.1 — Spacecraft Software v1.2 Palette

| Token | Hex | TUI Role |
|-------|-----|----------|
| Void Navy | `#000027` | Background (mandatory) |
| Molten Amber | `#D98E32` | Body text, active readout |
| Steel Blue | `#4B7EB0` | Headers, primary accent, focused widgets |
| Radium Green | `#50FA7B` | Success, selected items |
| Liquid Coolant | `#8BE9FD` | Info, hints, links |
| Red Oxide | `#FF5C5C` | Warning, error |

**Removed in v1.2:** Steel Orange (`#FE6B00`) is no longer a Spacecraft Software palette token. Earlier Flux drafts referencing it are obsolete.

### 11.2 — Screen Flow

| Screen | Purpose | Navigation |
|--------|---------|------------|
| Main Menu | Apply DNS / NTP / VPN / Status / Restore / Detect / Quit | `j`/`k`, Enter |
| Provider Select | Choose provider | `j`/`k`, Enter |
| Tier Select | Valid tiers only | `j`/`k`, Enter |
| Protocol Select | Valid protocols for provider + detected backend | `j`/`k`, Enter |
| Options | Toggle NTP, VPN, IPv4/IPv6-only | Space |
| Confirmation | Summary; confirm / cancel | `y`/`n`, Enter/Esc |
| Progress | backup → detect → configure → verify | Auto-advance |
| Status | Current DNS / NTP / VPN state | Esc |

### 11.3 — Keybindings (Standard §8)

- **Vim:** `j`/`k` up/down, `h`/`l` left/right, `g`/`G` top/bottom, `/` filter, `q` quit.
- **CUA:** `Ctrl+C` cancel, `Ctrl+Z` undo (= restore), `Tab`/`Shift+Tab` panels, `Enter` confirm, `Esc` back.

### 11.4 — Graceful Degradation

- `NO_COLOR` and `TERM=dumb` disable color; TUI still functional in plain text.
- `AI_AGENT=1` or `AGENT=1` → TUI suppressed, fall back to JSON mode with stderr warning.
- Non-TTY stdout → TUI never activates regardless of subcommand.

---

## 12 — Security, PQC & Spacecraft Software Compliance

### 12.1 — Priority 1: Memory Safety (Standard §3.1)

- **Rust** — the Spacecraft Software-mandated language. `rust-guidelines` skill loaded before any Rust is written.
- No `unsafe` blocks without a documented safety justification (code comment + entry in `SAFETY.md`).
- `cargo audit` runs on every CI build; any RUSTSEC advisory fails the build.

### 12.2 — Priority 2: Performance (Standard §3.2)

- Release builds: `-C target-cpu=native`, LTO, `opt-level=3`. PGO where applicable.
- Concurrency designed-in from the start (detection probes run in parallel via tokio or rayon).
- Startup target: under 100 ms to first TUI frame. Detection target: under 200 ms.
- Benchmarks (criterion) for the detection engine, provider lookup, and backup serialization.

### 12.3 — Priority 3: Hardened Security + PQC (Standard §3.3)

- ASLR + CFI enabled on all binaries (Rust compiler flags where supported).
- Elevation: parent process never runs as root. Scoped child for the configuration-write step only.
- Configuration payload passed to elevated child via **argv arrays or IPC**, never shell interpolation.
- **PQC readiness:** Flux relies on system TLS stacks for DoH/DoT/DoQ. Where Flux exposes its own TLS choice (e.g., `update-registry` fetch in v0.2+), use rustls with PQC-hybrid support: `X25519MLKEM768` KEM, `ML-DSA-65` signatures where the rustls release supports them. Migration plan documented in `docs/PQC.md`.
- Backups at `~/.local/share/flux/backups/` with mode `0600` (set explicitly after create).

### 12.4 — Privacy-Friendly Application (Standard §7)

- **No Tracking / No Ads:** zero telemetry, analytics, beacons. No outbound network traffic except: (a) the verify query to the user's chosen resolver, (b) `dns update-registry` (v0.2+, user-initiated), (c) external VPN subprocesses.
- **Minimal Permissions:** only essential permissions; requested lazily at point of use. Elevation requested per-operation, not per-session.
- **Local Storage:** all state local. No cloud sync. Backups under `~/.local/share/flux/`.

### 12.5 — Threat Model (Agent-Invoked)

When invoked by an agent, the CLI is the last line of defense before the host system. Per Agentic CLI Standard §7:

- **Path arguments:** canonicalize via `std::fs::canonicalize`; reject `..` sequences, encoded variants, and symlink escapes. Allow-list against expected directories (`/etc/`, `~/.local/share/flux/`).
- **String arguments:** reject control characters (0x00–0x08, 0x0B–0x0C, 0x0E–0x1F) at parse time. Reject ANSI escape sequences in inputs.
- **Numeric arguments:** bounds-check against schema-declared min/max.
- **Destructive operations:** `--yes` or `--force` required in non-TTY mode. Default-deny when interactive prompt is unavailable.
- **Sub-process arguments:** never interpolate user-provided strings into shell commands. Use argv arrays exclusively (`Command::args`, not `Command::arg(shell_string)`).

### 12.6 — Date / Time / Units Compliance (Standard §12)

- **ISO 8601 only:** dates `YYYY-MM-DD`; timestamps `YYYY-MM-DDTHH:MM:SSZ` with **mandatory Z suffix**.
- **Forbidden:** offset notation (`+03:00`, `-05:00`), local time in stored data, AM/PM, `--local-time` flag.
- **Durations:** `PT1H30M` format; prose forms forbidden in machine output.
- **Rust crate:** `jiff` (preferred) or `chrono`; never `time` 0.1.x. No `NaiveDateTime` in serialized output.
- **Units:** metric only. Latency in milliseconds (ms). No imperial in machine output.

### 12.7 — Licensing (Standard §4)

- **GPL-3.0-or-later.** Every Rust source file: `// SPDX-License-Identifier: GPL-3.0-or-later`.
- `license = "GPL-3.0-or-later"` in every `Cargo.toml`.
- Third-party dependencies must be GPL-compatible (MIT, Apache-2.0, BSD permitted). Copyleft-incompatible licenses forbidden.
- **Document files exempt:** the SPDX header rule does NOT apply to `.docx`, `.pdf`, `.odt`, `.md` (Standard §4). License is stated at project root.

---

## 13 — Attribution & Maintainership

Per Standard v1.2 §13, every Spacecraft Software product surfaces the following attribution in `--help`, `--version`, README, and any About/Info screen.

| Field | Value |
|-------|-------|
| Maintainer | Mohamed Hammad |
| Contact | `Mohamed.Hammad@SpacecraftSoftware.org` |
| Copyright | (c) 2026 Mohamed Hammad |
| License | GPL-3.0-or-later |
| Website | https://Flux.SpacecraftSoftware.org/ |

**Per-surface rules:**

- `--version` (human): footer line "Maintained by Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt;" + project URL.
- `--version --json`: include `"maintainer"` and `"website"` in metadata envelope.
- `--help`: footer with project URL and maintainer name.
- `README.md`: "Maintainer" section with name, email, project URL.
- TUI About screen: maintainer, project URL, copyright year.

Contact must always be `Mohamed.Hammad@SpacecraftSoftware.org` — never a personal domain or GitHub handle.

---

## 14 — Future Scope

### 14.1 — GUI Interface (v0.2.0+)

- GTK4 (gtk4-rs) or Iced framework, Material Design + Spacecraft Software palette.
- System tray indicator showing current DNS provider and VPN state.

### 14.2 — Windows + macOS (v0.2.0+)

- Windows: PowerShell + netsh backend, UAC elevation, WARP Windows client.
- macOS: scutil, configuration profiles, sudo/doas.

### 14.3 — Fetchable Registry & Stamps (v0.2 / v0.3)

- `dns update-registry` — GPG-verified TOML from GitHub. See §5.8.
- DNSCrypt stamps fetched at first user-initiated run rather than build time.

### 14.4 — Additional Features

- **DNS Benchmark** — latency / reliability comparison; TUI chart or JSON export.
- **Custom Providers** — user-defined entries in `~/.config/flux/providers.toml`.
- **Profile System** — `dns apply --profile work`.
- **Scheduled Switching** — cron / systemd-timer integration.
- **Additional VPN adapters** — WireGuard, Mullvad, ProtonVPN.

---

## 15 — Compliance Audit Gate

Before tagging any release of Flux, verify each item below. This list is the runtime instantiation of Standard v1.2 §14 for this project.

| Standard § | Requirement | Status |
|------------|-------------|:------:|
| §2 | Naming: Flux satisfies v1.2 aerospace/sci-fi/AI convention (justified §1.1) | ✓ |
| §3.1 | Memory safety: Rust; ASLR + CFI; `cargo-audit` | ✓ |
| §3.2 | Concurrency designed-in; benchmarks present | ✓ |
| §3.3 | Hardened security; PQC migration plan documented | ✓ |
| §4 | GPL-3.0-or-later; SPDX headers on source files (not docs) | ✓ |
| §5 | Posture files present (README, NOTICE, CONTRIBUTING, LICENSE) | ✓ |
| §6.1 | POSIX-compliant CLI | ✓ |
| §7 | PFA: no tracking, minimal permissions, local storage | ✓ |
| §8 | CUA + Vim keybindings in TUI | ✓ |
| §9 | Void Navy background; six-token palette only; no Steel Orange | ✓ |
| §10 | Share Tech Mono + Inconsolata (OFL fonts) | ✓ |
| §11 | Material Design (GUI: future); WCAG 2.1 AA contrast verified | ✓ |
| §12 | ISO 8601 UTC with mandatory Z; ISO 8601 durations; metric units; `jiff`/`chrono` only | ✓ |
| §13 | Maintainer name, email, project URL in `--help` / `--version` / README | ✓ |
| SFRS §1 | Non-negotiables: UTF-8 no-BOM, POSIX output, `--json`, stdout-data-only | ✓ |
| SFRS §4 | Canonical exit codes (§9.8) | ✓ |
| SFRS §5 | Output mode detection cascade (§9.5) | ✓ |
| SFRS §6 | JSON envelope `{ metadata, data }` (§9.6) | ✓ |
| Agentic §2 | `AGENTS.md` / `CLAUDE.md` / `SKILL.md` / `CONTRIBUTING.md` | ✓ |
| Agentic §3 | Tips-thinking error hints | ✓ |
| Agentic §6 | MCP lazy schema loading | ✓ |
| Agentic §7 | Threat model: path canonicalization, control char rejection | ✓ |

---

## 16 — Normative References

| Reference | Description |
|-----------|-------------|
| The Spacecraft Software Standard v1.2 (2026-05-11) | Master engineering standard. Governs all sections. |
| Spacecraft Software SFRS v1.0.0 (2026-04-10) | Dual-Mode Self-Documenting CLI Framework. |
| Spacecraft Software Agentic CLI Standard v1.0.0 (2026-04-10) | Agent-facing UX layer. |
| Spacecraft Software Document Format | ODF/MS Office authoring with GFM Markdown companion. |
| IEEE Std 1003.1 (POSIX) | CLI utility conventions. |
| RFC 1035 | DNS plain protocol. |
| RFC 7858 | DNS over TLS (DoT). |
| RFC 8484 | DNS over HTTPS (DoH). |
| RFC 9250 | DNS over QUIC (DoQ). |
| RFC 5905 | NTPv4. |
| DNSCrypt Protocol v2 | DNSCrypt encrypted DNS. |
| JSON Schema Draft 2020-12 | Schema format for `dns schema`. |
| Model Context Protocol (MCP) | Agent function-calling protocol. |
| WCAG 2.1 Level AA | Accessibility contrast. |
| NIST FIPS 203 / 204 / 205 | Post-quantum standards: ML-KEM, ML-DSA, SLH-DSA. |

---

*--- Forged in Spacecraft Software ---*
