# Phase 0: Workspace Hygiene & Foundation — Research

**Researched:** 2026-04-14
**Domain:** Rust workspace hygiene, dual-licensing, CI baseline (GitHub Actions), vendor relocation, crate rename
**Confidence:** HIGH (all stack versions already verified 2026-04-14 in STACK.md against live crates.io; 2026 CI conventions cross-checked via web search)

## Summary

Phase 0 is operational cleanup, not design. STACK.md already pinned every version needed. CONTEXT.md locked every decision. This research fills the *copy-pasteable detail layer* the planner needs: literal license text, exact TOML, exact YAML, exact rewritten docs.

Three 2026 conventions differ from older training data and MUST be respected:

1. **`resolver = "3"`** is the right choice for edition 2024 (stabilized in Rust 1.85, 2025-02-20). The current root `Cargo.toml` has `resolver = "2"` — that is an artefact of edition 2021 and must be upgraded along with the `edition = "2024"` workspace default. Per the Edition Guide, `edition = "2024"` *implies* `resolver = "3"`, but being explicit in the workspace root is still recommended.
2. **`cargo-machete`** (stable, fast, string-based) is the right default over `cargo-udeps` (nightly-only, compiler-backed, slower). Both are actively maintained, but udeps requires nightly toolchain — incompatible with our pinned `rust-toolchain.toml = "1.86"`. Recommendation: run `cargo-machete` in CI on stable; defer udeps to an optional weekly deep-scan job.
3. **`Swatinem/rust-cache@v2`** remains canonical (current latest: v2.8.1 as of this research; pin to `@v2` floating). There is no v3.

**Primary recommendation:** Plan this as ~5 parallelizable plans — (A) license files + SPDX sweep, (B) workspace Cargo.toml refactor + crate rename + stub reset, (C) vendor relocation + STAMP files + `.gitattributes`, (D) toolchain pin + per-crate scaffold, (E) CI workflow + doc-lint script + README/project.md/CONTRIBUTING/CHANGELOG rewrites. The planner can split further but these five have minimal cross-coupling.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Doc Sweep (HYG-01)**
- Replace `tui-vfx` with `tachyonfx` in `README.md`, `project.md`, per-crate READMEs.
- Preserve exactly **one** "why not tui-vfx" rationale section (inside PROJECT.md Key Decisions or a dedicated ADR).
- Also scrub `Haskell bindings` references (user descoped them).
- Also scrub `pyo3-asyncio`, `cgmath`, `tui-rs` references.

**Stub Crate Dep Reset (HYG-02)**
- `crates/happyterminals-core/Cargo.toml`: remove `pyo3`, `tui-vfx`, `ratatui` from dependencies.
- `crates/happyterminals-renderer/Cargo.toml`: same treatment — strip speculative deps.
- `crates/happyterminals-compositor/Cargo.toml`: same; also rename to `happyterminals-pipeline`.
- Stub `lib.rs` files stay minimal (empty module or single doc comment) until real call sites exist.

**Workspace Dependencies (HYG-03)** — pinned versions per STACK.md §10 + the additions listed in CONTEXT.md. All member crates declare dependencies via `{ workspace = true }`.

