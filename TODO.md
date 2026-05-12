# Flux v0.1.0 — TODO
<!-- SPDX-License-Identifier: GPL-3.0-or-later (does not apply to this document per Standard v1.2 §4; informational only) -->
<!--
  Companion to: Flux_PRD_v0.1.0.docx / Flux_PRD_v0.1.0.md
  Standard: The Steelbore Standard v1.2 (2026-05-11)
  Spec dates: SFRS v1.0.0 + Agentic CLI v1.0.0 (both 2026-04-10)
  Last revision: 2026-05-12
-->

**DNS Selector & Network Configurator — MVP Task List**
A Steelbore Project · Maintainer: Mohamed Hammad &lt;Mohamed.Hammad@Steelbore.com&gt;
Project URL: https://Flux.Steelbore.com/

---

## Legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[!]` Blocked / needs decision

References in `(§N.N)` point to the matching PRD section. **SFRS** = `steelbore-cli-standard`. **ACS** = `steelbore-agentic-cli`. **STD** = `steelbore-standard`. **DOC** = `steelbore-document-format`.

---

## 0 — Project Bootstrap

- [x] Initialise Cargo workspace (`dns` binary crate + internal library crates)
- [x] Set `license = "GPL-3.0-or-later"` in every `Cargo.toml`
- [x] Add `// SPDX-License-Identifier: GPL-3.0-or-later` header template (Rust source files only; documents exempt per STD §4)
- [x] Configure `.cargo/config.toml`: `target-cpu=native`, LTO, `opt-level=3` for release profile
- [x] Set up CI pipeline:
  - [x] `cargo build --release`
  - [x] `cargo test`
  - [x] `cargo clippy -- -D warnings`
  - [ ] `cargo audit` (RUSTSEC fails build)
  - [x] SPDX header lint on `*.rs` only
  - [ ] License-compat lint (`cargo deny check licenses`)
- [x] Establish directory layout:
  ```
  flux/
  ├── src/
  │   ├── main.rs
  │   ├── cli/        # clap wiring, global flags, output cascade
  │   ├── agent/      # JSON envelope, schema, describe, mcp
  │   ├── tui/        # ratatui screens
  │   ├── orchestrator/
  │   ├── detection/
  │   ├── backends/   # resolved, nmcli, resolv.conf, nixos, bsd
  │   ├── registry/
  │   ├── ntp/
  │   ├── vpn/        # warp, adguard
  │   └── privilege/
  ├── docs/
  │   ├── PQC.md
  │   └── SAFETY.md
  ├── Cargo.toml
  ├── README.md       # STD §5.2
  ├── NOTICE.md       # STD §5.2
  ├── CONTRIBUTING.md # STD §5.2 + ACS §2
  ├── AGENTS.md       # ACS §2
  ├── CLAUDE.md       # ACS §2
  ├── SKILL.md        # ACS §2
  ├── LICENSE
  └── TODO.md
  ```

---

## 1 — Repository Posture & Context Files (STD §5, ACS §2)

### 1.1 — Posture Files (STD §5.2)

- [x] `README.md` — install, quick-start, supported OS, "Project Posture" section linking NOTICE.md + CONTRIBUTING.md, attribution block (STD §13.2)
- [x] `NOTICE.md` — no-warranty / no-liability stance; defers to GPL-3.0-or-later for binding terms
- [x] `CONTRIBUTING.md` — contribution scope, PR-acceptance discretion (STD §5.4), DCO sign-off, security reporting, license-of-contributions
- [x] `LICENSE` — verbatim GPL-3.0-or-later text
- [x] Declare posture as **Personal / Hobby** in README (default per STD §5.1; no §5.3 general-use carve-out for v0.1.0)

### 1.2 — Agent Context Files (ACS §2)

- [x] `AGENTS.md` — coding conventions (Rust + Nushell/Ion preferred), `cargo test` / `cargo clippy` commands, repo invariants (`no unsafe without SAFETY.md`, `UTC Z mandatory`, `argv arrays only`), forbidden patterns (shell interpolation, `NaiveDateTime` in output, `time` 0.1.x crate)
- [x] `CLAUDE.md` — AGENTS.md content + Claude-specific: skills loaded (`rust-guidelines`, `steelbore-standard`, `steelbore-cli-standard`, `steelbore-agentic-cli`), MCP servers expected, preferred shells (Nushell + Ion)
- [x] `SKILL.md` — YAML frontmatter (name, description, license, maintainer, website) + capability surface of the `dns` CLI

