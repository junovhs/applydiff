# ApplyDiff – Gauntlet & Lab Testing

> Evidence-first confidence, not vibes. We verify both the **what** (output) and the **how** (internal path).

## Philosophy

The patcher is safety-critical. Tests must prove:
- Correct final state (byte-for-byte where applicable)
- Correct internal behavior (fast/slow path, thresholds, safety guards)
- Clear, structured observability so failures teach us something

We enforce this with:
- **Isolation:** Every case runs in a temp sandbox
- **Determinism:** Fixtures under `tests/<CASE_ID>/` or generated programmatically
- **Instrumentation:** Structured JSONL logs from subsystems (`matcher`, `applier`)
- **Metadata:** `meta.json` specifies expected counts + log breadcrumbs

---

## Current Test Coverage

**Automated gauntlet: 3/3 passing ✅**

### LF01 – Large File Patch at Start ✅
**What:** Exact-match edit at line 1 of 50K-line file  
**Verifies:**
- Byte-for-byte output correctness
- Fast path used (`"action":"fast_path_match"` in logs)
- `ok=1, fail=0` counts match expectation

### MA01a – Simple Ambiguity ✅
**What:** Patch matches two identical 3-line YAML blocks  
**Verifies:**
- Ambiguity detection triggers
- Patch rejected (`ok=0, fail=1`)
- Log contains `"ambiguous_match"` breadcrumb

### MA01b – Indentation Ambiguity ✅
**What:** Patch matches two Python functions with identical content  
**Verifies:**
- Ambiguity detection across different formatting
- Rejection even with whitespace variance
- Log contains `"ambiguous_match"`

---

## Manual Lab (Quick Smoke Tests)

Create test environment:

```bash
TEST_DIR="$HOME/ApplyDiffLab"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/src" "$TEST_DIR/docs" "$TEST_DIR/notes" "$TEST_DIR/new/nested"

# LF file
cat > "$TEST_DIR/readme.md" <<'EOF'
# ApplyDiff Lab
Welcome to the lab.
EOF

# CRLF file (Windows line endings)
printf 'function greet(){\r\n  console.log("Hello world");\r\n}\r\n' > "$TEST_DIR/src/app.js"

# Additional LF files
cat > "$TEST_DIR/src/math.js" <<'EOF'
export function add(a, b) {
  return a + b;
}
EOF

cat > "$TEST_DIR/docs/guide.md" <<'EOF'
## Guide
Steps go here.
EOF

# Ambiguity test file
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

Open ApplyDiff → **Select Directory** → `~/ApplyDiffLab` → Paste test patches.

### Scenario A: Exact Match (Sanity Check)

```
PATCH readme.md fuzz=1.0
FROM
# ApplyDiff Lab
TO
# ApplyDiff Lab (patched)
END
```

**Expected:**
- Green diff in preview
- "Apply Patch" button appears
- After apply: v1 tab appears, file updated

**Verify:** `cat ~/ApplyDiffLab/readme.md` shows `# ApplyDiff Lab (patched)`

### Scenario B: CRLF Preservation

```
PATCH src/app.js fuzz=0.85
FROM
  console.log("Hello world");
TO
  console.log("Hello brave new world");
END
```

**Expected:**
- Diff shows single line change
- File keeps CRLF endings (`\r\n`)

**Verify:** `xxd -g 1 ~/ApplyDiffLab/src/app.js | grep -A1 "Hello"` shows `0d 0a` bytes

### Scenario C: Append-Create (New File)

```
PATCH new/nested/log.txt fuzz=1.0
FROM
TO
Log started
Entry: 1
END
```

**Expected:**
- Diff shows "new file" with added lines
- Parent directories created automatically

**Verify:** `cat ~/ApplyDiffLab/new/nested/log.txt` shows both lines

### Scenario D: Path Traversal Rejection

```
PATCH ../escape.txt fuzz=1.0
FROM
TO
You should never see this.
END
```

**Expected:**
- Console shows ❌ "Patch path escapes target directory"
- No "Apply Patch" button (or orange "Apply Valid Changes" if mixed with valid blocks)

### Scenario E: Ambiguity Trap

```
PATCH notes/duplicate.txt fuzz=0.90
FROM
start
    marker: section
    value: target
end
TO
start
    marker: section
    value: PATCHED
end
END
```

**Expected:**
- No match ≥ 0.90 (two equally-good targets)
- Console shows ❌ with ambiguity mention
- Terminal logs show `"ambiguous_match"`

