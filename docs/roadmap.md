# ApplyDiff Technical Roadmap: Resilient State Management (v0.2.0+)

This roadmap integrates the resilience architecture in phases. The **GUI is the primary workflow**; resilience logic lives in the Rust backend and is triggered via IPC.

---

## Phase 1 — Core Resilience Logic & State Persistence (v0.2.0)

**Goal:** Establish persistent session state, essential metrics, and objective Prediction Error (PE) detection tied to **patch application failure**.

| #       | Feature / Task                     | Principle                  | Code Component & Action                                                                                                          | Threshold/Metric                 |
| ------- | ---------------------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| **1.1** | **Session State Definition & I/O** | Centralized State          | Define `.applydiff_session.json` schema (`exchange_count`, `total_errors`, `patch_count`, `percent_changed`, `original_hash`).   | N/A                              |
| **1.2** | **Initial State Loading**          | Session Persistence        | `commands.rs`: on app start/folder pick, read state or create baseline.                                                          | N/A                              |
| **1.3** | **Keystone File Identification**   | Hierarchical Context       | `commands.rs`: read `saccade` `PACK.txt`; persist `[KEYSTONE]` files in session.                                                 | N/A                              |
| **1.4** | **Patch Application Enhancement**  | Prediction Error Detection | `commands.rs` (`apply_patch_impl`): on `applier.apply_block(..)` failure (e.g., `ErrorCode::NoMatch`), increment `total_errors`. | `total_errors`                   |
| **1.5** | **File Metric Tracking**           | Threshold Decisions        | `apply_patch_impl`: on success, update `patch_count` and `percent_changed` vs. original hash.                                    | `patch_count`, `percent_changed` |
| **1.6** | **Session Counter Increment**      | Periodic Maintenance       | Increment `exchange_count` whenever a new session briefing is generated/sent.                                                    | `exchange_count`                 |

**Phase 1 Acceptance:**

* `.applydiff_session.json` persists across restarts.
* Failed patch apply → `total_errors++`.
* Successful apply updates per-file `patch_count` & `percent_changed`.
* `exchange_count` increments on each generated briefing.

---

## Phase 2 — Proactive Guidance & GUI Integration (v0.3.0)

**Goal:** Surface backend signals through the existing UI via IPC: generate the **Salience Briefing**, display health, and enable scheduled refresh.

| #       | Feature / Task                     | Principle            | Code Component & Action                                                                                                                                                                            | Threshold/Metric        |
| ------- | ---------------------------------- | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| **2.1** | **“Copy Briefing” IPC**            | Selective Attention  | `commands.rs` (`get_session_briefing`): read state, check thresholds (Keystone changes, **8+ patches**, **>70% change**, **3+ errors**, **10 exchanges**), generate briefing text, return via IPC. | All P1 metrics          |
| **2.2** | **Conversation Health Monitor UI** | PE Visibility        | `app/index.html`, `app/app-state.js`: show `total_errors` & `exchange_count` with status chips (e.g., “2/3 Errors”, “7/10 Exchanges”).                                                             | 3+ errors, 10 exchanges |
| **2.3** | **Keystone Context Panel**         | Hierarchical Context | `app/index.html`, `app/app-state.js`: persistent editor/view for `CONTEXT.md` / Keystone schema.                                                                                                   | N/A                     |
| **2.4** | **Scheduled Refresh IPC + Button** | Periodic Maintenance | `commands.rs` (`refresh_session`): new snapshots, reset counters; UI **Refresh** button wired via `tauri-bridge.js`.                                                                               | N/A                     |
| **2.5** | **Wire Buttons**                   | Proactive Guidance   | `app/tauri-bridge.js`: route Copy-Briefing → `get_session_briefing`; Refresh → `refresh_session`.                                                                                                  | N/A                     |
| **2.6** | **Reframe Version Tabs**           | Consolidation Points | `app/version-tabs.js`: label versions as **Consolidation Points**.                                                                                                                                 | N/A                     |

**Phase 2 Acceptance:**

* One click **Copy Briefing** returns briefing text that reflects current thresholds.
* Health Monitor shows live counts and threshold warnings.
* **Refresh** resets counters and snapshots successfully.

---

## Phase 3 — Advanced Automation & Polish (v0.4.0+)

**Goal:** Strengthen drift guardrails, salience scoring, and session history—without changing the GUI-first flow.

| #       | Feature / Task                       | Principle              | Code Component & Action                                                                                                                                                   | Threshold/Metric |
| ------- | ------------------------------------ | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| **3.1** | **Automatic Drift Enforcement**      | PE Tracking            | After a failed apply, if `total_errors >= 3`, emit persistent **Drift Likely** warning and suggest immediate Refresh.                                                     | 3+ errors        |
| **3.2** | **Salience Scoring & Context Decay** | Selective Attention    | Track per-file recency/usage to influence briefing suggestions (recently modified files + Keystone).                                                                      | File recency     |
| **3.3** | **Session Archiving**                | Resilience & Review    | Archive `.applydiff_session.json` on Refresh to `.applydiff_archive/` with timestamp.                                                                                     | N/A              |
| **3.4** | **Comprehensive Threshold Check**    | Quantitative Precision | Ensure crossover logic for **>70% change** or **8+ patches** recommends full-file replace; Keystone change triggers whole-file warning; **10 exchanges** prompts refresh. | As listed        |
| **3.5** | **UI Error Reporting Enhancement**   | Feedback Loop          | Improve UI logs to explicitly note when a failed apply caused `total_errors++`.                                                                                           | N/A              |

**Phase 3 Acceptance:**

* **Drift Likely** appears automatically at 3+ errors.
* Briefing reflects salience/decay signals.
* Checkpoints are archived and retrievable.

---

## Technical Dependencies (Recap)

| Code File                    | Current Role      | Required Changes                                                                                                                           |
| ---------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `_backend/src/commands.rs`   | Core IPC handlers | **Add** `get_session_briefing`, `refresh_session`. **Ensure** `apply_patch_impl` updates metrics and increments `total_errors` on failure. |
| `app/app-state.js`           | UI state          | **Add** `total_errors`, `exchange_count`, Keystone & file metrics exposure.                                                                |
| `app/tauri-bridge.js`        | IPC wiring        | **Route** Copy-Briefing and Refresh to new commands; surface error increments to UI.                                                       |
| `app/index.html`             | Main UI           | **Add** Keystone panel + Conversation Health Monitor panel.                                                                                |
| `app/version-tabs.js`        | Versions UI       | **Rename** entries to “Consolidation Points”.                                                                                              |
| `applydiff-core/src/match/*` | Patch matching    | **No matching changes** required (tuning optional).                                                                                        |

