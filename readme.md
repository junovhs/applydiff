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
- Version history with notes for tracking changes.

## Features

**Patch format (simple & LLM-friendly)**

Type this format directly into the app or save to a .txt file:

```
PATCH relative/path/to/file.ext fuzz=0.85
FROM
exact text in the file
TO
replacement text
END
```

**Format details:**
- `fuzz`: 0.0 to 1.0 (1.0 = exact only, 0.85 default allows minor variations)
- Leave `FROM` section empty to create/append a new file
- Multiple blocks allowed back-to-back for multi-file patches
- Each block must end with `END` keyword

**Core features:**
- ✅ **Auto-paste**: Click patch area with clipboard full → instant paste
- ✅ **Preview first**: Unified diffs with syntax highlighting (green/red)
- ✅ **Auto-preview**: Triggers on paste and after 3s typing idle
- ✅ **Version history**: Each apply creates a new version tab (v1, v2, v3...)
- ✅ **Version notes**: Right-click any version tab to add/edit notes (shown with 📝)
- ✅ **Navigate versions**: Click tabs or use ← → arrows to view past patches
- ✅ **Smart matching**: Fast exact path, then layered fuzzy search with ambiguity detection
- ✅ **CRLF/LF preservation**: Output endings harmonize to matched region
- ✅ **Path safety**: Blocks escaping directory (../) rejected at preview
- ✅ **Partial apply**: Valid blocks apply, invalid skip; button shows "Apply Valid Changes"
- ✅ **Auto-backup**: Timestamped `.applydiff_backup_YYYYMMDD_HHMMSS` folders
- ✅ **Structured console**: Colored logs with timestamps, expandable with toggle
- ✅ **Self-test gauntlet**: Built-in test runner (currently 3/3 passing)

## Build & Run

Prereqs: Rust toolchain + Tauri v2 prerequisites.

```bash
cargo tauri dev
```

For release build:
```bash
cargo tauri build
```

## Using the App

1. **Select Directory** → Choose your project root
2. **Paste patch** → Click patch area (auto-pastes from clipboard) or type
3. **Review preview** → Green/red diff shows changes; console shows match details
4. **Apply** → Creates backup, applies changes, creates version tab
5. **Add notes** → Right-click version tabs to document what changed
6. **Navigate history** → Click v1, v2, v3... or use arrows to view past patches

## Safety Semantics

**Matching strategy:**
1. **Fast path**: Exact substring match (logged as `fast_path_match`)
2. **Layered search**: Whitespace normalization → relative indent → fuzzy
3. **Ambiguity detection**: Rejects patches matching multiple locations equally
4. **Threshold**: No match below `fuzz` score → block rejected (no write)

**Line endings:**
- Replacement text harmonizes to matched region (`\n` or `\r\n`)
- Files without trailing newline stay that way unless replacement adds one

**Path safety:**
- All paths relative to selected root
- Any `..` traversal rejected at preview/apply

**Partial apply:**
- Valid blocks apply, failures reported per-block
- Backups always taken before first write
- Console shows `N applied, M failed` summary

## Testing

**Automated gauntlet:** 3/3 tests passing
- ✅ **LF01-Replace-Start**: 50K line file, exact match, fast path verified
- ✅ **MA01a-Simple-Ambiguity**: Detects and rejects duplicate targets
- ✅ **MA01b-Indentation-Ambiguity**: Handles formatting variance

Run tests: Click **Run Self-Test** in console panel.

See **TESTING.md** for manual lab scenarios and test expansion roadmap.

## Architecture

**Frontend:** `public/index.html` - Vanilla HTML/CSS/JS, no bundler
- Tauri API via `window.__TAURI__.core.invoke`
- Version history stored in-memory (resets on app restart)
- Console uses DOM manipulation for colored log entries

**Rust backend:** `src-tauri/`
- `parser.rs` → Parses patch blocks with regex
- `matcher.rs` → Multi-strategy matching (exact → normalized → fuzzy)
- `apply.rs` → File operations, EOL harmonization, path validation
- `backup.rs` → Timestamped backup folders
- `gauntlet.rs` → Self-test framework with sandbox isolation
- `tauri_commands.rs` → Frontend API: `preview_patch`, `apply_patch`, `pick_folder`, etc.

**Logging:** Structured JSONL to stdout, subsystem/action/message format.

## Known Issues

1. **Chat UI patch display**: Some chat interfaces mangle patch syntax when rendering. Always copy patches from plain text or the app itself.
2. **Version history persistence**: Versions reset on app restart (in-memory only).
3. **Multi-block preview**: Preview shows combined diff for all blocks.

## Acknowledgments & Prior Art

ApplyDiff is an **independent, from-scratch implementation** in Rust, inspired by:

* **Aider** (Paul Gauthier): Conceptual inspiration for textual patch formats, content-based matching, and flexible fallbacks
* **Cursor Fast Apply**: Two-stage workflow (plan → apply) concept
* **Git unified diffs**: Machine-readable format standards
* **Rust ecosystem**: `similar`, `strsim`, `regex` crates for diffing and matching

We're grateful to these projects for advancing LLM-assisted editing.

## Roadmap

**Near-term:**
- [ ] Persist version history to disk
- [ ] Show current directory path in header
- [ ] Export version history to files
- [ ] Improved multi-block preview

**Medium-term:**
- [ ] Whitespace normalization improvements
- [ ] Hunk decomposition for large blocks
- [ ] Unified diff input support (optional)
- [ ] More gauntlet tests (LF02-05, FS01-04, MA02-05)

**Long-term:**
- [ ] Repo map generation for AI context
- [ ] Token-efficient context retrieval
- [ ] Integration with external diff tools
- [ ] Syntax highlighting in patch editor