### 1.3 — Engineering-Reference Files

- [x] `docs/PQC.md` — PQC migration plan (rustls hybrid: `X25519MLKEM768` KEM, `ML-DSA-65` signatures)
- [x] `docs/SAFETY.md` — registry of every documented `unsafe` block with justification

---

## 2 — Provider Registry (§5)

### 2.1 — Data Model

- [x] Define `Provider` struct: name, slug, tiers, protocols, NTP server, notes
- [x] Define `Tier` enum: `Standard`, `Malware`, `Family`, `Unfiltered`, `Ecs`, `Unsecured`, `Secured`
- [x] Define `Protocol` enum: `Plain`, `DoT`, `DoH`, `DoQ`, `DnsCrypt`, `Warp`
- [x] Define `ProviderAddresses` struct: IPv4 primary/secondary, IPv6 primary/secondary, DoT hostname, DoH URL, DoQ URL, DNSCrypt stamp
- [x] `serde(skip_serializing_if = "Option::is_none")` on every optional field (ACS §8 token economy)
- [x] Implement provider × protocol compatibility matrix enforcement (§5.1)

### 2.2 — Compile-time Registry Entries

- [x] **Google Public DNS** (§5.2) — Plain, DoT, DoH; NTP `time.google.com`; no tiers/DoQ/DNSCrypt/WARP
- [x] **Cloudflare DNS** (§5.3) — Standard/Malware/Family tiers; Plain/DoT/DoH; NTP `time.cloudflare.com`; WARP via `warp-cli`
- [x] **AdGuard DNS** (§5.4) — Default/Family/Unfiltered tiers; all six protocols; compile-in DNSCrypt stamps; AdGuard VPN CLI orchestration
- [x] **Quad9** (§5.5) — Secured/Secured+ECS/Unsecured; Plain/DoT/DoH/DNSCrypt; stamps from `quad9.net/quad9-resolvers.toml`
- [x] **OpenDNS / Cisco** (§5.6) — Standard/FamilyShield; Plain + DoH only

### 2.3 — Registry API

- [x] `registry::get_provider(slug) -> Option<Provider>`
- [x] `registry::list_providers() -> Vec<Provider>`
- [x] `registry::valid_protocols(provider, tier, backend) -> Vec<Protocol>`
- [x] `registry::validate_combination(provider, tier, protocol) -> Result<(), AppError>` (returns structured error, §9.7)

---

## 3 — System Detection Engine (§7)

- [x] Detection priority order per §7.1: NixOS → BSD → systemd-resolved → NetworkManager → resolvectl → /etc/resolv.conf
- [x] BSD detection: `uname -s` → `FreeBSD` / `OpenBSD` / `NetBSD` (§7.2)
- [x] systemd-resolved + version check (v250+ for DoH)
- [x] NetworkManager + `nmcli` PATH check + DNS plugin setting
- [x] resolvectl status check + managed-interface enumeration
- [x] /etc/resolv.conf existence + non-symlink-to-stub check + immutable attr warning
- [~] Conflict resolution per §7.4 (resolved vs NM vs resolvectl precedence)
- [x] Protocol → backend constraint enforcement per §7.4 (DoQ/DNSCrypt → local stub via dnsproxy/dnscrypt-proxy)
- [~] Parallel probe runs via `tokio` or `rayon` (STD §3.2 — concurrency designed-in)
- [x] Export `detect::detect_backend() -> Result<Backend, AppError>`
- [x] Export `detect::detect_ntp_backend() -> Result<NtpBackend, AppError>`
- [~] Target: detection completes in under 200 ms (PRD §12.2)

---

## 4 — Backend Adapters (§8)

### 4.1 — Common Interface

- [x] Define `DnsBackend` trait: `apply()`, `backup()`, `restore()`, `status()`, `verify()`
- [x] Define `BackupRecord` struct: UTC timestamp (`Z` suffix mandatory), backend type, snapshot

### 4.2 — Linux Backends

