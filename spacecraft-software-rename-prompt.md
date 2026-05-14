# Spacecraft Software Rename Prompt (v3)

> Reusable prompt — paste into Claude Code, Codex, Cursor, Gemini-CLI, Antigravity, or any other coding agent. Run it once per repository.

---

## Context

The umbrella project, engineering standard, brand identity, and all associated assets formerly named **"Steelbore"** are renamed to **Spacecraft Software** (standard English capitalization — `Spacecraft`, one word, lowercase `c` mid-word, **not** `SpaceCraft`).

**One exception:** the operating-system line retains the **Steelbore** name. These terms stay untouched:

- `Steelbore OS`
- `Steelbore OS Bravais` (current distribution)
- `Steelbore OS Lattice` (legacy / historical reference)

Everywhere else, **`Steelbore` becomes `Spacecraft Software`** — including the Standard, Theme, color palette, brand guidelines, CLI compliance terminology, organization references, project-registry mentions, and the GitHub organization URL.

---

## Identity facts

- **Org / umbrella name:** Spacecraft Software
- **Domain:** `SpacecraftSoftware.org` (case-insensitive in URLs; render lowercase or initial-cap in display as the surface allows)
- **Contact email:** `Mohamed.Hammad@SpacecraftSoftware.org`
- **GitHub organization:** `github.com/Spacecraft-Software` (hyphenated — GitHub orgs disallow spaces; renamed from `github.com/Steelbore`)
- **Personal GitHub account:** `github.com/UnbreakableMJ` (Mohamed's personal repos — **separate** from the org, never renamed)
- **Author / Maintainer / Owner fields:** `Mohamed Hammad`

---

## Substitution rules — APPLY broadly

The default disposition for any `Steelbore` reference is: **rename it to `Spacecraft Software`** unless it falls under the OS-line exception.

### Brand / project references

| Old | New |
|---|---|
| `Steelbore` (standalone, referring to the project) | `Spacecraft Software` |
| `Steelbore Project` / `The Steelbore Project` | `Spacecraft Software` |
| `Steelbore project` / `Steelbore-project` | `Spacecraft Software` |
| `Steelbore initiative` | `Spacecraft Software` |
| `Steelbore umbrella` / `Steelbore-umbrella` | `Spacecraft Software` |
| `Steelbore organization` / `Steelbore org` | `Spacecraft Software` |
| `Steelbore ecosystem` | `Spacecraft Software ecosystem` |
| `The Steelbore Standard` | `The Spacecraft Software Standard` |
| `Steelbore Standard v1.x` | `Spacecraft Software Standard v1.x` |
| `Steelbore Theme` | `Spacecraft Software Theme` |
| `Steelbore colors` / `Steelbore color palette` | `Spacecraft Software colors` / `Spacecraft Software color palette` |
| `Steelbore branding` / `Steelbore brand guidelines` | `Spacecraft Software branding` / `Spacecraft Software brand guidelines` |
| `Steelbore-compliant` / `Steelbore compliance` | `Spacecraft Software-compliant` / `Spacecraft Software compliance` |
| `Steelbore-conformant` | `Spacecraft Software-conformant` |
| `Steelbore CLI` (when referring to the CLI Standard / surface) | `Spacecraft Software CLI` |
| `Steelbore subproject` | `Spacecraft Software project` |
| `Steelbore project registry` | `Spacecraft Software project registry` |
| `part of Steelbore` / `under Steelbore` | `part of Spacecraft Software` / `under Spacecraft Software` |

### Skill-ID text references (text inside files only — actual skill folders renamed separately)

| Old | New |
|---|---|
| `steelbore-standard` | `spacecraft-standard` |
| `steelbore-cli-standard` | `spacecraft-cli-standard` |
| `steelbore-cli-preference` | `spacecraft-cli-preference` |
| `steelbore-cli-shell` | `spacecraft-cli-shell` |
| `steelbore-missing-pkg` | `spacecraft-missing-pkg` |
| `steelbore-brand-guidelines` | `spacecraft-brand-guidelines` |
| `steelbore-theme-factory` | `spacecraft-theme-factory` |
| `steelbore-document-format` | `spacecraft-document-format` |
| `steelbore-agentic-cli` | `spacecraft-agentic-cli` |

### URLs, contact, ownership

| Old | New |
|---|---|
| `github.com/Steelbore/...` (the GitHub organization) | `github.com/Spacecraft-Software/...` |
| `git@github.com:Steelbore/...` (SSH form) | `git@github.com:Spacecraft-Software/...` |
| `steelbore.com`, `steelbore.org`, `steelbore.dev`, or any `*.steelbore.*` | `SpacecraftSoftware.org` (or appropriate subpath) |
| Any prior project email (`*@steelbore.*`) | `Mohamed.Hammad@SpacecraftSoftware.org` |
| `Author:` / `Maintainer:` / `Owner:` / `Copyright Holder:` / TOML `authors` arrays / `package.json` `author` | `Mohamed Hammad` |
| `Author:` field listing `Steelbore` as the entity | `Mohamed Hammad` |

Remember to update `[remote "origin"]` URLs in `.git/config` if the repo was cloned from the old org path — though normally that's the developer's local concern, not a tracked file.

---

## Hard exceptions — DO NOT rename

### Steelbore-named items that stay

- `Steelbore OS`
- `Steelbore OS Bravais`
- `Steelbore OS Lattice` (legacy / historical references)

### Other do-not-touch items

- All metallurgy/geology subproject codenames: Zamak, Ferrite OS, Forge, Bravais, Ferrocast, Craton, Pearlite, Gitway, Lodestone, Flux, Specs, MJ Benchmark, Caliper, Ironway, Mawaqit, Skills, Pulse, Anvil-SSH, and any other codename
- The §9 color tokens themselves (Void Navy `#000027` etc.) — the *names* of colors don't change; only surrounding text like `Steelbore color palette` → `Spacecraft Software color palette`
- The GPL-3.0-or-later license body — only the copyright-holder line is touched
- Third-party references: Safety-Critical Rust Consortium, Rust Foundation, NixOS, Limine, sequoia-chameleon-gnupg, greetd, tuigreet, COSMIC DE, Niri, LeftWM, Eww, Ironbar, etc.
- **`github.com/UnbreakableMJ/...` URLs** — this is Mohamed's *personal* GitHub account, separate from the Spacecraft Software org. Never rename these. (Org URLs under `github.com/Steelbore/...` DO get renamed — see substitution table above.)

---

## Files to walk

Every text file, with priority on:

- `README.md` and any `README.*` variants
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `SUPPORT.md`, `GOVERNANCE.md`, `CHANGELOG.md`
- `LICENSE` / `LICENSE.*` — only the copyright-holder line; never the GPL body
- `Cargo.toml` — `[package].authors`, `homepage`, `repository`, `description`, `[package.metadata.*]`
- `package.json` — `author`, `homepage`, `repository`, `bugs`, `description`
- `pyproject.toml`, `setup.py`, `setup.cfg`
- `flake.nix`, `default.nix`, `shell.nix` — `description`, `meta`, and any `inputs` URLs pointing to the old org
- `.github/` — workflows, issue/PR templates, `FUNDING.yml`, `CODEOWNERS`
- `docs/` — every `.md`, `.rst`, `.adoc`, `.typ`
- Source code — file-header comments, doc comments, `SPDX-FileCopyrightText` lines
- CLI surface — man pages, help text, `--version` strings, banner output, embedded ASCII art
- Website source if present (HTML, Astro, Hugo, mdBook)
- Container / CI / Nix configuration referencing the org, Standard, or domain
- **Flake inputs** specifically — `flake.nix` files may pin `github:Steelbore/<repo>` inputs that need to become `github:Spacecraft-Software/<repo>`
- **Cargo `git =` dependencies** — `Cargo.toml` entries like `foo = { git = "https://github.com/Steelbore/foo" }` need their URL updated

---

## Ambiguity handling

When you encounter a `Steelbore` occurrence whose meaning is genuinely unclear from context:

1. **Do not guess.** Leave it as-is.
2. Add it to `AMBIGUOUS_REVIEW.md` with: file path, line number, surrounding 3 lines of context, and your interpretation question.
3. The human reviews and decides.

Most likely ambiguity: phrases like `Steelbore docs` or `Steelbore configuration` — could mean OS-specific docs or umbrella docs. Default rule: **if it could reasonably mean the OS, flag it for review.**

---

## Required output

1. **Change summary** — table of `path | substitutions_applied | substitutions_skipped` per file touched.
2. **Ambiguity report** — `AMBIGUOUS_REVIEW.md` listing every flagged occurrence.
3. **Patch** — a single diff or PR-ready branch. Do not push or merge; leave for human review.

Suggested commit message:

```
chore: rename Steelbore -> Spacecraft Software

Per the brand consolidation:
- Org, Standard, Theme, palette, brand, CLI compliance: Spacecraft Software
- OS line (Steelbore OS, Steelbore OS Bravais): retained
- Domain: SpacecraftSoftware.org
- GitHub org: github.com/Spacecraft-Software (was github.com/Steelbore)
- Maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>

Personal repos at github.com/UnbreakableMJ are deliberately untouched.
Skill-folder renames in /mnt/skills/user/ are a separate phase.
```

---

## Optional verification pass

After the agent walk, verify mechanically with `rg` (Spacecraft Software CLI-preference compliant):

```nu
# Nushell — any remaining 'Steelbore' references should ONLY be inside
# "Steelbore OS", "Steelbore OS Bravais", or "Steelbore OS Lattice"
rg -n 'Steelbore' --type-add 'text:*.{md,toml,nix,json,yml,yaml,rs,nu,ion,sh,html,rst,adoc}' -t text

# Any remaining 'github.com/Steelbore' is a miss
rg -n 'github\.com[:/]Steelbore' --type-add 'text:*.{md,toml,nix,json,yml,yaml,rs,nu,ion,sh,html,rst,adoc}' -t text

# UnbreakableMJ references should still be present and unchanged where they
# referred to personal repos
rg -n 'UnbreakableMJ'
```

---

## Final checklist

- [ ] All non-OS `Steelbore` references renamed to `Spacecraft Software`
- [ ] `Steelbore OS`, `Steelbore OS Bravais`, `Steelbore OS Lattice` preserved verbatim
- [ ] All `github.com/Steelbore/...` URLs renamed to `github.com/Spacecraft-Software/...` (HTTPS and SSH forms)
- [ ] All `github.com/UnbreakableMJ/...` URLs preserved verbatim
- [ ] Flake inputs and Cargo `git =` URLs updated to the new org
- [ ] Domain replaced everywhere with `SpacecraftSoftware.org`
- [ ] Contact email is `Mohamed.Hammad@SpacecraftSoftware.org`
- [ ] Author / Maintainer / Owner / Copyright holder is `Mohamed Hammad`
- [ ] GPL license body unmodified
- [ ] No subproject codename altered
- [ ] All ambiguous cases captured in `AMBIGUOUS_REVIEW.md`
