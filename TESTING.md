# ApplyDiff – Gauntlet & Lab Testing

> Evidence-first confidence, not vibes. We verify both the **what** (output) and the **how** (internal path).

## Philosophy

The patcher is safety-critical. Tests must prove:
- Correct final state (byte-for-byte where applicable)
- Correct internal behavior (fast/slow path, thresholds, safety guards)
- Clear, structured observability so failures teach us something

We enforce this with:
- **Isolation:** Every case runs in a temp sandbox.
- **Determinism:** Fixtures live under `tests/<CASE_ID>/` or are generated programmatically.
- **Instrumentation:** Structured logs (JSONL) from subsystems like `matcher`, `applier`.
- **Metadata:** `meta.json` describes expected outputs *and* expected log breadcrumbs.

---

## Current MVP Surface (what we verify right now)

**Core invariants**
- Search strategy prefers exact substring fast-path; otherwise layered search with fuzzy threshold.
- No line numbers; matching is content/context based.
- **Path safety:** Edits may not escape the selected directory.
- **CRLF/LF preservation:** Replacement lines harmonize to the existing EOL where a match is found.
- **Append-create:** Empty `--- from` on a non-existent file treats "before" as empty and previews a diff.
- **Partial apply:** Valid blocks apply, invalid blocks are skipped; backups are always created first.
- **Backups:** Timestamped sibling folder `.applydiff_backup_YYYYMMDD_*`.

**UI invariants**
- Patch panel locked until directory is chosen.
- Auto-preview on paste, and after 3s idle while typing.
- "Apply Patch" appears for all-green; "Apply Valid Changes" appears for mixed results.
- Console has **Clear**; floating buttons never overlap content.

---

## Manual Lab (quick smoke tests)

Create the lab in `~/ApplyDiffLab` (Windows MINGW64 Bash):

```bash
TEST_DIR="$HOME/ApplyDiffLab"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/src" "$TEST_DIR/docs" "$TEST_DIR/notes" "$TEST_DIR/new/nested" "$TEST_DIR/deep/dir"

# LF
cat > "$TEST_DIR/readme.md" <<'EOF'
# ApplyDiff Lab
Welcome to the lab.
EOF

# CRLF
printf 'function greet(){\r\n  console.log("Hello world");\r\n}\r\n' > "$TEST_DIR/src/app.js"

# More LF
cat > "$TEST_DIR/src/math.js" <<'EOF'
export function add(a, b) {
  return a + b;
}
EOF

cat > "$TEST_DIR/docs/guide.md" <<'EOF'
## Guide
Steps go here.
EOF

# Two nearly identical blocks to create ambiguity potential
cat > "$TEST_DIR/notes/duplicate.txt" <<'EOF'
id: A
start
  marker: section
  value: target
end

id: B
start
  marker: section
  value: target
end
EOF

echo "Lab created at $TEST_DIR"
```

Open ApplyDiff → **Select Directory** → choose `~/ApplyDiffLab`.
Click **Patch** and paste the blocks below.

### A) Exact match (sanity)

```
>>> file: readme.md | fuzz=1.0
--- from
# ApplyDiff Lab
--- to
# ApplyDiff Lab (patched)
<<<
```

**Expect:** Green diff; apply succeeds.
**Verify:** `readme.md` now starts with `# ApplyDiff Lab (patched)`.

### B) CRLF harmonization

```
>>> file: src/app.js | fuzz=0.85
--- from
  console.log("Hello world");
--- to
  console.log("Hello brave new world");
<<<
```

**Expect:** Diff shows one changed line; **file keeps CRLF** (`0d 0a`).
**Optional check:** `xxd -g 1 -c 1 ~/ApplyDiffLab/src/app.js | head` shows `0d` `0a` line endings.

### C) Append-create (new file) — **verified**

```
>>> file: new/nested/log.txt | fuzz=1.0
--- from
--- to
Log started
Entry: 1
<<<
```

**Expect:** Diff preview shows an added file; apply creates `new/nested/log.txt` with two lines.

### D) Path traversal guard

```
>>> file: ../escape.txt | fuzz=1.0
--- from
--- to
You should never see this.
<<<
```

**Expect:** Preview log contains ❌ "Patch path escapes target directory".
**Apply** button remains **orange** ("Apply Valid Changes") only if other blocks in the same submission are valid; otherwise hidden.

### E) Ambiguity trap (fuzzy)

Tabs vs spaces make the context fuzzy; there are two similar windows.

```
>>> file: notes/duplicate.txt | fuzz=0.90
--- from
start
    marker: section
    value: target
end
--- to
start
    marker: section
    value: PATCHED
end
<<<
```

**Expect:** No match ≥ 0.90; preview shows ❌.
**Fix:** Disambiguate by including `id: A` lines in `from`.

### F) Mixed patch (partial apply) — **verified**

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

>>> file: ../should-not-write.txt | fuzz=1.0
--- from
--- to
nope
<<<
```

**Expect:** Diff shows the first two hunks; third is rejected in preview.
Button reads **Apply Valid Changes** (orange).
**After apply:** console reports `2 applied, 1 failed`, `new/report.txt` contains `Report v1`.

---

## Completed Gauntlet Case

### LF01 – Large File Patch at Start

**Objective:** Verify exact-match edit at start of a 50k-line file and that the engine uses the fast path.
**Verification:**

* Byte-for-byte equality with expected `after/large_file.txt`.
* Reported `ok=1, fail=0`.
* Log contains `"action":"fast_path_match"` from the matcher.
  **Confidence:** 10/10 — both output and internal path are proven.

---

## White-Box Expectations (log probes)

We assert these log breadcrumbs during tests:

| Subsystem | Action                  | Meaning                               |
| --------- | ----------------------- | ------------------------------------- |
| matcher   | `fast_path_match`       | Exact substring path used             |
| matcher   | `search_start`          | Enter layered search (no exact match) |
| matcher   | `no_match_threshold`    | Best score `< fuzz` threshold         |
| applier   | `path_escape` (message) | Attempted path leaves root (rejected) |

Tests check for these strings verbatim in captured logs.

---

## Roadmap (next test authoring)

**Large File (`LF`)**

* `LF02-Replace-Middle`, `LF03-Replace-End`, `LF04-Multi-Line-Replace`, `LF05-Fuzzy-No-Match (bounded)`

**Matcher & Applier (`MA`)**

* `MA01-True-Ambiguity` (tie detection)
* `MA02-Sequential-Dependency`
* `MA03-CRLF-to-LF-Harmonization` (already observed manually; add formal gauntlet)
* `MA04-Delete-No-EOL`
* `MA05-Append-No-EOL`

**Filesystem (`FS`)**

* `FS01-Create-Subdirectory` (covered by Lab C; formalize)
* `FS02-Empty-File-Patch`
* `FS03-Binary-File` (non-UTF8 read error, no write)
* `FS04-Read-Only-File`

> As we expand flexible matching (whitespace/indent normalization, hunk decomposition), add mirrored tests that prove failure modes are **safe first** (no writes) and successes are **traceable** via logs.