- [x] systemd-resolved → `[Resolve]` stanza in `resolved.conf.d/flux.conf` with `DNS=`, `DNSOverTLS=`, `DNSSEC=`
- [x] NetworkManager → `nmcli connection modify <con> ipv4.dns ...` + ipv6 + `nmcli connection up`
- [~] resolvectl → `resolvectl dns <iface>`, `resolvectl dot <iface> yes`
- [x] /etc/resolv.conf → write `nameserver` lines; preserve `# flux` comments; handle immutable attribute
- [~] NixOS → generate Nix expression fragment, print to stdout (+ optional clipboard copy), instruct `nixos-rebuild switch`

### 4.3 — BSD Backends (§7.2)

- [~] FreeBSD → `local_unbound(8)` forwarder for encrypted DNS; resolv.conf → `127.0.0.1`
- [~] OpenBSD → `unwind(8)` or `unbound(8)`; resolv.conf → `127.0.0.1`
- [~] NetBSD → `unbound` (pkgsrc); resolv.conf → `127.0.0.1`

### 4.4 — Local Stub Resolver (DoQ / DNSCrypt)

- [~] Detect `dnsproxy` or `dnscrypt-proxy` in PATH
- [~] Offer install via detected package manager
- [~] Write forwarder config; restart service; point system DNS at `127.0.0.1`

---

## 5 — Privilege Layer (STD §3.1, §3.3)

- [x] Detect elevation tool: `sudo` → `pkexec` → `doas` (BSD-native)
- [x] Parent TUI/CLI never runs as UID 0
- [~] Spawn elevated child only for config-write step
- [x] Pass payload via **argv arrays or IPC** (ACS §7 — never shell interpolation)
- [x] Graceful error on elevation refusal → exit code 4 (`ELEVATION_FAILED`)

---

## 6 — Backup & Restore (STD §3.3)

- [x] Backup dir: `~/.local/share/flux/backups/` with mode `0600` (explicit `chmod` after create)
- [x] Filename: `YYYY-MM-DDTHH:MM:SSZ_<backend>.bak` (ISO 8601 UTC with mandatory `Z` per STD §12.2)
- [~] Backup scope: full DNS state snapshot (all affected files / nmcli output)
- [x] `dns backup` — manual trigger
- [x] `dns restore` — revert most recent backup; confirm before apply (`--yes` required in non-TTY)
- [x] Keep last N backups (configurable, default 10); auto-prune oldest
- [x] JSON metadata file alongside each snapshot: `{backend, provider, timestamp, schema_version}`

---

## 7 — Post-Apply Verification

- [x] After every `apply`, query a known host (e.g., `dns.google`) via the new resolver IP directly (bypass system cache)
- [x] Default timeout 5 s
- [x] On success: print confirmation with resolver IP and RTT in **milliseconds** (STD §12 — metric units)
- [x] On failure: exit code 7 (`VERIFICATION_FAILED`); offer auto-restore
- [x] `--no-verify` flag skips

---

## 8 — NTP Configuration (§6)

- [x] Detection: `systemd-timesyncd`, `chrony`, `ntpd`, `openntpd`, NixOS declarative
- [~] Adapters for each backend; restart service after write
- [x] Provider → NTP mapping: Cloudflare → `time.cloudflare.com`, Google → `time.google.com`, others → `pool.ntp.org`
- [x] `--ntp` flag on `dns apply`
- [x] `dns ntp --provider <slug>` standalone subcommand
- [~] Back up existing NTP config before write (ISO 8601 UTC `Z` filename)

---

## 9 — VPN Orchestration (§5.7)

### 9.1 — Cloudflare WARP (§5.7.1)

- [x] Detect `warp-cli` in PATH
- [~] Install hints per distro (Arch AUR, Debian/Ubuntu APT repo, Fedora/openSUSE RPM repo, NixOS nixpkgs)
- [~] `dns vpn connect -p cloudflare` → first-run `warp-cli register`; then `connect`; wait for `Connected` status
- [~] `dns vpn disconnect -p cloudflare` → `warp-cli disconnect`
- [~] WARP+ license via `--license`; pass to `warp-cli set-license`
- [~] Surface WARP state in `dns status`

### 9.2 — AdGuard VPN CLI (§5.7.2)

- [x] Detect `adguardvpn-cli`; offer install hint
- [~] `dns vpn connect -p adguard` → first-run `login`; then `connect [--location <city>]`
- [~] `dns vpn disconnect -p adguard`
- [~] `--vpn-protocol http2|quic`, mode TUN vs SOCKS5
- [~] Surface AdGuard VPN state in `dns status`
- [x] **Never link or bundle** `adguardvpn-cli` (not FOSS)

