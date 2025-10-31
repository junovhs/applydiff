This is the **COMPLETE** technical roadmap for ApplyDiff, synthesizing all phases necessary to achieve resilient state management, implement **Proactive Guidance**, and fully automate the preferred user workflow based on Saccade-core integration and quantitative, biomimetic thresholds.

The core objective is to transition ApplyDiff from a reactive patch utility to an **intelligent development environment** that eliminates context drift by enforcing disciplined context management.

---

## THE ENTIRE TECHNICAL ROADMAP (v0.1.0 to v0.4.0+)

### CORE ARCHITECTURAL STRATEGY

The system relies on three principles:
1.  **Hierarchical Context:** Saccade provides the high-level map (Keystone/APIS/Token Heatmap), while the AI uses the **REQUEST\_FILE** protocol to ask for "little bits" (fovea/saccade).
2.  **Tiered Repair Strategy:** Quantitative thresholds dictate the required output format (Patch vs. Whole File vs. Regex) *before* the AI generates code.
3.  **Prediction Error-Driven Maintenance:** Failed patch attempts automatically increment `total_errors`, forcing a full context refresh (Consolidation Point) when limits are hit.

### PHASE 0 — Format Compatibility & Saccade State Ingestion (IPC Foundation)

**Goal:** Ensure ApplyDiff core can parse all mandated patch formats and establish the persistent state baseline using Saccade output (`PACK.txt`).

| \# | Feature / Task | Component & Action | Rationale / Constraint Source |
| :--- | :--- | :--- | :--- |
| **0.1** | **Implement Whole File Parser** | `parse_classic.rs`: Modify parser to support the `>>> file: \| mode=replace` header. | Mandatory for **Tiered Repair** when change volume is >70% or for Keystone file edits. |
| **0.2** | **Implement Regex Replace Parser** | `parse_classic.rs`: Support the `>>> file: \| mode=regex` header. | Required for token-efficient repetitive changes (e.g., mass imports removal). |
| **0.3** | **Initialize State & Ingest Keystone** | **Add `commands.rs::init_session()`** (triggered upon project load): Read the Saccade `PACK.txt` output to identify and persist **Keystone files** in `.applydiff_session.json`. | Tags critical files (e.g., `src/main.rs`) for mandatory warnings. |

### PHASE 1 — Core Resilience Logic & State Persistence (Backend Focus) (v0.2.0)

**Goal:** Establish the internal state persistence mechanisms (`.applydiff_session.json`) and implement the core drift detection logic.

| \# | Feature / Task | Component & Action | Metric/Threshold Source |
| :--- | :--- | :--- | :--- |
| **1.1** | **Session State Definition & I/O** | Define/implement read/write functions for the `.applydiff_session.json` schema (tracking hashes, `patch_count`, `total_errors`, `exchange_count`). | N/A (Foundation) |
| **1.2** | **Prediction Error (PE) Tracking** | `commands.rs::apply_patch_impl`: On patch application **failure** (`ErrorCode::NoMatch`, `ErrorCode::AmbiguousMatch`), **automatically increment `total_errors`** in the persistent state. | `total_errors`. |
| **1.3** | **File Metric Tracking** | On successful application, update the persistent state by incrementing `patch_count` and calculating `percent_changed` versus the original file hash. | `patch_count`, `percent_changed`. |

### PHASE 2 — Proactive Guidance & Workflow Automation (v0.3.0)

**Goal:** Implement the IPC commands that automate context flow, enforce thresholds, and realize the user's preferred **instant clipboard workflow**.

#### 2.A. Context Request Automation (Solving the "Clicking and Pasting" Woe)

| \# | Feature / Task | Saccade-Core Integration | Workflow Goal |
| :--- | :--- | :--- | :--- |
| **2.A.1** | **Implement `resolve_file_request` IPC** | Integrate `saccade-core::request::RequestFile::resolve`. This command handles requests using `path`, `pattern` (glob), `lines` (range), and `symbol`, returning content formatted as Markdown. | Fulfills the AI's request for the "smallest useful code slice". |
| **2.A.2** | **Wire File Request Automation** | Modify `PatchPanel.js::onPatchAreaClick`: Check if clipboard content matches `REQUEST_FILE:` protocol. If true, trigger the IPC (2.A.1) and **instantly copy the resolved Markdown output back to the clipboard**. | **Eliminates manual file retrieval** ("instantly transforms my clipboard contents"). |