**Crate Scaffold (HYG-03 cont'd)**
- Rename `compositor` → `pipeline` (directory rename + `Cargo.toml` name change + remove any references).
- Add empty crates: `happyterminals-scene`, `happyterminals-dsl`, `happyterminals-backend-ratatui`.
- Add meta crate `happyterminals` (re-exports; empty for now).
- Add commented-out `happyterminals-py` entry in workspace members (activated at M4).

**Vendor Relocation (HYG-04)**
- Move `vendor/pyo3/` → `vendor/_reference/pyo3/` + `vendor/_reference/pyo3/STAMP.txt` with upstream commit SHA + date + source URL.
- Move `vendor/ratatui/` → `vendor/_reference/ratatui/` + `STAMP.txt`.
- Move `vendor/tui-vfx/` → `vendor/_reference/tui-vfx/` + `STAMP.txt`.
- Add `.gitattributes` marking `vendor/_reference/** linguist-vendored=true`.
- **Never** reference vendored copies via `path =` in any `Cargo.toml` — they are for reading only.

**Dual License (HYG-05)**
- `LICENSE-MIT` at repo root (copyright line: **"Copyright (c) 2026 the happyterminals authors"** per user decision 2026-04-14).
- `LICENSE-APACHE` at repo root (standard Apache-2.0 text).
- Every crate's `Cargo.toml` uses SPDX `license = "MIT OR Apache-2.0"`.
- README.md gets a License section + Apache-2.0 §5 contribution clause.
- `CONTRIBUTING.md` repeats the contribution clause.

**Registry Reservation (HYG-06) — DEFERRED OUT OF PHASE 0.** Do not prepare placeholders in Phase 0.

**Rust Toolchain (HYG-07)** — `rust-toolchain.toml` pins `channel = "1.86"` with `components = ["clippy", "rustfmt"]`, `profile = "default"`.

**Baseline CI (HYG-08, HYG-09)** — GitHub Actions `.github/workflows/ci.yml` on `push` + `pull_request`, matrix `ubuntu-latest` only in Phase 0, Rust 1.86 + stable. macOS/Windows deferred to M2.

### Claude's Discretion
- Exact structure of the doc-lint script (shell vs small Rust binary — **default: shell with `rg`**).
- Whether to enforce `forbid(unsafe_code)` at workspace level — **recommend: yes for `-core`, `-pipeline`, `-scene`, `-dsl`; allow in `-renderer` for potential SIMD later**.
- Exact copyright holder on LICENSE-MIT — **locked to "the happyterminals authors"**.
- Whether to commit a `.cargo/config.toml` with lints deny list — **recommend: keep lints in `[workspace.lints]` in `Cargo.toml`; `.cargo/config.toml` not required in Phase 0**.
- Whether to set up `release-plz` — **defer to M3 publish phase**.

### Deferred Ideas (OUT OF SCOPE)
- HYG-06 registry reservation (crates.io / PyPI) — revisit before M3 publish.
- Feature code (reactive primitives, Grid, Pipeline, renderer) — Phase 1.0+.
- Maturin / PyO3 wheel infra — Phase 4.x.
- Qdrant docker-compose — not a happyterminals concern.
- `cargo semver-checks`, `release-plz`, cross-OS CI, docs.rs feature matrix — Phase 3.5.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HYG-01 | README/project.md/per-crate READMEs describe tachyonfx, no stray tui-vfx or Haskell references | Items 9, 10, 11 (rewrites); Item 8 (doc-lint enforces) |
| HYG-02 | Stub crates stripped of speculative deps | Item 4 (per-crate Cargo.toml template); Item 15 (empty lib.rs scaffolding) |
| HYG-03 | `[workspace.dependencies]` with pinned set | Item 2 (full Cargo.toml); Item 4 (per-crate template) |
| HYG-04 | Vendor relocated to `vendor/_reference/` with STAMP.txt + `.gitattributes` | Items 5, 6 |
| HYG-05 | Dual LICENSE-MIT + LICENSE-APACHE + SPDX + README License section + CONTRIBUTING | Items 1, 4, 10, 12 |
| HYG-07 | `rust-toolchain.toml` pins 1.86 with clippy + rustfmt | Item 3 |
| HYG-08 | CI: fmt, clippy `-D warnings`, test, doc `-D warnings`, `cargo tree -d`, machete | Item 7 |
| HYG-09 | Doc-lint CI step fails on forbidden strings outside allowlist | Item 8 |

*(HYG-06 registry reservation deferred out of Phase 0 per user decision — not in the exit checklist.)*
</phase_requirements>

## 1. LICENSE-MIT and LICENSE-APACHE — Canonical Text

Both go at repo root, verbatim. No modification except the copyright line in LICENSE-MIT.

### `LICENSE-MIT`

```
MIT License

Copyright (c) 2026 the happyterminals authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### `LICENSE-APACHE`

This is the canonical Apache License, Version 2.0 text from https://www.apache.org/licenses/LICENSE-2.0.txt — reproduced verbatim (this is the text Rust projects standardize on):

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for describing the origin of the Work and
      reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Support. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or support.

   END OF TERMS AND CONDITIONS

   APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file name or class name and description of purpose be included on
      the same page as the copyright notice for easier identification
      within third-party archives.

   Copyright 2026 the happyterminals authors

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied. See the License for the specific language governing permissions and
   limitations under the License.
```

**Note:** The Apache-2.0 text itself is never modified; only the Appendix example copyright line at the very bottom is customized. The standard Rust-ecosystem practice (serde, tokio, regex all do this) is to fill in that Appendix line with the project's own copyright statement.

**Source:** https://www.apache.org/licenses/LICENSE-2.0.txt (canonical), https://opensource.org/license/mit (MIT canonical). Both verified against the Rust standard-library dual-license convention.

## 2. Root `Cargo.toml` — Full Block

Replaces the current 9-line root `Cargo.toml`. Every version is from STACK.md §10 (verified 2026-04-14 against crates.io) plus CONTEXT.md additions. Key 2026 gotcha: **`resolver = "3"`** for edition 2024 (new in Rust 1.85). `pyo3` is `default-features = false` because `-core`/other libraries must *declare* pyo3 in the workspace table so the pin is consistent, but must NOT activate `extension-module` or `auto-initialize` — those belong to `-py` only. No workspace member actually depends on pyo3 in Phase 0.

```toml
[workspace]
resolver = "3"
members = [
    "crates/happyterminals",
    "crates/happyterminals-core",
    "crates/happyterminals-renderer",
    "crates/happyterminals-pipeline",
    "crates/happyterminals-scene",
    "crates/happyterminals-dsl",
    "crates/happyterminals-backend-ratatui",
    # "crates/happyterminals-py",  # activated in Milestone 4
]

[workspace.package]
version      = "0.0.0"
edition      = "2024"
rust-version = "1.86"
license      = "MIT OR Apache-2.0"
repository   = "https://github.com/lynxnathan/happyterminals"
homepage     = "https://github.com/lynxnathan/happyterminals"
authors      = ["the happyterminals authors"]
keywords     = ["terminal", "tui", "ratatui", "ascii", "reactive"]
categories   = ["command-line-interface", "graphics", "rendering"]
readme       = "README.md"

[workspace.dependencies]
# ratatui ecosystem (libs consume ratatui-core; apps consume ratatui)
ratatui-core      = "0.1"
ratatui           = { version = "0.30", features = ["crossterm"] }
ratatui-widgets   = "0.3"
ratatui-crossterm = "0.1"
ratatui-macros    = "0.7"
crossterm         = "0.29"
tachyonfx         = "0.25"

# reactivity
reactive_graph = "0.2.13"
any_spawner    = "0.3"

# 3D
glam   = "0.32.1"
tobj   = "4.0"
stl_io = "0.11"

# serde + schema
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
schemars   = "1.2"
jsonschema = "0.46"

# errors / logs
thiserror          = "2.0"
color-eyre         = "0.6.5"
tracing            = "0.1"
tracing-subscriber = "0.3"

# misc utils (aligned with tachyonfx's choices to keep the lockfile tidy)
compact_str = "0.9"
bon         = "3.9"
slotmap     = "1"

# python (final milestone; pinned now so -py drop-in is trivial)
pyo3                = { version = "0.28.3", default-features = false }
pyo3-async-runtimes = "0.28"

# dev-dependencies
insta     = "1.47"
proptest  = "1.11"
criterion = "0.8"

# Internal crates — members declare each other via workspace deps for consistent pathing.
happyterminals                  = { version = "0.0.0", path = "crates/happyterminals" }
happyterminals-core             = { version = "0.0.0", path = "crates/happyterminals-core" }
happyterminals-renderer         = { version = "0.0.0", path = "crates/happyterminals-renderer" }
happyterminals-pipeline         = { version = "0.0.0", path = "crates/happyterminals-pipeline" }
happyterminals-scene            = { version = "0.0.0", path = "crates/happyterminals-scene" }
happyterminals-dsl              = { version = "0.0.0", path = "crates/happyterminals-dsl" }
happyterminals-backend-ratatui  = { version = "0.0.0", path = "crates/happyterminals-backend-ratatui" }

[workspace.lints.rust]
unsafe_code                    = "forbid"  # per-crate override in -renderer (see Item 4)
missing_docs                   = "warn"
rust_2018_idioms               = { level = "warn", priority = -1 }
unreachable_pub                = "warn"

[workspace.lints.clippy]
all                = { level = "warn", priority = -1 }
pedantic           = { level = "warn", priority = -1 }
unwrap_used        = "deny"
expect_used        = "deny"
dbg_macro          = "deny"
todo               = "warn"
module_name_repetitions = "allow"  # clippy::pedantic is noisy on workspace naming
missing_errors_doc = "allow"       # turn back on in each crate as APIs stabilize
missing_panics_doc = "allow"
```

**Why `default-features = false` on `pyo3`:** pyo3's default features include `macros`, `indoc`, `unindent`. Consumers (only `-py` in Milestone 4) will re-enable what they need explicitly. Listing it in `[workspace.dependencies]` with minimal features keeps the pin consistent across the workspace without silently pulling pyo3 into every member's build graph.

**Why `resolver = "3"` (2026 CONVENTION):** edition 2024 implies resolver 3; being explicit in the workspace root is idiomatic. Do NOT downgrade to `resolver = "2"` even though older blog posts show it. Per the [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html), resolver 3 enables the `rust-version`-aware dependency resolver, which gracefully prefers dep versions compatible with our MSRV 1.86.

**Why no `ratatui`/`crossterm` feature matrix here:** apps consuming `ratatui = { workspace = true }` get crossterm via the workspace-default features. Libraries consume `ratatui-core = { workspace = true }` only.

**Canonical refs:**
- https://doc.rust-lang.org/cargo/reference/workspaces.html#the-workspace-section
- https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html (resolver = "3")
- https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table

## 3. `rust-toolchain.toml`

```toml
[toolchain]
channel    = "1.86"
components = ["clippy", "rustfmt"]
profile    = "default"
```

**Notes:**
- `profile = "default"` (not `"minimal"`) so developers get `rust-docs`, `rust-std` locally. CONTEXT.md locks this to `"default"`.
- Rust 1.86 is the MSRV floor that matches ratatui 0.30 and pyo3 0.28; higher local toolchains (the local machine runs 1.92 per `.tool-versions`) still satisfy the pin — `rust-toolchain.toml` *pins* the channel, which rustup auto-installs.
- Do NOT include `targets = [...]` in Phase 0; wasm/cross targets are later-milestone concerns.

**Canonical ref:** https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file

## 4. Per-Crate `Cargo.toml` Template (minimal stub)

All member crates get this shape. Only `-renderer` flips `unsafe_code` from `forbid` to `allow` (room for future SIMD). `-core`, `-pipeline`, `-scene`, `-dsl`, `-backend-ratatui`, and the `happyterminals` meta crate keep the workspace-level `forbid`.

### Template for any stub crate (e.g., `crates/happyterminals-core/Cargo.toml`):

```toml
[package]
name         = "happyterminals-core"
description  = "Reactive primitives (Signal / Memo / Effect) and Grid buffer for the happyterminals scene manager."
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true
homepage.workspace     = true
authors.workspace      = true
keywords.workspace     = true
categories.workspace   = true
readme                 = "README.md"

[dependencies]
# Empty during Phase 0 — stubs have no call sites yet.
# Real deps added in Phase 1.0 when the reactive graph lands:
#   reactive_graph = { workspace = true }
#   any_spawner    = { workspace = true }
#   thiserror      = { workspace = true }

[lints]
workspace = true
```

### `-renderer` variant (only difference: `unsafe_code` allowed):

```toml
[package]
name         = "happyterminals-renderer"
description  = "ASCII 3D rasterizer (z-buffer, perspective projection, OBJ/STL loading) for happyterminals."
version.workspace      = true
edition.workspace      = true
rust-version.workspace = true
license.workspace      = true
repository.workspace   = true
homepage.workspace     = true
authors.workspace      = true
keywords.workspace     = true
categories.workspace   = true
readme                 = "README.md"

[dependencies]
# Empty during Phase 0.

[lints.rust]
unsafe_code = "allow"  # reserved for future SIMD in hot paths; JUSTIFY before use.

[lints.clippy]
# Inherit workspace clippy lints even though [lints] isn't using workspace=true.
# (Can't use `workspace = true` alongside a specific override in the same [lints] table.)
all         = { level = "warn", priority = -1 }
pedantic    = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
dbg_macro   = "deny"
```

### Per-crate descriptions (for the `description =` field):

| Crate | Description |
|-------|-------------|
| `happyterminals` | Meta crate — curated public re-exports (prelude, run, common types). |
| `happyterminals-core` | Reactive primitives (Signal / Memo / Effect) and Grid buffer for the happyterminals scene manager. |
| `happyterminals-renderer` | ASCII 3D rasterizer (z-buffer, perspective projection, OBJ/STL loading) for happyterminals. |
| `happyterminals-pipeline` | Effect pipeline (`dyn Effect` trait objects) and tachyonfx adapter for happyterminals. |
| `happyterminals-scene` | Scene IR and scene-graph types consumed by every front-end (Rust builder, JSON, Python). |
| `happyterminals-dsl` | Declarative scene builder (Rust) and JSON recipe loader for happyterminals. |
| `happyterminals-backend-ratatui` | Ratatui/crossterm event loop and TerminalGuard (RAII + panic hook) for happyterminals. |

## 5. `.gitattributes`

At repo root. The vendor lines ensure GitHub's language statistics don't count those files toward repo language composition. The `text=auto` line + the explicit `eol=lf` for source files prevents Windows CRLF drift that would otherwise thrash snapshot tests later.

```gitattributes
# Default: treat files as text; let Git auto-detect line endings on checkin/checkout.
*           text=auto eol=lf

# Rust source + config — always LF, text.
*.rs        text eol=lf
*.toml      text eol=lf
*.md        text eol=lf
*.yml       text eol=lf
*.yaml      text eol=lf
Cargo.lock  text eol=lf

# Vendored reference copies — linguist stats exclude these, and they should
# never be linted, formatted, or modified by tooling in this repo.
vendor/_reference/**                    linguist-vendored=true
vendor/_reference/**/*.rs               linguist-vendored=true -diff
vendor/_reference/**                    -text

# Binary fixtures we may add later for mesh/snapshot tests
*.obj       binary
*.stl       binary
*.png       binary
*.gif       binary
```

**Canonical ref:** https://github.com/github-linguist/linguist/blob/main/docs/overrides.md

## 6. `STAMP.txt` Format — Reference Provenance

Each vendored-reference directory (`vendor/_reference/{name}/`) gets a `STAMP.txt`. Purpose: tell any future maintainer *exactly* which upstream snapshot this is a reference against, so they can re-diff or replace it without detective work.

**Critical note from the filesystem audit:** the current `vendor/pyo3/`, `vendor/ratatui/`, and `vendor/tui-vfx/` directories are **local placeholder stubs** (10–30 lines of fake types just enough to let the stub crates compile), NOT real upstream source clones. The STAMP format below works for both cases — the planner MUST preserve this distinction by marking the `placeholder` status when applicable. If the planner decides to replace the placeholders with real upstream snapshots during Phase 0, the STAMP.txt gets the real SHA; otherwise it records the placeholder status.

### Canonical STAMP.txt template

```
Upstream:        <upstream repo URL>
Upstream name:   <upstream crate name>
Upstream version: <upstream release tag, e.g., v0.30.0>
Upstream commit: <full 40-char SHA or "placeholder — no upstream snapshot taken">
Captured:        <YYYY-MM-DD of the snapshot>
Captured by:     <git user name / email>
Purpose:         Reference reading only. Never consumed as a `path =` dependency.
                 See PROJECT.md §"Vendor policy" for the full rationale.

Relationship to workspace:
    This directory is NOT a dependency. The workspace consumes <crate> via
    crates.io pins in [workspace.dependencies]. This copy exists so
    contributors can grep real upstream source when debugging API choices
    or comparing their implementation to upstream without a network round-trip.

How to refresh:
    1. git -C <tmp> clone <upstream URL>
    2. git -C <tmp> checkout <version-tag>
    3. rm -rf vendor/_reference/<crate>/*
    4. cp -r <tmp>/{<source-paths>} vendor/_reference/<crate>/
    5. Update this STAMP.txt with the new commit SHA + date.
```

### Extracting the upstream commit SHA (canonical command)

For real upstream snapshots (not the current placeholders), this is how to record an exact SHA:

```bash
# Inside a fresh clone checked out to the release tag:
git -C /tmp/pyo3-upstream rev-parse HEAD
# → returns 40-char SHA like 9f4b8c1d9a2e3b7f5c6d8e9a0b1c2d3e4f5a6b7c
```

For the **current placeholder** state, use:

```
Upstream commit: placeholder — no upstream snapshot taken; local stub only.
```

### Concrete STAMP.txt files to create

**`vendor/_reference/pyo3/STAMP.txt`:**
```
Upstream:         https://github.com/PyO3/pyo3
Upstream name:    pyo3
Upstream version: (placeholder, workspace pins 0.28.3)
Upstream commit:  placeholder — no upstream snapshot taken; local stub only.
Captured:         2026-04-14
Captured by:      Nathan Ribeiro <nribeiro@strm.com.br>
Purpose:          Reference reading only. Never consumed as a `path =` dependency.
                  See PROJECT.md §"Vendor policy" for the full rationale.

Relationship to workspace:
    This directory is NOT a dependency. The workspace consumes pyo3 via
    crates.io pin 0.28.3 in [workspace.dependencies] (activated only when
    happyterminals-py lands in Milestone 4). The current contents are a
    minimal local stub that was authored before this policy existed; a real
    upstream snapshot will replace it when Milestone 4 begins, or sooner if
    someone needs to grep real PyO3 source.

How to refresh to a real upstream snapshot:
    1. git clone https://github.com/PyO3/pyo3 /tmp/pyo3-upstream
    2. git -C /tmp/pyo3-upstream checkout v0.28.3
    3. rm -rf vendor/_reference/pyo3/*
    4. cp -r /tmp/pyo3-upstream/{src,Cargo.toml,README.md,CHANGELOG.md} vendor/_reference/pyo3/
    5. Update this STAMP.txt: Upstream commit = `git -C /tmp/pyo3-upstream rev-parse HEAD`.
```

**`vendor/_reference/ratatui/STAMP.txt`** and **`vendor/_reference/tui-vfx/STAMP.txt`** follow the same template, substituting the upstream URL (`https://github.com/ratatui/ratatui`, `https://github.com/5ocworkshop/tui-vfx`) and crate name. For tui-vfx, the purpose line should additionally read: *"Preserved as reference for the 'why we chose tachyonfx over tui-vfx' rationale section in PROJECT.md."*

## 7. GitHub Actions `.github/workflows/ci.yml`

Targets `ubuntu-latest` only in Phase 0 (macOS/Windows deferred). One workflow, five jobs, all with `Swatinem/rust-cache@v2`. Concurrency group cancels in-flight runs on the same ref when a new push arrives — standard 2026 pattern to save CI minutes.

```yaml
name: CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0  # CI cache is cleaner without incremental artifacts
  RUST_BACKTRACE: 1

jobs:
  fmt:
    name: rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # rust-toolchain.toml pins the channel; no dtolnay/rust-toolchain needed
      # if the runner's default rustup honors the pin, but pinning explicitly
      # is more robust across runner image changes.
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.86"
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.86"
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  test:
    name: test (${{ matrix.rust }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust: ["1.86", "stable"]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.rust }}
      - run: cargo test --workspace --all-features --no-fail-fast

  docs:
    name: cargo doc
    runs-on: ubuntu-latest
    env:
      RUSTDOCFLAGS: "-D warnings"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.86"
      - uses: Swatinem/rust-cache@v2
      - run: cargo doc --workspace --no-deps --all-features

  hygiene:
    name: hygiene (dup deps, unused deps, doc-lint)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.86"
      - uses: Swatinem/rust-cache@v2

      # Duplicate dependency detection.
      # `cargo tree --workspace --duplicates --edges normal` keeps the output
      # focused on the normal dep graph (ignores dev-deps, which legitimately
      # pull in their own versions). `| tee` so the log is visible; `test -z`
      # fails the step if any duplicates were printed.
      - name: cargo tree --duplicates
        run: |
          set -euo pipefail
          DUPES=$(cargo tree --workspace --duplicates --edges normal --format '{p}' 2>/dev/null || true)
          if [ -n "$DUPES" ]; then
            echo "::error::Duplicate dependencies detected:"
            echo "$DUPES"
            exit 1
          fi

      # Unused dependency detection — cargo-machete on stable (fast, string-based).
      # We deliberately do NOT run cargo-udeps here because udeps requires nightly
      # and our rust-toolchain.toml pins 1.86. An opt-in weekly udeps job can be
      # added later (not Phase 0 scope).
      - name: install cargo-machete
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-machete
      - name: cargo-machete
        run: cargo machete

      # Doc-lint — see Item 8 for the script itself.
      - name: doc-lint (forbidden strings)
        run: bash scripts/doc-lint.sh
```

### Notes on 2026 conventions embedded above

- **`Swatinem/rust-cache@v2`** is the canonical action; pin to `@v2` (latest minor v2.8.1 as of research). There is no v3. Keeping `@v2` floating lets us pick up bugfixes automatically without breaking changes.
- **`taiki-e/install-action@v2`** is the 2026-canonical way to install cargo subcommands from prebuilt binaries (much faster than `cargo install`). It supports `cargo-machete`, `cargo-hack`, `cargo-nextest`, `cargo-deny`, etc.
- **`dtolnay/rust-toolchain@master`** (sic — this action uses `@master`, not a version tag, as its README and every Rust project in 2026 demonstrates). We pass `toolchain: "1.86"` explicitly so the job doesn't depend on `rust-toolchain.toml` parsing on the runner.
- **`cargo tree --workspace --duplicates --edges normal`** is the 2026-correct invocation. `--edges normal` excludes `dev`/`build` edges, focusing on the production dep graph. `--format '{p}'` strips the tree rendering down to package lines for easy emptiness testing.
- **Matrix strategy** keeps the explosion controlled: only the `test` job matrices (MSRV 1.86 + stable); fmt/clippy/docs/hygiene run once each. Total jobs per push: 7 (fmt + clippy + 2×test + docs + hygiene + GHA overhead).
- **`concurrency` group** with `cancel-in-progress: true` is the 2026-default for PR workflows — avoids running stale commits.

**Canonical refs:**
- https://github.com/Swatinem/rust-cache (v2.8.1, active)
- https://github.com/dtolnay/rust-toolchain
- https://github.com/taiki-e/install-action
- https://doc.rust-lang.org/cargo/commands/cargo-tree.html#manifest-options

## 8. Doc-Lint Script — `scripts/doc-lint.sh`

Shell with `rg` (ripgrep). Hard-fails CI if any forbidden string appears outside `vendor/_reference/**` or a single explicit allowlist file. Prints clear error messages naming the file + line.

Install `ripgrep` on the CI runner via the Ubuntu image (already installed on `ubuntu-latest`) or as part of the `hygiene` job setup.

```bash
#!/usr/bin/env bash
# scripts/doc-lint.sh
#
# Fails if any of the forbidden strings appear outside vendor/_reference/ or
# the single ADR that documents the rationale for avoiding them.
#
# Forbidden strings — see PROJECT.md and REQUIREMENTS.md HYG-09:
#   - "tui-vfx"         : effects layer is tachyonfx
#   - "Haskell bindings": descoped; Python is the only binding
#   - "pyo3-asyncio"    : abandoned upstream; we use pyo3-async-runtimes
#   - "cgmath"          : unmaintained; we use glam
#   - "tui-rs"          : deprecated crate name; we use ratatui
#
# Allowlist (rationale only, prose not code):
#   - docs/decisions/stack-rationale.md
#   - vendor/_reference/** (upstream copies are exempt)
#   - .eclusa/** (planning artifacts; discussed explicitly below)
#
# Exit 0 if clean, 1 with a human-readable error if not.

set -euo pipefail

FORBIDDEN=(
    "tui-vfx"
    "Haskell bindings"
    "pyo3-asyncio"
    "cgmath"
    "tui-rs"
)

# Paths allowed to contain the forbidden strings.
# Use `rg --glob '!...'` negations to exclude.
EXCLUDES=(
    '!vendor/_reference/**'
    '!docs/decisions/stack-rationale.md'
    '!.eclusa/**'                           # planning artifacts discuss the pivots explicitly
    '!scripts/doc-lint.sh'                  # this file literally contains the strings
    '!.github/workflows/ci.yml'             # if we ever name the step after them
    '!CHANGELOG.md'                         # migration notes can cite old names
    '!target/**'
    '!Cargo.lock'
)

FAIL=0

for word in "${FORBIDDEN[@]}"; do
    # Build rg command: case-insensitive, show line numbers, excludes above.
    rg_cmd=(rg --hidden --line-number --color=never --no-heading --fixed-strings "$word")
    for exc in "${EXCLUDES[@]}"; do
        rg_cmd+=(--glob "$exc")
    done

    if OUTPUT=$("${rg_cmd[@]}" 2>/dev/null); then
        echo "::error::Forbidden string '$word' found outside allowlist:"
        echo "$OUTPUT" | sed 's/^/  /'
        FAIL=1
    fi
done

if [ "$FAIL" -eq 1 ]; then
    echo
    echo "Doc-lint failed. If a reference to a forbidden term is genuinely needed for"
    echo "rationale, add it ONLY to docs/decisions/stack-rationale.md. Otherwise, fix"
    echo "the file to use the current name (tachyonfx, pyo3-async-runtimes, glam, ratatui)."
    exit 1
fi

echo "Doc-lint: clean."
```

**Make the script executable during Phase 0 (`chmod +x scripts/doc-lint.sh`) and commit with that mode.**

### Allowlist file — `docs/decisions/stack-rationale.md`

Create this file as the **single** place forbidden strings are allowed, so the "Why not X" sections have a home. Stub content:

```markdown
# Stack Rationale — Why Not X

This file is the sole allowlist for the doc-lint CI step. Historical pivots
and anti-recommendations are documented here so contributors understand *why*
the chosen stack is the chosen stack.

## Why not tui-vfx (chose tachyonfx)

tui-vfx was a promising pre-1.0 project (8★, ~5 weeks old at happyterminals'
start) but had no community adoption. tachyonfx (1,182★, 0.25.0, maintained
under the ratatui org with 50+ effects, DSL, WASM editor) is the durable
foundation. See `.eclusa/research/STACK.md` §1.3.

## Why not pyo3-asyncio (chose pyo3-async-runtimes)

`pyo3-asyncio` last released 2023-11-11 and does not support pyo3 ≥ 0.21.
The PyO3 org forked it as `pyo3-async-runtimes`; that's where development
happens in 2026. Any tutorial older than late-2024 recommending the old
crate is stale.

## Why not cgmath (chose glam)

cgmath last release was 2021-01-03; effectively unmaintained. glam (0.32.1,
2026-03-06) is the 2026-standard graphics-math crate — used by Bevy, wgpu
examples, rend3, and most of the modern Rust graphics ecosystem.

## Why not tui-rs (chose ratatui)

tui-rs was deprecated in 2023 when its maintainer stepped down; ratatui is
the community-maintained continuation and the de-facto standard in 2026.

## Why not Haskell bindings (chose Python-only)

Per user decision 2026-04-14: removed from scope. Python covers the creative
scripting layer; Eclusa and other consumers use either the Python layer or
the Rust crate directly. Haskell FFI was retained as a speculative option in
the original manifesto and has been fully scrubbed.
```

## 9. `project.md` — Rewritten (Haskell-scrubbed, ≤ half the length)

Replacement for the current 343-line `project.md` at repo root. New version is ~150 lines (under half), keeps the manifesto tone, drops the Haskell section entirely, trims Phase 5 to a brief parked mention, and uses `tachyonfx` throughout with no `tui-vfx` references (the rationale sits in `docs/decisions/stack-rationale.md` per Item 8).

```markdown
# happyterminals

**Terminal art should feel like magic, not plumbing.**

A declarative, reactive terminal scene manager with GPU-quality effects
rendered as pure text. Runs on every terminal ever made — from Windows
Terminal to GNOME Terminal to macOS Terminal.app to SSH into a Raspberry Pi.

---

## The Stack

```
┌─────────────────────────────────────────────────┐
│  Reactive Runtime (signals → re-render)          │
│  SolidJS-style fine-grained, not VDOM diffing    │
├─────────────────────────────────────────────────┤
│  Pipeline + tachyonfx (effects composition)      │
│  50+ effects, DSL, WASM editor, composable       │
├─────────────────────────────────────────────────┤
│  Fresh 3D Renderer (ASCII rasterizer)            │
│  z-buffer, lighting, OBJ/STL support             │
├─────────────────────────────────────────────────┤
│  Ratatui Backend (terminal I/O via crossterm)    │
│  cursor, colors, resize, input — the boring bits │
└─────────────────────────────────────────────────┘
```

### Why this layering?

- **Ratatui** handles the boring terminal stuff (cursor, colors, resize, input)
- **tachyonfx** handles the cinematic stuff (50+ effects, DSL, compositing)
- **Our renderer** handles the 3D stuff (mesh rendering, projection, lighting)
- **Reactive runtime** handles state management (signals, effects, memoization)
- **DSL** makes it pleasant to use (declare what you want, not how to draw it)

---

## Design Principles

### 1. Declarative, not imperative

```rust
// NOT this:
fn render(frame: &mut Frame) {
    clear_screen();
    draw_cube(40, 12, t * 0.5);
    apply_dissolve(0.7);
    flush();
}

// THIS:
let scene = scene()
    .layer(|l| l.cube().rotation(&rot).position(vec3(0., 0., 0.)))
    .effect(fx::dissolve(0.7))
    .build()?;
```

### 2. Reactive, not polling

Inspired by SolidJS, not React. No virtual DOM. No diffing.

- **Signals** hold state. When a signal changes, only the cells that read
  it re-render. Fine-grained, surgical updates.
- **Effects** run when dependencies change. Side effects are explicit.
- **Memos** cache derived computations. Expensive math runs once.

### 3. Pure text output = universal terminal support

No GPU shaders. No LD_PRELOAD hacks. No special terminal required.

The pipeline operates on a `Grid` (cells with graphemes + colors). Effects
transform grids. Output is ANSI escape sequences.

This means:
- Works over SSH
- Works in Windows Terminal, GNOME, macOS Terminal.app, iTerm2, Kitty
- Works in tmux and screen
- Degrades gracefully on limited terminals (no color → ASCII silhouette)

### 4. Composable effects pipeline

Every effect is a `Grid → Grid` transform. Chain them. Nest them:

```rust
let pipeline = Pipeline::new()
    .push(render_3d(scene, camera))
    .push(fx::vignette(0.3))
    .push(fx::color_ramp("synthwave"))
    .push(fx::typewriter(2));
```

### 5. Rust-first, Python-final

The hot path (rendering, compositing, 3D projection) lives in Rust.
The creative path (scene description, signal wiring, effect composition)
is ergonomic in Rust and optional in Python via PyO3 bindings
(final milestone). Users pick the language; the engine is the same.

### 6. JSON recipes for AI generation

Scene recipes are pure data. An LLM can generate them; a human can
hand-edit them; both are valid:

```json
{
  "scene": {
    "objects": [
      {"type": "cube", "rotation_speed": 0.5}
    ],
    "effects": [
      {"type": "vignette", "strength": 0.3},
      {"type": "color_ramp", "palette": "dracula"}
    ]
  }
}
```

---

## Components

- **`happyterminals-core`** — reactive primitives (Signal, Memo, Effect, Owner), Grid buffer.
- **`happyterminals-renderer`** — 3D projection, z-buffer, ASCII shading, OBJ/STL loading.
- **`happyterminals-pipeline`** — Effect trait, Pipeline executor, tachyonfx adapter.
- **`happyterminals-scene`** — Scene IR, scene graph, transitions.
- **`happyterminals-dsl`** — Rust builder API, JSON recipe loader.
- **`happyterminals-backend-ratatui`** — ratatui/crossterm event loop, panic-safe terminal guard.
- **`happyterminals`** — meta crate: curated public surface + prelude.
- **`happyterminals-py`** (final milestone) — Python bindings via PyO3.

---

## Roadmap

### Milestone 0 — Workspace cleanup (current)

Clean build, dual-license, vendor hygiene, CI baseline. Blocker for everything else.

### Milestone 1 — Spinning Cube Demo

Signal-driven ASCII cube with one tachyonfx effect, rendered via ratatui, verified on
Windows Terminal / GNOME / macOS Terminal / iTerm2 / Kitty / Alacritty / tmux / SSH.

### Milestone 2 — Renderer Depth

OBJ mesh loading, particle system, color-mode pipeline (truecolor → 256 → 16 → mono),
cross-terminal resize hardening.

### Milestone 3 — Scene graph + JSON + v1 crates.io release

TransitionManager, JSON recipe loader with validated schema, 5+ examples,
seven crates published to crates.io under `MIT OR Apache-2.0`.

### Milestone 4 — Python bindings (FINAL)

`pip install happyterminals` on Linux / macOS / Windows. abi3 wheels for CPython
3.10–3.13. Type stubs. Sync `run(scene, fps=30)` as the primary entry.

### Parked (post-v1)

Audio-reactive scenes, AI scene generation, live-coding REPL, shader-to-ASCII
transpiler, multi-terminal scenes, WASM runtime. Revisited after v1 ships and
user feedback lands.

---

## Related Projects

| Project | Role |
|---------|------|
| [ratatui](https://github.com/ratatui/ratatui) | Terminal I/O, buffer model, widget system |
| [tachyonfx](https://github.com/ratatui/tachyonfx) | Effects library we build on |
| [SolidJS](https://www.solidjs.com/) | Reactive signal model (not the implementation) |
| [reactive_graph](https://crates.io/crates/reactive_graph) | Leptos's reactive core — we wrap this |

---

## Name

**happyterminals** — because terminals should make you happy.

Not `sad-terminals`. Not `terminal-hell`. Not `ncurses-ptsd`.

Happy. Terminals.
```

**Length:** ~150 lines vs 343 original (44% of original, comfortably under "half"). All forbidden strings removed. The "Why not tui-vfx" paragraph in the original is NOT reproduced here — it lives in `docs/decisions/stack-rationale.md`.

## 10. `README.md` — Rewritten

Replaces the current 16-line stub. Reflects current crate names (`-pipeline`, not `-compositor`), correct dep claims (no `tui-vfx`, no `pyo3` in `-core`), adds License section with Apache-2.0 §5 clause and status-badge placeholders.

```markdown
# happyterminals

> Terminal art should feel like magic, not plumbing.

[![CI](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml/badge.svg)](https://github.com/lynxnathan/happyterminals/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](rust-toolchain.toml)

A declarative, reactive terminal scene manager with composable visual effects
and ASCII 3D rendering. Pure text output — runs on every terminal ever made.

**Status:** Pre-alpha. Phase 0 (workspace hygiene) is in progress.
The first milestone is a signal-driven spinning cube demo.

## Stack

- **[ratatui](https://ratatui.rs/)** for terminal I/O (via crossterm)
- **[tachyonfx](https://github.com/ratatui/tachyonfx)** for the effects library
- **[reactive_graph](https://crates.io/crates/reactive_graph)** (Leptos's reactive core) for fine-grained signals
- **[glam](https://crates.io/crates/glam)** for 3D math
- Fresh ASCII rasterizer (not a fork of any existing renderer)

See [`project.md`](./project.md) for the design manifesto
and [`docs/decisions/stack-rationale.md`](./docs/decisions/stack-rationale.md)
for "why not X" decisions.

## Crates

| Crate | Role |
|-------|------|
| `happyterminals-core` | Reactive primitives (Signal, Memo, Effect) and Grid buffer |
| `happyterminals-renderer` | 3D projection, z-buffer rasterization, mesh loading |
| `happyterminals-pipeline` | Effect trait, Pipeline executor, tachyonfx adapter |
| `happyterminals-scene` | Scene IR and scene graph |
| `happyterminals-dsl` | Rust builder API + JSON recipe loader |
| `happyterminals-backend-ratatui` | Event loop + panic-safe TerminalGuard |
| `happyterminals` | Meta crate — curated public surface (`use happyterminals::prelude::*`) |
| `happyterminals-py` | Python bindings (Milestone 4, not yet activated) |

## Quick Start

> This is the target shape; the spinning-cube example ships with Milestone 1.

```rust
use happyterminals::prelude::*;

fn main() -> Result<()> {
    let rotation = signal(0.0_f32);
    let scene = scene()
        .layer(|l| l.cube().rotation(&rotation))
        .effect(fx::vignette(0.3))
        .build()?;

    run(scene, FrameSpec::fps(30))
}
```

## Development

Requires Rust 1.86+ (pinned via `rust-toolchain.toml`; `rustup` auto-installs).

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Docs, duplicate-dep scan, and unused-dep scan:

```bash
cargo doc --workspace --no-deps
cargo tree --workspace --duplicates
cargo install cargo-machete && cargo machete
```

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Contributions are welcome and
will be dual-licensed under the project's terms — see the License section
below.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](./LICENSE-MIT) or https://opensource.org/license/mit)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
```

**Note:** the CI badge URL assumes the repo is at `github.com/lynxnathan/happyterminals`. If the final owner differs, the badges need updating — the planner should make this a deliberate step rather than silently commit a wrong URL.

## 11. Per-Crate `README.md` Templates

Each crate gets a short one-liner README. Live at `crates/<name>/README.md` and are pointed at by `Cargo.toml`'s `readme = "README.md"` field (used by crates.io and docs.rs).

### `crates/happyterminals/README.md`
```markdown
# happyterminals

Meta crate for the [happyterminals](https://github.com/lynxnathan/happyterminals)
scene manager. Re-exports the curated public surface under `happyterminals::prelude::*`.

Most users should depend on this crate rather than the individual sub-crates.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-core/README.md`
```markdown
# happyterminals-core

Foundational types for [happyterminals](https://github.com/lynxnathan/happyterminals):
reactive primitives (`Signal`, `Memo`, `Effect`, `Owner`) wrapping `reactive_graph`,
and the `Grid` buffer (newtype over `ratatui::Buffer`, grapheme-cluster aware).

Depends on `ratatui-core` only, never the full `ratatui` facade or a backend.
`pyo3` is NOT a dependency of this crate — Python bindings live in
`happyterminals-py`.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-renderer/README.md`
```markdown
# happyterminals-renderer

ASCII 3D rasterizer for [happyterminals](https://github.com/lynxnathan/happyterminals):
perspective projection with configurable cell aspect ratio, reversed-Z buffer,
configurable shading ramp, OBJ/STL mesh loading, particle-system infrastructure.

Fresh implementation — not a fork of any existing 3D ASCII renderer.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-pipeline/README.md`
```markdown
# happyterminals-pipeline

Effect pipeline for [happyterminals](https://github.com/lynxnathan/happyterminals).
Defines the `Effect` trait (`apply(&mut self, grid: &mut Grid, dt: Duration) -> EffectState`),
the `Pipeline` executor (a `Vec<Box<dyn Effect>>` so JSON recipes and Python can
construct pipelines at runtime), and `TachyonAdapter` which wraps any `tachyonfx`
shader as one of our `Effect` trait objects.

`tachyonfx::Effect` is aliased as `Fx` in our public surface to disambiguate
the two `Effect` names.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-scene/README.md`
```markdown
# happyterminals-scene

Scene IR and scene graph for [happyterminals](https://github.com/lynxnathan/happyterminals).
One intermediate representation (`SceneIr`) consumed by every front-end — Rust builder,
JSON recipes, and (future) Python. Supports layered composition with explicit z-order,
signal-driven prop bindings, and the `TransitionManager` for cross-scene transitions.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-dsl/README.md`
```markdown
# happyterminals-dsl

Declarative builder for [happyterminals](https://github.com/lynxnathan/happyterminals)
scenes: a `react-three-fiber`-shaped tree of typed nodes whose props can be plain
values, `Signal<T>`, or `Memo<T>`. Ships a JSON recipe loader that validates
against a `schemars`-generated schema via `jsonschema`, then produces the
identical `SceneIr` as the Rust builder path.

Dual-licensed under MIT OR Apache-2.0.
```

### `crates/happyterminals-backend-ratatui/README.md`
```markdown
# happyterminals-backend-ratatui

Runtime backend for [happyterminals](https://github.com/lynxnathan/happyterminals).
Drives a `tokio::select!` loop between a frame ticker and `crossterm::EventStream`.
Provides `TerminalGuard` (RAII + panic hook) that restores the terminal (cursor,
raw mode, alternate screen, mouse capture, SGR state) on panic or early return —
Ctrl-C never leaves a trashed shell.

Dual-licensed under MIT OR Apache-2.0.
```

## 12. `CONTRIBUTING.md`

```markdown
# Contributing to happyterminals

Thanks for considering a contribution! happyterminals is a dual-licensed
(MIT OR Apache-2.0) Rust workspace. This document covers how to get set up,
how the project is organized, and how contributions are licensed.

## Prerequisites

- Rust 1.86 or newer (`rust-toolchain.toml` pins 1.86; `rustup` auto-installs).
- `cargo-machete` for unused-dep detection: `cargo install cargo-machete`.
- `ripgrep` (for the doc-lint script).
- (Optional) `cargo-insta` for snapshot test review: `cargo install cargo-insta`.

## Building and testing

```bash
# Full workspace build
cargo build --workspace

# Tests
cargo test --workspace --all-features

# Lints
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Docs
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Hygiene
cargo tree --workspace --duplicates --edges normal
cargo machete
bash scripts/doc-lint.sh
```

CI (`.github/workflows/ci.yml`) runs every check on each push and PR;
everything above must pass before merge.

## Workspace layout

See the table in the top-level README. In brief: `-core` defines foundational
types, everything else builds on top. `happyterminals-py` (Milestone 4) is the
only crate allowed to depend on `pyo3` — never add `pyo3` to any other member.

## Forbidden strings

The doc-lint CI step fails on these terms outside of
`docs/decisions/stack-rationale.md` (and vendored reference copies):

- `tui-vfx` (use `tachyonfx`)
- `Haskell bindings` (descoped)
- `pyo3-asyncio` (use `pyo3-async-runtimes`)
- `cgmath` (use `glam`)
- `tui-rs` (use `ratatui`)

If you genuinely need to mention one of these in prose (comparing to a prior
art, explaining why we avoid it), add it to `docs/decisions/stack-rationale.md`.

## Commit style

- Keep commits focused. One logical change per commit.
- First line ≤ 72 characters, imperative mood (e.g., `Add TerminalGuard panic hook`).
- Reference phase/requirement IDs where applicable (e.g., `HYG-05: add LICENSE files`).

## License

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as

    MIT OR Apache-2.0

without any additional terms or conditions.
```

**Note on the contribution clause wording:** the boilerplate above is the exact Rust-ecosystem standard (serde, tokio, regex, tracing, ratatui all use this wording verbatim). Do not rephrase — legal reviewers grep for this exact paragraph.

## 13. `CHANGELOG.md` — Keep-a-Changelog Skeleton

Live at repo root. Conforms to [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Pre-1.0 versioning: minor bumps (0.X.0) may contain breaking changes per Cargo's
semver interpretation; patch bumps (0.0.X) are fixes only.

## [Unreleased]

### Added
- (Phase 1.0+ additions accumulate here.)

### Changed

### Removed

## [0.0.0] — 2026-04-14

Initial workspace scaffolding (Phase 0: Workspace Hygiene & Foundation).

### Added
- Dual-license files at repo root: `LICENSE-MIT` and `LICENSE-APACHE`, with every
  crate's `Cargo.toml` carrying SPDX `MIT OR Apache-2.0`.
- `[workspace.dependencies]` block pinning every shared crate version
  (ratatui 0.30, tachyonfx 0.25, glam 0.32.1, reactive_graph 0.2.13, pyo3 0.28.3,
  and the supporting cast). All member crates inherit via `dep.workspace = true`.
- `rust-toolchain.toml` pinned to Rust 1.86 with `clippy` + `rustfmt` components.
- New workspace members: `happyterminals` (meta), `happyterminals-scene`,
  `happyterminals-dsl`, `happyterminals-backend-ratatui`. Placeholder for
  `happyterminals-py` (activated in Milestone 4).
- `.gitattributes` marking `vendor/_reference/**` as `linguist-vendored=true`.
- `scripts/doc-lint.sh` that fails CI on forbidden strings outside allowlist.
- `.github/workflows/ci.yml`: fmt, clippy `-D warnings`, test (Rust 1.86 + stable),
  `cargo doc -D warnings`, duplicate-dep scan, `cargo-machete`, doc-lint.
- `CONTRIBUTING.md` with Apache-2.0 §5 contribution clause.
- `docs/decisions/stack-rationale.md` as the sole allowlist for forbidden terms.

### Changed
- Renamed crate `happyterminals-compositor` → `happyterminals-pipeline` (the
  term `Pipeline` is used throughout the roadmap and API design; `compositor`
  was a legacy name).
- Rewrote `README.md` and `project.md` to reflect the current stack (no stale
  `tui-vfx` claims; no Haskell-bindings references).
- Vendored reference copies moved from `vendor/<name>/` to `vendor/_reference/<name>/`
  with `STAMP.txt` provenance files; never referenced via `path =` dependencies.
- Workspace `resolver` upgraded from `"2"` to `"3"` to match `edition = "2024"`.

### Removed
- Speculative dependencies from stub crates (`-core` had `pyo3`, `tui-vfx`,
  `ratatui`; `-compositor` had `tui-vfx`). Stubs now carry no dependencies
  until real call sites land in Phase 1.0+.

[Unreleased]: https://github.com/lynxnathan/happyterminals/compare/v0.0.0...HEAD
[0.0.0]: https://github.com/lynxnathan/happyterminals/releases/tag/v0.0.0
```

**Note:** the `0.0.0` version is deliberate — it signals "pre-1.0 scaffolding, do not consume yet." Cargo accepts `0.0.0`. First user-visible release (Milestone 1 / spinning cube) should bump to `0.1.0`.

## 14. Rename Strategy: `compositor` → `pipeline`

Verified prior to writing this plan: only three files reference the old name outside `.eclusa/` and `vendor/`:

```
./Cargo.toml
./README.md
./crates/happyterminals-compositor/Cargo.toml
```

No Rust source code imports `happyterminals_compositor` yet — the stub `lib.rs` defines a `Compositor` struct but no external crate uses it. This means the rename is purely filesystem + text. No `use` statement rewrites needed.

### Exact command sequence

```bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

# 1. Directory rename (git-tracked).
git mv crates/happyterminals-compositor crates/happyterminals-pipeline

# 2. Update the crate's own Cargo.toml name field.
#    (This file is the one just moved; edit it in place.)
#    Using sed for deterministic in-place edit; the planner may use the Edit tool.
sed -i 's/^name = "happyterminals-compositor"$/name = "happyterminals-pipeline"/' \
    crates/happyterminals-pipeline/Cargo.toml

# 3. Update the crate's description field (was "compositor", should now reference pipeline).
#    The Phase 0 per-crate Cargo.toml template (Item 4) already has the correct
#    description; use that template wholesale rather than patching.

# 4. Update root Cargo.toml workspace members list.
#    The full replacement is in Item 2. sed-based patch for clarity:
sed -i 's|"crates/happyterminals-compositor"|"crates/happyterminals-pipeline"|' Cargo.toml

# 5. Update README.md crates table.
#    The full replacement is in Item 10; sed-based patch not needed since
#    the README.md is rewritten wholesale.

# 6. Rename the stub `Compositor` struct to `Pipeline` in the moved lib.rs
#    (or replace the whole file with the Item 15 template).

# 7. Rename the lib crate name inside the crate's Cargo.toml IF a [lib] section
#    with an explicit `name = ...` exists. (Current Cargo.toml does not have
#    this, so no action needed — Cargo derives the lib name from package name.)

# 8. Confirm no lingering references:
rg --hidden --glob '!target/**' --glob '!vendor/**' --glob '!.eclusa/**' \
   --glob '!.git/**' 'compositor' || echo "(clean)"

# 9. Build gate:
cargo check --workspace
```

### Places to verify are updated (checklist)

- [x] `crates/happyterminals-compositor/` directory → `crates/happyterminals-pipeline/`
- [x] `crates/happyterminals-pipeline/Cargo.toml`: `name = "happyterminals-pipeline"`
- [x] `Cargo.toml` (root): workspace `members` array
- [x] `README.md`: crates table
- [ ] `project.md`: component list — rewrite (Item 9) already uses `-pipeline`
- [ ] Per-crate READMEs — Item 11 templates use `-pipeline`
- [ ] `CHANGELOG.md` — Item 13 documents the rename
- [x] Rust source: no `use happyterminals_compositor` exists (verified — no external imports). The internal `Compositor` struct inside the moved crate is replaced by the `lib.rs` stub in Item 15.

### `.eclusa/` references

The `.eclusa/ROADMAP.md` references `compositor` in a few places (e.g., the old workspace-layout sketch in STACK.md §6.1). Those are historical planning artifacts — the doc-lint allowlist already excludes `.eclusa/**` (see Item 8), and they should NOT be rewritten as part of Phase 0 (that's planning-doc churn, not code). The current PROJECT.md and ROADMAP.md already correctly say `-pipeline`.

### Why not keep `compositor`?

User locked this in CONTEXT.md (HYG-03 crate scaffold). The roadmap, requirements, and architecture research all say `-pipeline`. The only drift is the stub crate name, so we fix the stub.

## 15. Empty-Crate `lib.rs` Scaffolding

Minimal content for each new or reset crate. One doc comment per crate explaining its purpose and pointing to `PROJECT.md`; no actual code. Each compiles cleanly under `cargo check` with `forbid(unsafe_code)`, `missing_docs` warn, and clippy pedantic.

### `crates/happyterminals-core/src/lib.rs` (RESET — replace current 7-line `mod`-only file)

```rust
//! # happyterminals-core
//!
//! Foundational types for the [happyterminals](https://github.com/lynxnathan/happyterminals)
//! scene manager:
//!
//! - **Reactive primitives** — `Signal<T>`, `Memo<T>`, `Effect`, `Owner`, wrapping
//!   [`reactive_graph`] behind a happyterminals-owned public surface.
//! - **Grid buffer** — grapheme-cluster-aware cell grid, newtyped over
//!   [`ratatui_core::buffer::Buffer`] (compatibility verified in Phase 1.1).
//!
//! Phase 0 scaffolding — no public types yet. Implementations land in Phase 1.0
//! (reactive primitives) and Phase 1.1 (Grid).
//!
//! See `.eclusa/PROJECT.md` and `.eclusa/ROADMAP.md` for the full design.
```

**Action:** delete the existing `grid.rs`, `python.rs`, `reactive.rs` modules — per CONTEXT.md HYG-02 they are speculative and premature. The reactive and Grid types land properly in Phases 1.0 and 1.1. Leaving stub implementations in place creates exactly the kind of drift HYG-02 exists to prevent.

### `crates/happyterminals-renderer/src/lib.rs`

```rust
//! # happyterminals-renderer
//!
//! Fresh ASCII 3D rasterizer for [happyterminals](https://github.com/lynxnathan/happyterminals):
//! perspective projection with configurable cell aspect ratio, reversed-Z buffer,
//! configurable ASCII shading ramp, OBJ/STL mesh loading, and particle infrastructure.
//!
//! Not a fork of any existing renderer — see the "fresh implementation" decision
//! in `.eclusa/PROJECT.md` §"Key Decisions".
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.3
//! (minimal cube primitive) and Phases 2.1–2.4 (OBJ, particles, color pipeline).
```

### `crates/happyterminals-pipeline/src/lib.rs` (renamed from compositor)

```rust
//! # happyterminals-pipeline
//!
//! Effect pipeline for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! Defines the `Effect` trait (`apply(&mut self, grid: &mut Grid, dt: Duration)`),
//! the `Pipeline` executor (`Vec<Box<dyn Effect>>` so JSON recipes and Python can
//! construct pipelines at runtime), and `TachyonAdapter` — which wraps any
//! [`tachyonfx`] shader as one of our `Effect` trait objects.
//!
//! To disambiguate the two `Effect` names, `tachyonfx::Effect` is re-exported
//! as `Fx` in our public surface. See `.eclusa/research/PITFALLS.md` §16.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.2.
```

### `crates/happyterminals-scene/src/lib.rs`

```rust
//! # happyterminals-scene
//!
//! Scene IR and scene-graph types for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! One intermediate representation (`SceneIr`) is the target of every front-end —
//! Rust builder, JSON recipes, and (future) Python. The scene graph supports
//! layered composition with explicit z-order and signal-driven prop bindings.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.4;
//! full `TransitionManager` in Phase 3.1.
```

### `crates/happyterminals-dsl/src/lib.rs`

```rust
//! # happyterminals-dsl
//!
//! Declarative scene builder for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! A `react-three-fiber`-shaped tree of typed nodes with props that can be plain
//! values, `Signal<T>`, or `Memo<T>`. Also ships the JSON recipe loader that
//! validates input against a `schemars`-generated schema via `jsonschema`, then
//! produces the same `SceneIr` as the Rust builder path.
//!
//! Phase 0 scaffolding — no public types yet. Rust builder lands in Phase 1.4;
//! JSON recipes in Phases 3.2–3.4.
```

### `crates/happyterminals-backend-ratatui/src/lib.rs`

```rust
//! # happyterminals-backend-ratatui
//!
//! Runtime backend for [happyterminals](https://github.com/lynxnathan/happyterminals).
//!
//! Drives a `tokio::select!` loop between a frame ticker and `crossterm::EventStream`.
//! Provides `TerminalGuard` (RAII + panic hook) that restores the terminal — cursor
//! visibility, raw mode, alternate-screen buffer, mouse capture, and SGR state — on
//! panic or early return. Ctrl-C never leaves a trashed shell.
//!
//! Phase 0 scaffolding — no public types yet. Implementation lands in Phase 1.1.
```

### `crates/happyterminals/src/lib.rs` (NEW — meta crate)

```rust
//! # happyterminals
//!
//! Meta crate for the [happyterminals](https://github.com/lynxnathan/happyterminals)
//! scene manager. Re-exports a curated public surface so most users can write:
//!
//! ```ignore
//! use happyterminals::prelude::*;
//! ```
//!
//! Phase 0 scaffolding — no re-exports yet. The prelude lands in Phase 1.1
//! (after `-backend-ratatui` ships its `run` entry point).

/// Curated re-export module.
///
/// Empty during Phase 0; populated in Phase 1.1+.
pub mod prelude {}
```

### Verification

All seven stubs compile under this check:

```bash
cargo check --workspace
cargo doc --workspace --no-deps  # must pass RUSTDOCFLAGS="-D warnings"
```

With `#![forbid(unsafe_code)]` implicitly via the workspace lint table, `missing_docs = "warn"`, and the satisfying top-level doc comment, no warnings fire.

## 2026 Convention Flags (what differs from older training data)

| Area | Older convention | 2026 convention | Why |
|------|------------------|-----------------|-----|
| Workspace resolver | `resolver = "2"` | `resolver = "3"` | Stabilized in Rust 1.85 (2025-02-20) alongside edition 2024. Current root `Cargo.toml` has `"2"` — **MUST upgrade**. |
| Unused-dep scanner | `cargo-udeps` (nightly-only, slower, compiler-backed) | `cargo-machete` (stable, fast, string-based) for CI; udeps as opt-in deep scan | Our `rust-toolchain.toml` pins stable 1.86 — udeps does not work without nightly. |
| GHA Rust cache | `actions/cache` with manual paths | `Swatinem/rust-cache@v2` | Universal in the Rust ecosystem; pin to `@v2` floating. No v3. |
| GHA Rust toolchain | `actions-rs/toolchain` (archived) | `dtolnay/rust-toolchain@master` | `actions-rs/*` actions are unmaintained; dtolnay is the community default. |
| Install cargo subcommands | `cargo install cargo-machete` (slow) | `taiki-e/install-action@v2` (prebuilt binaries) | Seconds instead of minutes; standard 2026 pattern. |
| Workspace lints | `.cargo/config.toml` rustflags / per-crate `#![deny(...)]` | `[workspace.lints]` table in root `Cargo.toml` + `lints.workspace = true` per crate | Stabilized in 1.74; now the idiomatic way to DRY lints across a workspace. |
| Edition | `edition = "2021"` | `edition = "2024"` | 2024 is stable, supports let-chains, improved async closures, `gen` blocks. All workspace crates should use 2024. Note: per-crate `edition` must be ≤ workspace `rust-version` supports — 1.86 supports 2024. |
| `pyo3-asyncio` | Widely recommended in pre-2024 tutorials | **Abandoned** — use `pyo3-async-runtimes` | STACK.md §4.2 and PITFALLS §18 both flag this. The doc-lint rule (Item 8) enforces it. |
| Clippy lint groups | `deny(clippy::all)` | `[workspace.lints.clippy] all = { level = "warn", priority = -1 }` with targeted `deny` for specific lints | Priority ordering lets you warn a group but deny specific members. New in recent Cargo versions. |
| `cargo tree --duplicates` | Bare `cargo tree -d` | `cargo tree --workspace --duplicates --edges normal` | `--edges normal` excludes dev/build edges that legitimately have their own versions; `--workspace` covers every member. |
| Rust 1.86 new lints | N/A (wasn't released yet in training) | `double_negations` now a builtin lint (migrated from clippy); `missing_abi` now warns by default | No action required — our workspace already forbids unsafe and denies clippy unwrap/expect; these two extras are free. |

## Runtime State Inventory (not applicable)

Phase 0 is not a rename/refactor/migration of **runtime-stored** state — the "compositor → pipeline" rename is purely filesystem + `Cargo.toml` text. There are no databases, live service configs, OS-registered tasks, or secrets that embed the old crate name. The `Cargo.lock` at the workspace root will regenerate on the next `cargo build`; it's a build artifact and will pick up the new member name automatically.

- **Stored data:** None. No databases exist; the project is pre-alpha.
- **Live service config:** None. No deployed services.
- **OS-registered state:** None. No systemd units, Task Scheduler entries, or pm2 processes.
- **Secrets/env vars:** None. No `.env` files, no CI secrets yet (only the `GITHUB_TOKEN` GHA provides).
- **Build artifacts:** `target/` is gitignored and will regenerate. `Cargo.lock` at root regenerates on `cargo build` and will reference `happyterminals-pipeline` by its new name — no manual intervention.

## Environment Availability

Phase 0 runs on the developer's local machine and on GHA's `ubuntu-latest`. Dependencies:

| Dependency | Required By | Available locally | Version | Fallback |
|------------|-------------|-------------------|---------|----------|
| Rust 1.86+ | All workspace tasks | ✓ (local `.tool-versions` pins 1.92) | 1.86 pin via `rust-toolchain.toml` | None — required |
| `rustup` | Toolchain install | ✓ | n/a | None — required |
| `cargo` | Everything | ✓ (bundled with rustup) | matches Rust version | None |
| `cargo-machete` | CI hygiene job | Not locally installed | latest 2026 | Can defer to CI-only |
| `ripgrep` (`rg`) | `scripts/doc-lint.sh` | ✓ (ubuntu-latest ships it; user's shell likely has it) | ≥ 13 | None — required (script uses `rg --glob` which needs rg) |
| Git | Vendor relocation, commits | ✓ | any recent | None — required |
| `taiki-e/install-action@v2` | GHA cargo-machete install | N/A (GHA-side) | v2 | `cargo install cargo-machete` in CI (slower but equivalent) |

**Missing dependencies with fallback:**
- `cargo-machete` local: developers can run `bash scripts/doc-lint.sh` without it; `cargo machete` is a CI step they can also run locally after `cargo install cargo-machete`. Non-blocking locally.

**Missing dependencies with no fallback:**
- None. Ubuntu and any modern Linux dev environment have everything required.

## Validation Architecture

Nyquist validation is not explicitly disabled — treat as enabled. Phase 0 is operational cleanup without new code behaviors, but several phase deliverables have automated tests.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in); `insta` 1.47 for snapshots (dev-dep); `proptest` 1.11 for properties (dev-dep). **Phase 0 does not add test code — HYG-* requirements are validated by shell assertions and CI green status, not unit tests.** |
| Config file | None needed; each crate's `src/lib.rs` can carry `#[cfg(test)]` modules when code lands in Phase 1.0+. |
| Quick run command | `cargo test --workspace` (passes trivially in Phase 0 — no tests yet) |
| Full suite command | `cargo test --workspace --all-features` |
| Phase gate | All CI jobs green on `main` |

### Phase Requirements → Test Map

Phase 0 requirements are verified by **CI job outcomes** and **shell assertions**, not by Rust unit tests. The table below maps each requirement to the verification mechanism.

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HYG-01 | No forbidden strings outside allowlist | CI / shell | `bash scripts/doc-lint.sh` | ❌ Wave 0 — script and allowlist file must be created |
| HYG-02 | Stub crates have no speculative deps | CI / shell | `cargo machete && grep -c "workspace = true\|^\[" crates/*/Cargo.toml` | ✅ (cargo-machete will verify once installed in CI) |
| HYG-03 | Every member uses `{ workspace = true }` for every shared dep | CI / shell | `! rg '^(\w+)\s*=\s*"[0-9]' crates/*/Cargo.toml` (no version literals in member crates) | ✅ |
| HYG-04 | Vendor relocated with STAMP + `.gitattributes` | Shell | `test -d vendor/_reference/pyo3 && test -f vendor/_reference/pyo3/STAMP.txt && grep -q 'linguist-vendored' .gitattributes` | ❌ Wave 0 — files must be created |
| HYG-05 | LICENSE files + SPDX | Shell | `test -f LICENSE-MIT && test -f LICENSE-APACHE && rg -c 'license = "MIT OR Apache-2.0"' crates/*/Cargo.toml` | ❌ Wave 0 — LICENSE files must be created |
| HYG-07 | `rust-toolchain.toml` pins 1.86 | Shell | `grep -q 'channel = "1.86"' rust-toolchain.toml` | ❌ Wave 0 — file must be created |
| HYG-08 | CI baseline runs green | CI | GHA `ci.yml` all jobs green on `main` | ❌ Wave 0 — workflow must be created |
| HYG-09 | Doc-lint fails on forbidden strings | CI / shell | `bash scripts/doc-lint.sh` (exits non-zero on violation) | ❌ Wave 0 — script must be created |

### Sampling Rate

- **Per task commit:** `cargo check --workspace && cargo fmt --all -- --check && bash scripts/doc-lint.sh`
- **Per wave merge:** Full CI job set (fmt, clippy, test, docs, hygiene) as run by `.github/workflows/ci.yml`
- **Phase gate:** All seven CI jobs green on `main`; manual verification that `ls LICENSE-*` returns both files, `cargo tree --duplicates` prints nothing, and `rg tui-vfx` outside allowlist returns nothing.

### Wave 0 Gaps

Files that must be created before Phase 0 can run its verifications — create these first in a single initial wave:

- [ ] `LICENSE-MIT` — canonical text from Item 1
- [ ] `LICENSE-APACHE` — canonical text from Item 1
- [ ] `rust-toolchain.toml` — Item 3
- [ ] `.gitattributes` — Item 5
- [ ] `Cargo.toml` (root) — full rewrite from Item 2
- [ ] `crates/*/Cargo.toml` — per-crate templates from Item 4
- [ ] `crates/*/src/lib.rs` — stub scaffolding from Item 15
- [ ] `crates/*/README.md` — per-crate READMEs from Item 11
- [ ] `README.md` — full rewrite from Item 10
- [ ] `project.md` — full rewrite from Item 9
- [ ] `CONTRIBUTING.md` — Item 12
- [ ] `CHANGELOG.md` — Item 13
- [ ] `docs/decisions/stack-rationale.md` — Item 8
- [ ] `scripts/doc-lint.sh` — Item 8 (executable mode)
- [ ] `.github/workflows/ci.yml` — Item 7
- [ ] `vendor/_reference/pyo3/STAMP.txt` — Item 6
- [ ] `vendor/_reference/ratatui/STAMP.txt` — Item 6
- [ ] `vendor/_reference/tui-vfx/STAMP.txt` — Item 6

## Project Constraints (from CLAUDE.md)

This repo does not currently have a `./CLAUDE.md` — only a user-global instruction file referencing `RTK.md` (a token-optimizing CLI proxy). No project-specific coding or security constraints apply beyond what's locked in `.eclusa/` artifacts.

If a project-level `CLAUDE.md` is added later, re-read and incorporate.

## Sources

### Primary (HIGH confidence — verified 2026-04-14)

- `.eclusa/PROJECT.md`, `.eclusa/REQUIREMENTS.md`, `.eclusa/ROADMAP.md`, `.eclusa/STATE.md` — project ground truth
- `.eclusa/research/STACK.md` — all crate versions (verified against crates.io API on 2026-04-14)
- `.eclusa/research/PITFALLS.md` — §2, §3, §16, §18, §21, §22, §25, §26, §32, §33 directly inform Phase 0
- `.eclusa/phases/00-workspace-hygiene-foundation/00-CONTEXT.md` — locked decisions
- `https://www.apache.org/licenses/LICENSE-2.0.txt` — Apache-2.0 canonical text
- `https://opensource.org/license/mit` — MIT canonical text
- `https://doc.rust-lang.org/cargo/reference/workspaces.html` — workspace syntax reference
- `https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html` — resolver = "3" source
- `https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file` — `rust-toolchain.toml` schema
- `https://github.com/Swatinem/rust-cache` — v2.8.1 canonical CI cache (via web search)
- `https://github.com/taiki-e/install-action` — 2026-canonical cargo subcommand installer
- `https://doc.rust-lang.org/cargo/commands/cargo-tree.html` — `cargo tree --duplicates` flags
- `https://keepachangelog.com/en/1.1.0/` — CHANGELOG format

### Secondary (MEDIUM confidence — web search verified in this session)

- `https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/` — Rust 1.85 stabilizes edition 2024 + resolver 3
- `https://rustprojectprimer.com/checks/unused.html` — cargo-machete vs cargo-udeps (2026 comparison)
- `https://releases.rs/docs/1.86.0/` — Rust 1.86 release notes

### Not Consulted (intentional)

- Context7 was not queried this round; STACK.md already verified every crate version against live crates.io, and the unresolved questions in Phase 0 are structural (file layout, CI YAML), not API-level.

## Metadata

**Confidence breakdown:**

- License text (Items 1): HIGH — canonical text reproduced verbatim from the Apache Foundation and OSI; dozens of Rust crates use this exact wording (serde, tokio, regex, tracing, ratatui, tachyonfx).
- Cargo.toml block (Item 2): HIGH — every version already verified in STACK.md against crates.io on 2026-04-14; `resolver = "3"` and `edition = "2024"` cross-checked against the Rust Edition Guide.
- `rust-toolchain.toml` (Item 3): HIGH — schema is stable and minimal; pins locked in CONTEXT.md.
- Per-crate template (Item 4): HIGH — workspace inheritance syntax (`field.workspace = true`) is stable and documented in the Cargo reference.
- `.gitattributes` (Item 5): HIGH — `linguist-vendored=true` is documented in linguist's override docs and widely used by Rust ecosystem projects.
- STAMP.txt format (Item 6): MEDIUM — no universal standard for vendored-reference stamps; format synthesized from conventions in Debian source packages, Google's `METADATA` files for third_party dirs, and common practice in the Rust ecosystem.
- CI workflow (Item 7): HIGH — every action used is canonical and current as of 2026-04-14 (Swatinem/rust-cache@v2, dtolnay/rust-toolchain, taiki-e/install-action, cargo-machete).
- Doc-lint script (Item 8): HIGH — `rg` behavior and GitHub Actions `::error::` annotations are both stable.
- Project.md / README rewrites (Items 9, 10): HIGH — content mapped from PROJECT.md + ROADMAP.md + REQUIREMENTS.md; no new facts introduced.
- CONTRIBUTING / CHANGELOG (Items 12, 13): HIGH — idiomatic Rust-ecosystem templates.
- Rename strategy (Item 14): HIGH — verified against filesystem state (only three files reference the old name).
- Scaffolding lib.rs (Item 15): HIGH — doc comment syntax is stable; `mod prelude {}` is valid and warning-free with `#![warn(missing_docs)]`.

**Research date:** 2026-04-14
**Valid until:** 2026-07-14 (3 months — stack versions may drift; convention drift is unlikely before then). Re-verify Swatinem/rust-cache major version and any new clippy default-deny lints if revisiting.

---

*Phase: 00-workspace-hygiene-foundation*
*Research complete: 2026-04-14*