### 9.3 — Shared VPN Trait

- [x] `VpnProvider` trait: `connect()`, `disconnect()`, `status()`, `is_available()`
- [~] `dns vpn status` aggregates state for all detected clients

---

## 10 — CLI Surface (§9) — SFRS v1.0.0

### 10.1 — Argument Parsing

- [x] Use `clap` 4.x derive API (POSIX-compliant)
- [x] Positional shorthand for `apply`: `dns apply <provider> [tier] <protocol>` (§9.1)
- [x] Flagged form: `-p/--provider`, `-t/--tier`, `-P/--protocol`
- [x] Disambiguate positional: second arg is a tier if it matches a known tier, else a protocol
- [x] Positional and flagged forms produce identical internal state
- [x] Reject control chars (0x00–0x08, 0x0B–0x0C, 0x0E–0x1F) and ANSI escapes in all string args (ACS §7)
- [x] Canonicalize path args via `std::fs::canonicalize`; reject `..` / encoded variants / symlink escapes; allow-list against `/etc/` and `~/.local/share/flux/` (ACS §7)
- [~] Bounds-check numeric args against schema-declared min/max

### 10.2 — Subcommands (§9.2)

- [x] `dns apply` — apply DNS (+ optional NTP/VPN); idempotent
- [x] `dns status` — show DNS/NTP/VPN state
- [x] `dns list` — providers / tiers / protocols / VPN clients
- [x] `dns restore` — revert most recent backup
- [x] `dns verify` — test current DNS resolution
- [x] `dns detect` — display detected backend info
- [x] `dns backup` — manual snapshot
- [x] `dns ntp` — configure NTP independently
- [x] `dns vpn connect | disconnect | status`
- [x] `dns schema` — emit JSON Schema Draft 2020-12 for the full CLI surface (SFRS §2 Rule 4)
- [x] `dns describe` — human + machine manifest of the CLI (SFRS §2 Rule 4)
- [x] `dns mcp` — launch MCP server with lazy schema loading (SFRS §2 Rule 8; ACS §6)
- [x] `dns update-registry` — v0.2+ stub (print "coming in v0.2.0")
- [x] `dns` (no subcommand) / `dns --format explore` → TUI

### 10.3 — Global Flags (SFRS §3) — IDENTICAL ACROSS ALL STEELBORE CLIs

- [x] `--json` — alias for `--format json`
- [x] `--format <fmt>` — `json`, `jsonl`, `yaml`, `csv`, `explore`
- [x] `--fields <list>` — comma-separated field selection (ACS token economy)
- [x] `--dry-run` — emit action plan as JSON; no side effects; required on every write
- [x] `-v / --verbose` — diagnostic output to **stderr** (never stdout)
- [x] `-q / --quiet` — suppress non-error stderr
- [x] `--no-color` — equivalent to `--color=never`
- [x] `--color <when>` — `never` / `always` / `auto`
- [x] `-h / --help` — ≥2 examples per subcommand; one with `--json`; footer with maintainer + project URL (STD §13.2)
- [x] `-V / --version` — version + maintainer line + project URL; JSON mode includes `"maintainer"` and `"website"` in metadata
- [x] `--absolute-time` — disable relative-time rendering in human mode (JSON always UTC + `Z`)
- [x] `-0 / --print0` — NUL-delimited output for `xargs -0`
- [x] `--yes / --force` — skip confirmation in non-TTY mode (required for destructive ops)

### 10.4 — Apply Subcommand Flags (§9.4)

- [x] `-4 / --ipv4-only`, `-6 / --ipv6-only`
- [x] `--ntp` — also configure NTP
- [x] `--vpn warp|adguard`
- [x] `--no-backup`, `--no-verify`

### 10.5 — Output Mode Detection Cascade (§9.5, SFRS §5)

- [x] Single `OutputMode` struct shared by every subcommand
- [x] Cascade order:
  1. Explicit `--format` / `--json` flag
  2. Agent env: `AI_AGENT=1`, `AGENT=1`, `CI=true` → JSON, no-color, no-TUI, non-interactive, minimal verbosity
  3. `isatty(stdout)` → human mode + color
  4. Non-TTY → JSON mode
  5. Fallback → human mode
