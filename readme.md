# applydiff

**Sleek desktop app for applying AI-generated patches with fuzzy matching and bulletproof git safety.**

Built for the modern workflow: AI writes code diffs, you paste them in, applydiff handles the rest. No more manual find-and-replace disasters.

![Status: Working](https://img.shields.io/badge/status-working-brightgreen) ![Platform: Cross-platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

---

## The Vision

**The Problem:** AI assistants like Claude generate code changes as text diffs. Manually applying them is error-prone and tedious. Most diff tools are clunky terminal apps or break on fuzzy matches.

**The Solution:** A beautiful, floating desktop app where you:
1. **Click** to select your project directory (must be a git repo)
2. **Paste** the AI-generated patch
3. **Preview** to see exactly what will change
4. **Apply** with one click - complete with automatic git safety commit

If anything goes wrong, undo is just `git reset --hard HEAD~1`. No corrupted files. No surprises.

---

## Features

### 🎨 **Sleek UI**
- Dark, modern interface with rounded corners
- Large text area for pasting patches
- Real-time output log showing progress
- Responsive buttons with gradient hover effects
- Native folder picker

### 🛡️ **Safety First**
- **Mandatory git safety commit** before applying (instant undo)
- **Rejects ambiguous matches** (multiple equally-good hits)
- **Preview mode** - see what will change before committing
- **Validates working tree** is clean before proceeding
- **Bounded operations** - no runaway loops from bad AI patches

### 🧠 **Smart Matching**
- **Fuzzy string matching** with configurable thresholds
- Handles slight variations in whitespace/formatting
- Reports confidence scores for all matches
- Fails fast on low-confidence matches
- Works even when AI gets the context slightly wrong

### 🔒 **Po10 Compliant**
- Zero recursion
- All loops bounded with runtime guards
- ≥2 assertions per function
- Structured JSONL error logging
- Zero warnings with strict lints
- Mission-critical code standards for patch application

---

## Installation

### Build from Source

```bash
# Clone the repo
git clone https://github.com/yourusername/applydiff
cd applydiff

# Build release binary
cargo build --release

# Run it
./target/release/applydiff
```

### Requirements
- Rust 1.70+
- Git installed and in PATH

---

## How to Use

### 1. **Launch the App**
```bash
./target/release/applydiff
```

A sleek dark window opens.

### 2. **Select Your Project**
Click **📁 Browse** and select your git repository directory.

### 3. **Paste Your Patch**
Copy an AI-generated patch and paste it into the large text area. Format:

```
>>> file: src/main.rs | fuzz=0.85
--- from
old code here
multiple lines ok
--- to
new code here
also multiple lines
<<<

>>> file: lib/utils.js | fuzz=0.9
--- from
another old snippet
--- to
replacement
<<<
```

**Options per block:**
- `fuzz=0.85` - Match threshold (0.0-1.0, default 0.8)

### 4. **Preview First**
Click **👁 Preview** to see:
- Which files will be patched
- Where matches were found (byte offset)
- Confidence scores for each match
- Any errors or warnings

### 5. **Apply the Patch**
If preview looks good, click **✓ Apply Patch**.

The app will:
1. ✅ Verify working tree is clean
2. ✅ Create automatic safety commit
3. ✅ Apply all patches with fuzzy matching
4. ✅ Show results in the output log

**Undo anytime:**
```bash
cd your-project
git reset --hard HEAD~1
```

---

## Safety Checks

applydiff will **refuse to run** if:
- Target directory is not a git repository
- Working tree has uncommitted changes
- Match is ambiguous (multiple ~equal hits)
- Match confidence is below threshold
- File exceeds size limits (10MB default)
- Input exceeds 100MB

---

## Error Handling

All errors include:
- **Stable machine-parseable codes** (1000-5999)
- **Structured JSON context**
- **Clear next steps**

Example error log (JSONL format):
```jsonl
{"ts":"2025-10-13T12:34:56Z","level":"error","rid":12345,"subsystem":"apply","action":"write_file","code":3002,"msg":"Failed to write file","context":{"path":"src/main.rs","error":"Permission denied"}}
```

### Error Code Ranges
```
1000-1999: Parse errors (malformed patch)
2000-2999: Match errors (not found, ambiguous, low score)
3000-3999: File I/O errors (read/write failures)
4000-4999: Git errors (not a repo, dirty state)
5000-5999: Validation errors (bounds exceeded)
```

---

## Configuration

### Safety Limits
Hard-coded bounds (adjust in source if needed):

```rust
MAX_BLOCKS: 1000              // Max patch blocks per file
MAX_LINES_PER_BLOCK: 10000    // Max lines in from/to sections
MAX_FILE_SIZE: 10MB           // Max file size to patch
MAX_INPUT_SIZE: 100MB         // Max total input size
```

### Match Thresholds
- Default: `0.8` (80% similarity required)
- Per-block override: add `| fuzz=0.85` to the file header
- Range: `0.0` (match anything) to `1.0` (exact match only)

---

## Design Philosophy

### Why a Desktop GUI?
**Paste and go.** No command-line arguments to remember. No file paths to type. Just click, paste, preview, apply.

### Why Fuzzy Matching?
AI-generated diffs often have slight context mismatches. Exact-match tools fail. applydiff uses similarity scoring to find the right spot even when whitespace or nearby code differs slightly.

### Why Git-Only?
Simple, universal undo. No custom backup schemes. Everyone understands `git reset --hard HEAD~1`. Plus, you get a full audit trail of what changed.

### Why Bounded Operations?
AI can generate pathological patches. Without bounds, you could get infinite loops, excessive memory use, or DoS attacks. applydiff enforces hard limits on everything.

### Why Po10 Standards?
Patch application is **mission-critical**. A buggy patcher can corrupt your entire codebase. We apply NASA/JPL Power of Ten principles: no recursion, bounded loops, extensive assertions, zero warnings.

---

## Development

### Build and Test
```bash
# Format check
cargo fmt --check

# Lint (zero warnings enforced)
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

### Project Structure
```
applydiff/
├── src/
│   ├── main.rs        # GUI app + event handlers
│   ├── parser.rs      # Patch format parser
│   ├── matcher.rs     # Fuzzy string matching
│   ├── apply.rs       # File patching logic
│   ├── git.rs         # Git safety checks
│   ├── logger.rs      # Structured JSONL logging
│   └── error.rs       # Error types + codes
├── ui/
│   └── main.slint     # UI layout (Slint framework)
├── build.rs           # Build script for Slint
└── Cargo.toml         # Dependencies
```

---

## Roadmap

- [ ] **Syntax highlighting** in patch input area
- [ ] **Diff view** showing before/after side-by-side
- [ ] **Undo button** (calls `git reset` internally)
- [ ] **Recent patches** dropdown
- [ ] **Batch mode** - queue multiple patches
- [ ] **macOS/Linux packaging** (currently manual build)
- [ ] **Windows installer** (.msi)
- [ ] **Config file** for default thresholds and limits

---

## Contributing

PRs welcome! Please maintain:
- Po10 compliance (see `SAFETY.md`)
- Zero warnings in CI
- ≥2 assertions per function
- Bounded loops with explicit guards

---

## License

MIT License - see LICENSE file

---

## FAQ

**Q: Why not just use `git apply` or `patch`?**  
A: Those require exact matches. AI-generated diffs often have context mismatches. applydiff's fuzzy matching handles this gracefully.

**Q: What if my repo has uncommitted changes?**  
A: applydiff will refuse to run. Commit or stash first. This prevents accidentally mixing patch changes with your work.

**Q: Can I use this without a GUI?**  
A: Not currently. The CLI was removed in favor of the desktop app. If you need CLI, use an earlier version or build with the `cli` feature flag (disabled by default).

**Q: What if fuzzy matching picks the wrong location?**  
A: applydiff rejects ambiguous matches (multiple equally-good hits). If it's unsure, it fails rather than guessing. Adjust the `fuzz` threshold or make your patch more specific.

**Q: Does it work on Windows?**  
A: Yes! Tested on Windows 10/11 with Git Bash or native Git.

---

**Made for developers who work with AI assistants and value their sanity.**