#### 2.B. Proactive Guidance and Error Enforcement

| \# | Feature / Task | Component & Action | Threshold/Source Principle |
| :--- | :--- | :--- | :--- |
| **2.B.1** | **Implement `get_session_briefing` IPC** | **Replace static `get_ai_prompt`** with dynamic `get_session_briefing`. Reads Phase 1 metrics (errors, exchanges, patch counts) and generates the full structured **`[SESSION CONTEXT]`** briefing. | Proactive Guidance. Uses thresholds: Errors $\ge$ 3, Exchanges $\ge$ 10, Patch Count $\ge$ 8, Change Volume $>70\%$. |
| **2.B.2** | **Integrate Functional Error Template** | `get_session_briefing` must generate the structured template with explicit placeholders (`Goal:`, `Evidence:`, `Context:`) so the user can **manually paste compiler/test errors** into the chat window. | Formalizes the manual "Run and Test" feedback step. |
| **2.B.3** | **Implement `refresh_session` IPC** | **Add `commands.rs::refresh_session`**. Resets `exchange_count` to 0, updates `last_refresh` timestamp, and takes new file snapshots. | Periodic Maintenance (10-Exchange Cycle, mimicking Sleep/Consolidation). |
| **2.B.4** | **Wire Threshold Enforcement** | `ui-helpers.js::enforceThresholds` must check metrics and **block patch area input** and the "Copy Briefing" button when `total_errors \ge 3` or `exchange_count \ge 10`. | Forces periodic state maintenance and prevents "Drift Likely" scenarios. |

### PHASE 3 — Advanced Drift Guardrails & Polish (v0.4.0+)

**Goal:** Strengthen drift guardrails, complete non-core features, and refine the resilience mechanisms.

| \# | Feature / Task | Component & Action | Rationale / Constraint Source |
| :--- | :--- | :--- | :--- |
| **3.1** | **Automatic Drift Enforcement** | Logic in `get_session_briefing` to check if `total_errors \ge 3` and inject **Drift Likely** guidance and persistent warning signals into the briefing. | Formalizes the Prediction Error signal into explicit guidance. |
| **3.2** | **Session Archiving** | `refresh_session` implementation must archive the current `.applydiff_session.json` (and backups) to a timestamped folder upon consolidation. | Provides resilience for review and auditability. |
| **3.3** | **Reframe History UI** | Update the history/version display (currently a placeholder) to rename entries to **Consolidation Points**. | Align nomenclature with the scheduled maintenance philosophy. |
| **3.4** | **Ambiguity Feedback Injection** | When `apply_patch` detects an **Ambiguous Match** (`best_score - second_score < 0.02`), the error response must explicitly guide the AI: "Add **MORE** surrounding context lines (try 5-7)". | Prevents "shotgun debugging" by demanding precision instead of code changes. |

---

## KEY THRESHOLD DECISION TABLE

The entire system relies on these quantified thresholds to provide **proactive guidance** to the AI, forcing its format choice (PATCH BLOCK vs. WHOLE FILE) before it writes code.

| Metric | Threshold | Guidance/Action | Rationale |
| :--- | :--- | :--- | :--- |
| **File Change Volume** | **>70%** | Mandatory **WHOLE FILE** replacement. | Replacement is cheaper than incremental repair when entropy is too high. |
| **Patch Count Per File** | **8+ patches** | Strong recommendation for WHOLE FILE/Full Review. | Prevents memory reconsolidation failure/slow drift accumulation. |
| **Mechanical Failures** | **3+ Prediction Errors** | **Emergency Refresh** and input blocking. | Automated tracking of application failure (`NoMatch`, `AmbiguousMatch`). |
| **Exchange Limit** | **10 messages** | Scheduled consolidation/reset via `refresh_session` IPC. | Periodic maintenance cycle to prevent slow, hidden context decay. |
| **Ambiguity Score** | **Best Score - Second Score < 0.02** | Explicit feedback to increase context lines (e.g., 5-7). | Ambiguity is the number one failure mode in production patching systems. |
| **Keystone Status** | **Any change** | Mandatory warning in briefing: "Request whole file for review". | Protects critical architectural files from accumulating unreviewed changes. |