- [x] Informational env vars (do NOT change format): `CLAUDECODE`, `CURSOR_AGENT`, `GEMINI_CLI` → populate `metadata.invoking_agent`
- [x] `TERM=dumb` → suppress color and TUI (do not change format on its own)
- [x] Color precedence: `NO_COLOR` > `FORCE_COLOR` > `CLICOLOR` > `--color` > `--no-color` > TTY detection
- [x] **Explore guard:** if `AI_AGENT=1` and `--format explore` requested, fall back to JSON and warn on stderr (never trap an agent in interactive UI)

### 10.6 — JSON Output Envelope (§9.6, SFRS §6)

- [x] Define generic `Response<T>` type: `{ metadata: Metadata, data: T }`
- [x] `Metadata` struct: `tool`, `version`, `command`, `timestamp` (UTC `Z`), `pagination` (optional), `invoking_agent` (optional), `maintainer`, `website`
- [x] snake_case JSON property names
- [x] No bare-data serialization — every subcommand returns `Response<T>`
- [x] UTF-8 without BOM; no ANSI escapes; no log lines interleaved; single valid JSON document
- [x] Compact output (no pretty-printing) when stdout is non-TTY
- [x] Null fields omitted via `serde(skip_serializing_if = "Option::is_none")`
- [x] Numbers as JSON numbers (not strings); booleans `true`/`false`; nulls as JSON `null` (never `""` or `"N/A"`)
- [x] Schema version field for breaking-change tracking

### 10.7 — Structured Errors (§9.7, SFRS §1 #8, ACS §3)

- [x] Define `AppError` struct: `code` (enum), `exit_code`, `message`, `hint`, `timestamp` (UTC `Z`), `command`, `docs_url`
- [x] Enum variants for every canonical and tool-specific error condition
- [x] **Tips-thinking hints:** `hint` field MUST be a runnable command, not prose
  - [x] `DETECTION_FAILED` → `dns detect --json`
  - [x] `ELEVATION_FAILED` → `sudo dns apply ...`
  - [x] `CONFLICT` → `dns status --json`
  - [x] `APPLY_FAILED` → `dns restore && dns detect`
  - [x] `VERIFICATION_FAILED` → `dns restore`
  - [x] `VPN_ERROR` → `dns vpn status --json`
  - [x] `REGISTRY_FETCH_FAILED` (v0.2+) → `dns update-registry --verbose`
- [x] Errors emit to **stderr** (never stdout); stdout reserved for data
- [~] In human mode: render error as Red Oxide `#FF5C5C` text + readable hint
- [x] In machine mode: emit error JSON to stderr

### 10.8 — Canonical Exit Codes (§9.8, SFRS §4)

- [x] `0` — Success
- [x] `1` — General failure
- [x] `2` — Usage error
- [x] `3` — Resource not found (`DETECTION_FAILED`)
- [x] `4` — Permission denied (`ELEVATION_FAILED`)
- [x] `5` — Conflict
- [x] `6` — Apply failure (tool-specific; documented in `dns schema`)
- [x] `7` — Verification failure (tool-specific)
- [x] `8` — VPN error (tool-specific)
- [x] `9` — Registry fetch error (v0.2+; tool-specific)
- [x] `126`/`127`/`128+N` — POSIX-reserved (do not override)
- [x] Single `ExitCode` enum; every subcommand returns it
- [x] Document tool-specific codes (6–125) in `dns schema` output

### 10.9 — Self-Documentation (SFRS §2 Rule 4)

- [x] `dns schema` emits JSON Schema Draft 2020-12 covering every subcommand, every flag, every output type
- [~] Default output format for `dns schema` is Anthropic-format JSON Schema (drops directly into Claude function-calling)
- [x] `dns describe` emits human + machine manifest: subcommand tree, capability tags (`read`/`write`/`destructive`), required env vars, exit codes
- [x] Both subcommands respect `--fields` for payload trimming

---

## 11 — Agent Surface (ACS §6)

### 11.1 — MCP Server (`dns mcp`)

