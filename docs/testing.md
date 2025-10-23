# ApplyDiff – Gauntlet & Lab Testing

> Evidence-first confidence, not vibes. We verify the **what** (output), the **how** (internal path), and the **workflow** (safe handoffs).

## Philosophy

The `ApplyDiff Suite` is a safety-critical component in an AI-assisted workflow. Tests must prove:
- **Correct Final State:** Byte-for-byte correctness of the final file outputs.
- **Correct Internal Behavior:** The right code paths are taken (fast vs. slow), thresholds are respected, and safety guards are triggered.
- **Workflow Integrity:** The entire `git -> AI -> apply` loop is safe, enforcing best practices like clean working directories before starting.
- **Clear Observability:** Structured logs ensure every failure is a lesson.

We enforce this with:
- **Isolation:** Every case runs in a temporary sandbox created from a clean Git state.
- **Determinism:** Fixtures are defined in `tests/<CASE_ID>/` or generated programmatically.
- **Instrumentation:** Structured JSONL logs from all subsystems (`matcher`, `applier`, `git`, `workflow`).
- **Metadata:** `meta.json` specifies expected outcomes, including success/fail counts and log breadcrumbs.
- **Multi-layer Verification:** Hash comparisons, byte-level analysis, and Git state checks.

---

## Current Test Coverage

**Automated gauntlet: 8/18 passing ✅**

### Core Engine (8/8 Passing)
These tests validate the core patch-application logic.

-   **01-afb-append:** AFB-1 Append/Create ✅
-   **02-afb-replace:** AFB-1 Exact Replace ✅
-   **03-path-traversal:** Path Escape Rejection ✅
-   **04-append-create:** Empty FROM Creates File ✅
-   **05-large-file:** 50K Line Fast Path ✅
-   **06-ambiguity-simple:** Duplicate YAML Blocks ✅
-   **07-ambiguity-indent:** Similar Python Functions ✅
-   **08-crlf-preserve:** Byte-Level Line Ending Preservation (Undeniable Proof) ✅

*Details for these tests remain the same as the previous version.*

---

## Test Roadmap (18 Total Tests)

### Completed: 8/18 ✅
- ✅ All Core Engine tests (1-8)

### Planned: 10/18
#### Core Engine Tests (Planned)
- ❌ 09-mixed-results – Partial apply (2 succeed, 1 fails)
- ❌ 10-backup-restore – Backup exists, restore recovers exact state
- ❌ 11-symlink-escape – Symlinks outside sandbox rejected
- ❌ 12-large-file-middle – 50K line patch at line 25,000
- ❌ 13-large-file-end – 50K line patch at line 49,999
- ❌ 14-multiline-replace – Replace 100 consecutive lines
- ❌ 15-fuzzy-timeout – Worst-case fuzzy search bounded

#### Workflow & Integration Tests (Planned)
- ❌ **16-git-dirty-reject** – Rejects directory with uncommitted changes
    - **Verifies:** `git status` check, UI flow blocked, warning displayed.
- ❌ **17-git-clean-proceed** – Accepts clean directory and displays branch
    - **Verifies:** `git status` check, UI proceeds, branch name correctly parsed.
- ❌ **18-commit-assist-generation** – Generates conventional commit message
    - **Verifies:** Correct type/scope/subject parsing from prompt and file paths.

---

## Manual Lab (Quick Smoke Tests)

Create the test environment as described in the previous version. Start with a clean Git repository: `git init && git add . && git commit -m "Initial commit"`.

Open ApplyDiff → **Select Directory** → `~/ApplyDiffLab`.

### Scenarios A-F
*(These remain the same: Exact Match, CRLF Preservation, Append-Create, Path Traversal, Ambiguity, Partial Apply)*

### Scenario G: Git Handshake (Dirty State)

1.  Modify a file: `echo "dirty" >> ~/ApplyDiffLab/readme.md`
2.  In ApplyDiff, select the `~/ApplyDiffLab` directory.

**Expected:**
- A warning bar appears: "⚠️ Uncommitted Changes Detected."
- The "Smart Context Orb" for generating context **does not appear**.
- The workflow is blocked.
- After committing the change and re-selecting the directory, the warning disappears and the Orb appears.

### Scenario H: Full Workflow (Happy Path)

1.  Start with a clean Git repo. Select the `~/ApplyDiffLab` directory.
2.  In ApplyDiff's "My Prompt" box, paste: `Add a 'return a + b + c;' line to the add function in src/math.js`
3.  Drag the **Smart Context Orb (Blueprint)** to trigger the copy action.
4.  Paste the blueprint and the prompt into your Chat AI.
5.  Copy the AI's patch response.
6.  Paste the patch into ApplyDiff. A `✓ Armored Format` check appears.
7.  The Diff Preview looks correct. Click "Apply Patch".
8.  The Console confirms `1 applied, 0 failed`.
9.  Click the **"Prepare Commit"** button that appears.

**Expected:**
- The Commit Assistant panel opens with a generated message:
  ```
  feat(core): add third parameter to add function

  Add a 'return a + b + c;' line to the add function in src/math.js

  Files changed:
  - src/math.js
  ```
- Clicking "Copy Message" works as expected.

---

## White-Box Log Probes

Tests verify these structured log entries:

| Subsystem | Action | Meaning |
|-----------|--------|---------|
| `matcher` | `fast_path_match` | Exact substring used (optimal). |
| `matcher` | `ambiguous_match` | Two+ targets with similar scores. |
| `applier` | `path_escape_rejected`| Path validation rejected a block. |
| `git`     | `state_check_dirty` | Blocked workflow due to uncommitted changes. |
| `git`     | `state_check_clean` | Proceeded with workflow, branch detected. |
| `workflow`| `commit_assist_generated`| Commit message was generated. |

---

## Known Test Gaps

1.  **Atomicity & Backups**: Partial apply and restore round-trip (Tests #9, #10).
2.  **Filesystem Edge Cases**: Symlinks, permissions, etc. (Test #11).
3.  **Performance Bounds**: Worst-case fuzzy search and large file edits (Tests #12, #13, #15).
4.  **UI Interaction**: The kinetic feel of the "Smart Context Orb" and the guided commit button flow are currently manual tests.
5.  **Version History Persistence**: Ensuring version tabs survive an application restart.

---

## Test Quality Metrics

- **Coverage**: 8/18 tests, ~15 subsystem behaviors verified
- **Confidence**: **High** for core patch engine (matching, ambiguity, CRLF, path safety). **Planned** for Git integration and workflow handoffs.
- **Gaps**: See "Known Test Gaps" above. The largest gap is the lack of automated UI interaction tests.
- **False positives/negatives**: None observed; multi-layer verification minimizes risk.

**Current status: 44% complete (8/18)**  
**Target for v1.0:** 18/18 gauntlet tests passing, all manual lab scenarios verified.