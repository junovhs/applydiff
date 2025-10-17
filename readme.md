# ApplyDiff

Reliable, token-efficient code patching you can trust with AI outputs.  
Desktop app built with **Tauri v2** (Rust backend, vanilla HTML/CSS/JS frontend).

## Why this exists

LLMs are great at proposing changes, but not at the mechanical work of applying them. ApplyDiff takes AI-generated patches and applies them safely:
- No line numbers. Content/context matching only.
- Previews always before writes.
- Backups taken automatically.
- Safety rails for paths and endings.
- Partial apply: good blocks land, bad ones are skipped.

## Features (MVP)

**Patch format (simple & LLM-friendly)**

```
>>> file: relative/path/to/file.ext | fuzz=0.85
--- from
[exact text in the file]
--- to
[replacement text]
<<<
```

Notes:
- `fuzz` ∈ [0.0, 1.0]; 1.0 = exact only.
- Leave the `from` section empty to **create/append** a new file (append-create).
- Multiple blocks allowed back-to-back.

**Other features:**
- **Preview first**: unified diffs with green (+) and red (–).
- **Auto-preview**: on paste and after 3s idle while typing.
- **CRLF/LF preservation**: output endings harmonize to the matched region.
- **Path safety**: attempts to write outside the selected root are rejected.
- **Partial apply**: "Apply Valid Changes" when some blocks fail; backups before writes.
- **Backups**: saved next to your files: `.applydiff_backup_YYYYMMDD_*`.
- **Console**: structured log tail; **Clear Console** and **Run Self-Test**.
- **No bundler required**: simple static frontend.

## Build & Run

Prereqs: Rust toolchain + Tauri v2 prerequisites.

```bash
cargo tauri dev
```

## Using the App

1. **Select Directory** (enables editing).
2. **Click To Paste** (reads clipboard); if the clipboard is empty, a textarea opens for typing.
3. Preview appears automatically. If everything is green, **Apply Patch** appears.
   If some blocks are invalid (e.g., path escapes), you'll see **Apply Valid Changes** (orange).
4. After apply, backups are created automatically; preview re-runs against on-disk state.

## Safety semantics

**Matching**

* Fast path: exact substring match (logged as `fast_path_match`).
* Otherwise, layered search with a fuzzy threshold (`fuzz`).
* If no match ≥ threshold, the block is rejected (no write).

**Endings**

* The replacement's trailing newline is harmonized to the matched region (`\n` or `\r\n`).
* Files without a trailing newline remain without one unless the replacement explicitly adds it.

**Paths**

* Relative to the selected root.
* Any `..` traversal that would escape is rejected at preview/apply.

**Partial apply**

* Apply proceeds for blocks that pass; failures are reported per block.
* Backups are always taken first (timestamped folder next to the root).

## Quick Lab (manual smoke tests)

Create a local lab at `~/ApplyDiffLab`:

```bash
# See TESTING.md for the full script and scenarios A–F
```

Then use the patches in **TESTING.md**. Example mixed patch:

```
>>> file: readme.md | fuzz=1.0
--- from
Welcome to the lab.
--- to
Welcome to the patched lab.
<<<

>>> file: new/report.txt | fuzz=1.0
--- from
--- to
Report v1
<<<
```

After apply, you'll have `new/report.txt` containing:

```
Report v1
```

## Architecture

**Frontend:** `index.html` + inline JS. Talks to Tauri via `window.__TAURI__.core.invoke`.

**Rust backend (src-tauri):**

* `parser` → parses blocks.
* `apply` → matching & application (exact substring fast path + layered search).
* `backup` → timestamped backup folder.
* `tauri_commands.rs` → `preview_patch`, `apply_patch`, `pick_folder`, etc.
* Structured JSONL logs from subsystems (see console tail).

## Acknowledgments & Prior Art

ApplyDiff is an **independent, from-scratch implementation** in Rust. We were, however, informed and inspired by excellent public work in this space:

* **Aider** (Brett Koonce & contributors): demonstrated that *textual patch formats with flexible matching* work well for LLM-assisted editing. We borrowed ideas at a conceptual level:

  * Avoid line numbers; anchor by content and surrounding context.
  * Support multiple edit styles; prefer formats that force specificity.
  * Flexible matching/fallbacks (exact → normalization → fuzzy) improve reliability.

* **Cursor Fast Apply & two-stage workflows**: the idea of separating **planning** from **mechanical application** informed our roadmap (longer-term). ApplyDiff currently focuses on a robust local applier with strong safety rails.

* **Git unified diffs** and the broader ecosystem taught us the value of machine-consumable, deterministic formats and unified previews. Our preview uses unified diffs for readability.

* **Rust ecosystem crates & patterns**: standard library string handling, diffing approaches, and structured logging patterns influenced design. 

If we incorporate any external **code** (not just ideas) in the future, we will:

1. include a clear entry in **`NOTICE`** with upstream license,
2. keep original headers where required, and
3. link to the source and commit.

We're grateful to these projects for moving the field forward.

## Roadmap

* Flexible matching enhancements (whitespace & relative indentation normalization, hunk decomposition).
* Unified diff input support (optional), keeping search/replace as the default.
* Repo map context for prompt generation (token-efficient retrieval).
* Deeper gauntlet coverage (`LF/MA/FS` series — see TESTING.md).