- [x] Crate: `rmcp` (Rust MCP SDK)
- [x] Transport: stdio default; `--transport http --port <p>` optional (streamable-HTTP)
- [x] `tools/list` returns names + one-line descriptions + capability tags ONLY (no schemas)
- [x] `tools/get` loads full input/output JSON Schema on-demand for a specific tool
- [x] Capability tags: `read` (status, list, detect, verify, schema, describe), `write` (apply, ntp, vpn, backup), `destructive` (restore)
- [x] Lazy-loading discipline: full schema is NEVER advertised in `tools/list` (ACS §6)

### 11.2 — Token-Economy Hygiene (ACS §8)

- [x] `--fields` honored on every list/get command
- [x] `--format jsonl` available for streaming-eligible list commands
- [x] Error hints are runnable commands (§10.7)
- [x] Compact JSON output (no pretty-printing) when stdout is non-TTY
- [x] Timestamps as ISO 8601 strings (22 chars), never object encodings
- [x] Null-field omission across all serializers

---

## 12 — TUI Interface (§11)

### 12.1 — Library & Theme

- [x] Add `ratatui` + `crossterm` backend
- [x] **Steelbore v1.2 palette** (six tokens only — STD §9):
  - [x] Background: Void Navy `#000027`
  - [x] Body text: Molten Amber `#D98E32`
  - [x] Headings / accent: Steel Blue `#4B7EB0`
  - [x] Success / selection: Radium Green `#50FA7B`
  - [x] Info / links: Liquid Coolant `#8BE9FD`
  - [x] Warning / error: Red Oxide `#FF5C5C`
- [x] **REMOVED in v1.2:** Steel Orange `#FE6B00` — no longer a Steelbore token; do NOT use anywhere
- [~] Verify all foreground pairs meet WCAG 2.1 AA against Void Navy
- [x] Honor `NO_COLOR`, `--no-color`, `TERM=dumb`

### 12.2 — Screens

- [x] Main Menu (Apply DNS / NTP / VPN / Status / Restore / Detect / About / Quit)
- [x] Provider Select (scrollable list)
- [x] Tier Select (filtered by provider)
- [x] Protocol Select (filtered by provider + detected backend)
- [x] Options (NTP toggle, VPN choice, IPv4/IPv6-only)
- [x] Confirmation (summary; `y`/`Enter` confirm, `n`/`Esc` cancel)
- [x] Progress (backup → detect → configure → verify; auto-advance)
- [x] Status
- [x] **About** — maintainer, contact, copyright year, project URL (STD §13.2)
- [x] Breadcrumb / header bar

### 12.3 — Keybindings (STD §8)

- [x] Vim: `j`/`k`, `h`/`l`, `g`/`G`, `/`, `q`
- [x] CUA: `Ctrl+C`, `Ctrl+Z` (= restore), `Tab`/`Shift+Tab`, `Enter`, `Esc`
- [x] `Space` toggles options; `y`/`n` for confirmation

### 12.4 — Graceful Degradation (§11.4)

- [x] Non-TTY stdout → TUI never activates
- [x] `AI_AGENT=1` / `AGENT=1` → TUI suppressed, fall back to JSON + stderr warning
- [x] All interactive elements reachable via keyboard alone
- [x] Color never the sole state indicator — pair with text labels and symbols

---

## 13 — Security, PQC & Compliance (§12)

### 13.1 — Memory Safety (STD §3.1)

- [x] No `unsafe` blocks without entry in `docs/SAFETY.md` + inline justification
- [x] Load `rust-guidelines` skill before writing any Rust
- [x] `cargo audit` in CI; RUSTSEC advisory fails the build
- [~] `cargo deny check licenses` — GPL-compatible only

### 13.2 — Performance (STD §3.2)

- [x] Release profile: `target-cpu=native`, LTO, `opt-level=3`
- [~] PGO scaffolding (deferred but planned)
- [~] Concurrency designed-in: parallel detection probes via `tokio`/`rayon`
- [~] Criterion benchmarks: detection engine, provider lookup, backup serialization
- [~] Startup target: < 100 ms to first TUI frame
- [~] Detection target: < 200 ms total

### 13.3 — Hardened Security + PQC (STD §3.3)

- [~] ASLR + CFI compiler flags on all binaries
- [x] Privilege layer per §5
- [~] **PQC readiness:** for v0.2+ `update-registry`, use rustls with PQC-hybrid: `X25519MLKEM768` KEM, `ML-DSA-65` signatures
- [x] Document migration plan in `docs/PQC.md`