**Fix:** Add `id: A` to FROM block to disambiguate.

### Scenario F: Partial Apply (Mixed Results)

```
PATCH readme.md fuzz=1.0
FROM
Welcome to the lab.
TO
Welcome to the patched lab.
END

PATCH new/report.txt fuzz=1.0
FROM
TO
Report v1
END

PATCH ../should-not-write.txt fuzz=1.0
FROM
TO
nope
END
```

**Expected:**
- Diff shows first two changes only
- Button reads "Apply Valid Changes" (orange)
- Console shows `2 applied, 1 failed`

**Verify:** `report.txt` created, `escape.txt` not created

---

## White-Box Log Probes

Tests verify these structured log entries:

| Subsystem | Action | Meaning |
|-----------|--------|---------|
| `matcher` | `fast_path_match` | Exact substring used (optimal) |
| `matcher` | `search_start` | Entering layered fuzzy search |
| `matcher` | `ambiguous_match` | Two+ targets with similar scores |
| `matcher` | `no_match_threshold` | Best score below `fuzz` setting |
| `applier` | (path escape message) | Path validation rejected block |

Example log entry:
```json
{"ts":"2025-10-20T07:30:03Z","level":"info","rid":1760945133803,"subsystem":"matcher","action":"fast_path_match","msg":"unique exact substring (len=38)"}
```

---

## Test Expansion Roadmap

### Large File Series (`LF`)
- [ ] **LF02-Replace-Middle**: Edit at line 25K of 50K file
- [ ] **LF03-Replace-End**: Edit at line 49,999 of 50K file
- [ ] **LF04-Multi-Line-Replace**: Replace 100-line function
- [ ] **LF05-Bounded-Fuzzy**: Ensure fuzzy search terminates in reasonable time

### Matcher & Applier Series (`MA`)
- [x] **MA01a-Simple-Ambiguity**: YAML with duplicate blocks ✅
- [x] **MA01b-Indentation-Ambiguity**: Python with similar functions ✅
- [ ] **MA02-Sequential-Dependency**: Block 2 depends on Block 1 applying first
- [ ] **MA03-CRLF-LF-Mixing**: Patch with `\n`, file has `\r\n`
- [ ] **MA04-No-Final-Newline**: File/patch without trailing newline
- [ ] **MA05-Whitespace-Normalization**: Tabs vs spaces, extra whitespace

### Filesystem Series (`FS`)
- [ ] **FS01-Create-Nested-Dirs**: Deeply nested path creation
- [ ] **FS02-Empty-File**: Patch empty file → add content
- [ ] **FS03-Binary-File**: Reject non-UTF8 files gracefully
- [ ] **FS04-Read-Only-File**: Handle permission errors

### Version History Series (`VH`) - New!
- [ ] **VH01-Version-Navigation**: Apply 5 patches, navigate v1↔v5
- [ ] **VH02-Version-Notes**: Add/edit notes, verify display
- [ ] **VH03-Large-History**: 20+ versions, test performance
- [ ] **VH04-Multi-File-Versions**: Each version touches different files

---

## Running Tests

**Automated gauntlet:**
```bash
cargo tauri dev
# Click "Run Self-Test" in console
```

**Manual lab:**
1. Create `~/ApplyDiffLab` (see above)
2. Run scenarios A-F
3. Verify outputs match expectations
4. Check console logs for breadcrumbs

**Adding new tests:**
1. Create `tests/CASE_ID/{before/,after/,meta.json,patch.txt}`
2. `meta.json` must specify `expect_ok`, `expect_fail`, optional `expected_log_contains`
3. Run gauntlet, verify pass/fail
4. Commit test case with descriptive name

---

## Known Test Gaps

1. **Multi-block patches**: Current tests are single-block; need multi-file stress tests
2. **Performance bounds**: No tests for worst-case fuzzy search timing
3. **Concurrent applies**: No tests for rapid successive patches
4. **Version history persistence**: Versions are in-memory only (not tested after restart)
5. **Patch syntax edge cases**: Malformed blocks, missing markers, encoding issues

---

## Test Quality Metrics

- **Coverage**: 3 tests, 8 subsystem behaviors verified
- **Confidence**: High for exact matching, ambiguity detection, path safety
- **Gaps**: Whitespace normalization, multi-block, performance bounds
- **False positives**: None observed (all passes are legitimate)
- **False negatives**: Unknown (may exist in untested code paths)

**Target for v1.0:** 15+ gauntlet tests covering all `LF`, `MA`, `FS` series.