### 13.4 — PFA Policy (STD §7)

- [x] Zero telemetry / analytics / beacons / ads
- [x] Outbound network traffic ONLY from: verify query, `dns update-registry` (v0.2+, user-initiated), external VPN subprocesses
- [x] Minimal permissions; elevation requested per-operation, not per-session
- [x] All state local under `~/.local/share/flux/` and `~/.config/flux/`

### 13.5 — Threat Model (ACS §7)

- [x] Path canonicalization with traversal rejection (§10.1)
- [x] Control-character rejection on all string args (§10.1)
- [~] Numeric bounds checks against schema (§10.1)
- [x] Destructive ops require `--yes` / `--force` in non-TTY (§10.3)
- [x] argv arrays exclusively for sub-process invocation; never shell interpolation (§5)
- [~] Indirect-prompt-injection defenses: never echo untrusted strings verbatim into hint fields without sanitization

### 13.6 — Date / Time / Units Compliance (STD §12)

- [x] **Crate: `jiff` (preferred) or `chrono`** — NEVER `time` 0.1.x
- [x] All stored / transmitted timestamps: `YYYY-MM-DDTHH:MM:SSZ` with **mandatory Z suffix**
- [x] FORBIDDEN in output: offset notation (`+00:00`), local time, AM/PM, `--local-time` flag, `NaiveDateTime`
- [~] Durations: ISO 8601 (`PT1H30M`) format in machine output; prose forms only in `--help`
- [x] `--absolute-time` flag disables relative-time rendering in human mode
- [x] Latency reported in milliseconds (ms); no imperial units in machine output
- [~] `serde(with = "...")` or newtype enforces UTC on deserialization

### 13.7 — Licensing (STD §4)

- [x] `GPL-3.0-or-later` in every `Cargo.toml`
- [x] SPDX header `// SPDX-License-Identifier: GPL-3.0-or-later` on every `.rs` (NOT on `.docx` / `.md` / `.pdf` — STD §4 exemption)
- [~] `cargo deny check licenses`: GPL-compatible only (MIT, Apache-2.0, BSD permitted)

### 13.8 — Attribution Surfaces (STD §13.2)

- [x] `--version` human mode: footer "Maintained by Mohamed Hammad &lt;Mohamed.Hammad@Steelbore.com&gt;" + `https://Flux.Steelbore.com/`
- [x] `--version --json`: `metadata.maintainer` + `metadata.website` populated
- [x] `--help` footer: project URL + maintainer name
- [x] `README.md`: "Maintainer" section with name, email, project URL
- [x] TUI About screen: maintainer, project URL, copyright year
- [x] Contact email always `Mohamed.Hammad@Steelbore.com` — never personal domain or GitHub handle

---

## 14 — Package Manager Detection (§8.1)

- [x] Detect: `pacman`, `apt`/`apt-get`, `dnf`, `zypper`, `nix`, `pkg` (FreeBSD), `pkg_add` (OpenBSD), `pkgsrc` (NetBSD)
- [~] Use for VPN / stub resolver install offers
- [~] Per-distro WARP package source (§8.1 OS matrix)

---

## 15 — `dns list`, `dns status`, `dns detect` Output

- [x] `dns list --providers`: slug, name, tiers, protocols, NTP; `--json` and `--jsonl` supported
- [x] `dns list --tiers -p <slug>`: tiers for a provider
- [x] `dns list --protocols -p <slug> [-t <tier>]`: protocols valid for provider/tier/backend
- [x] `dns list --vpn`: VPN clients + detected install status
- [x] `dns status`: backend, active provider/tier/protocol, nameserver IPs, NTP server, VPN states; `--json` supported
- [x] `dns detect`: all detected backends in priority order; selected backend with rationale; NTP backend; package manager
- [x] All commands respect `--fields` for payload trimming

---

## 16 — Testing (SFRS §8, Item 10)

- [x] Unit tests: provider registry compatibility-matrix enforcement
- [x] Unit tests: argument parser (positional + flagged, edge cases, control-char rejection)
- [x] Unit tests: conflict-resolution logic
- [x] Unit tests: backup filename generation (ISO 8601 UTC `Z` format)
- [x] Unit tests: structured error rendering (human + JSON modes)
- [~] Unit tests: JSON Schema validity (`dns schema | jsonschema` validation)
- [x] Unit tests: exit-code mapping matches §10.8
- [x] Unit tests: output-mode cascade decisions for every env-var combination
- [~] Integration tests per backend adapter (skip if daemon absent)
- [~] Integration: dry-run on current system → no writes
- [x] Integration: `AI_AGENT=1` invocation → JSON, no-color, no-TUI
- [~] Integration: cross-shell round-trip — output parseable in POSIX sh + Bash + Nushell + PowerShell + Ion
- [~] Compliance: every BLOCKER / CRITICAL / MAJOR item from SFRS §9 has a test
- [x] `cargo test` passes in CI without external daemons (unit tier)

---

## 17 — Documentation

- [~] `dns --help` / `dns <sub> --help` matches §9 / §10.2 / §10.3 / §10.4 spec exactly (with footer per STD §13.2)
- [~] `man dns(1)` covering all subcommands, flags, exit codes (canonical map per §10.8)
- [x] `README.md`: install + quick-start + posture + attribution (STD §13.2)
- [~] `CHANGELOG.md` following Keep-a-Changelog format
- [x] `docs/PQC.md` — PQC migration plan (§13.3)
- [x] `docs/SAFETY.md` — `unsafe` block registry (§13.1)
- [x] `AGENTS.md`, `CLAUDE.md`, `SKILL.md` per §1.2
- [x] Every authored document (PRD, design doc) ships with its GFM `.md` sibling per DOC §2

---

## 18 — Compliance Audit Gate (§15 PRD)

Before tagging v0.1.0 release, verify every row of the PRD §15 audit table passes. This is the final gate.

- [x] STD §2 — Naming justified (Flux as aerospace/astronomy term)
- [~] STD §3.1/§3.2/§3.3 — Memory safety, performance, hardened security + PQC
- [x] STD §4 — GPL-3.0-or-later + SPDX headers on source
- [x] STD §5 — Posture files present
- [x] STD §6.1 — POSIX-compliant
- [x] STD §7 — PFA satisfied
- [~] STD §8 — Vim + CUA in TUI
- [~] STD §9 — Six-token palette only; no Steel Orange
- [~] STD §10 — Share Tech Mono + Inconsolata (where fonts apply)
- [~] STD §11 — WCAG 2.1 AA contrast
- [x] STD §12 — UTC `Z` mandatory; metric units; `jiff`/`chrono` only
- [x] STD §13 — Attribution in --help / --version / README / About
- [x] SFRS §1 — Non-negotiables (UTF-8 no-BOM, POSIX-parseable, --json universal, stdout-data-only)
- [x] SFRS §4 — Canonical exit codes
- [x] SFRS §5 — Output-mode cascade
- [x] SFRS §6 — JSON envelope `{metadata, data}`
- [x] ACS §2 — AGENTS.md / CLAUDE.md / SKILL.md / CONTRIBUTING.md present
- [x] ACS §3 — Tips-thinking hints (runnable commands)
- [~] ACS §6 — MCP lazy schema loading
- [~] ACS §7 — Threat model: canonicalization, control-char rejection, argv arrays
- [x] DOC §2 — Every authored document has a GFM `.md` sibling

---

## 19 — Out of Scope (v0.1.0)

> Tracked to prevent scope creep. See PRD §4.2 and §14.

- `[FUTURE]` GUI interface (GTK4 / Iced)
- `[FUTURE]` Windows support (PowerShell / netsh backend)
- `[FUTURE]` macOS support (`scutil`, configuration profiles)
- `[FUTURE]` DNS benchmark / latency comparison
- `[FUTURE]` Fetchable provider registry (`dns update-registry` real implementation, v0.2)
- `[FUTURE]` Runtime DNSCrypt stamp fetching (v0.2/v0.3)
- `[FUTURE]` Custom user-defined providers
- `[FUTURE]` Named profiles (`dns apply --profile work`)
- `[FUTURE]` Scheduled switching (cron / systemd-timer)
- `[FUTURE]` WireGuard / Mullvad / ProtonVPN CLI adapters
- `[FUTURE]` MCP server HTTP transport (stdio is sufficient for v0.1.0)
- `[FUTURE]` General-use carve-out per STD §5.3 (reconsidered post-MVP)

---

*--- Forged in Steelbore ---*
