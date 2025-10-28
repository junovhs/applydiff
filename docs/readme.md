Here’s a **GUI-first, conversation-aligned rewrite** of your README. I kept the content to what you provided—just tightened structure, simplified the user story, and clarified the “prediction error” framing.

---

# ApplyDiff · Resilient State Management Co-Pilot

ApplyDiff is evolving from a patch utility into an **intelligent development environment** for long-running AI coding sessions. It’s designed to eliminate **context drift**—the slow buildup of stale assumptions and subtle errors that derail LLM workflows—by enforcing disciplined, quantitative context management for predictable stability and token efficiency.

---

## Table of Contents

* [Why ApplyDiff](#why-applydiff)
* [Quick Start: How You’ll Use It](#quick-start-how-youll-use-it)
* [What You Get (At a Glance)](#what-you-get-at-a-glance)
* [Core Architectural Strategy](#core-architectural-strategy)
* [Integrated Workflow & GUI](#integrated-workflow--gui)

  * [Frontend Additions](#frontend-additions)
  * [Backend State Management (Rust)](#backend-state-management-rust)
  * [Development Cycle](#development-cycle)
* [Technical Thresholds](#technical-thresholds)
* [FAQ: Prediction Errors & Progressive Disclosure](#faq-prediction-errors--progressive-disclosure)
* [Inspiration: Resource-Rationality](#inspiration-resource-rationality)

---

## Why ApplyDiff

Long AI conversations **drift**: constraints get forgotten, code suggestions miss the mark, patches stop matching reality. ApplyDiff manages session memory and status **for you**, shifting the workflow to **proactive guidance**—the app tells the AI exactly what context to use **before** it responds. Our streamlined GUI is the surface for applying code; the intelligence runs behind the scenes like a health monitor.

---

## Quick Start: How You’ll Use It

> An approachable, low-info, user-first workflow.

1. **Copy Briefing → Ask**
   Click **Copy Briefing** in the UI. Paste that into your AI chat, then add your question. The briefing spotlights only what matters (recent/critical files), so the AI starts from a fresh, accurate view of your project.

2. **Paste Patch → Auto-Apply**
   Paste the AI’s output into the patch box. ApplyDiff auto-applies it. If a patch can’t be applied, the system records that failure as a clear, automatic signal of drift—no manual tracking.

3. **Refresh When Prompted**
   After ~**10 exchanges**, you’ll see **REFRESH RECOMMENDED**. Click **Refresh** to snapshot the current state and reset counters—preventing slow, hidden context decay.

**Why this feels better immediately**

* **Stay in flow:** No hand-assembling context; the briefing does it for you.
* **Fewer do-overs:** The AI is guided *before* it writes, reducing irrelevant or outdated changes.
* **Built-in guardrails:** Early warnings and scheduled refreshes keep long sessions stable.

---

## What You Get (At a Glance)

* **No more context loss:** Pause/resume safely; the system carries the memory.
* **Smarter AI output:** Pre-warnings (e.g., “8+ patches on this file”, “Keystone Context”) guide the model’s format (patch vs. replace).
* **Early warning system:** **3+ failed patch applies** flags **Drift Likely**, prompting a clean reset instead of hours of confusion.

---

## Core Architectural Strategy

Quantitative strategies that keep state accurate under resource limits and provide **proactive guidance** *before* code generation:

| Principle                                   | Technical Mandate                                          | Implementation                                                                                                                   |
| ------------------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Hierarchical Context (Schema Isolation)** | Separate foundational architecture from transient details. | **Keystone Context:** Critical files (e.g., `src/main.rs`, core types) are tagged and **always included** in high-level prompts. |
| **Prediction Error Detection**              | Track mismatches between expectation and reality.          | **Automated Drift Tracking:** Increment `total_errors` when a patch fails to apply.                                              |
| **Selective Context Injection**             | Spend tokens only on high-value files.                     | **Dynamic Salience Briefing:** Concise, prioritized session briefing using modified status + Keystone tags.                      |
| **Quantitative Entropy Thresholds**         | Use numeric rules to choose output format.                 | **Patch vs. Replace Guidance:** If **>70% change** or **8+ patches**, recommend full-file replacement.                           |
| **Periodic State Reset**                    | Enforce checkpoints to avoid silent error buildup.         | **10-Exchange Cycle:** Force consolidation and baseline refresh.                                                                 |

---

## Integrated Workflow & GUI

The GUI is the **primary** workflow. Resilience features are woven directly into the interface; CLI-centric flows are deprecated in favor of GUI-triggered IPC commands.

### Frontend Additions

* **Keystone Context Panel:** Persistent view/edit of `CONTEXT.md` and critical architectural constraints.
* **Conversation Health Monitor:** Live `exchange_count`, `total_errors` (e.g., “2/3 Errors • Refresh Recommended”).
* **Copy Briefing Button:** One click generates and copies the **Salience Map** for pasting into chat.
* **Threshold Warnings:** Visual cues when a file exceeds patch count or change-volume thresholds.

### Backend State Management (Rust)

All state is centralized in `_backend/src/commands.rs` and persisted to a single file.

* **State File:** `.applydiff_session.json` tracks file hashes, patch counts, lines changed, change percentages, and error metrics.
* **Automatic Error Logging:** In `apply_patch_impl`, if `applier.apply_block` fails (e.g., `ErrorCode::NoMatch`), `total_errors` increments automatically.
* **Context Generation:** An IPC command (wired to **Copy Briefing**) computes the context block, checks thresholds (Keystone changes, 10-exchange limit, 3-error limit), and formats proactive guidance.

### Development Cycle

1. **Initial Context:** Click **Copy Briefing** → paste briefing + Keystone Context + your query to the AI.
2. **Guided Response:** The AI adjusts format (patch vs. full file) per guidance (e.g., high patch count).
3. **Apply & Drift Check:** Paste AI output; ApplyDiff auto-applies and updates metrics. Failed apply → **Error Count++**.
4. **Maintenance:** At 10 exchanges, UI prompts **Refresh/Checkpoint** to consolidate state.

---

## Technical Thresholds

| Metric                  | Threshold                | Action                                                 | Source Principle                 |
| ----------------------- | ------------------------ | ------------------------------------------------------ | -------------------------------- |
| **File Change Volume**  | **>70%**                 | Mandatory full-file request                            | Tiered Repair/Replace Decisions  |
| **Patch Count**         | **8+**                   | Strong recommendation for full-file review/replacement | Memory Reconsolidation / Entropy |
| **Conversation Health** | **3+ Prediction Errors** | Emergency refresh                                      | Prediction Error-Driven Refresh  |
| **Exchange Limit**      | **10 messages**          | Scheduled consolidation/reset                          | Periodic Maintenance Cycles      |
| **Keystone Status**     | **Any change**           | Mandatory warning to review whole file                 | Hierarchical Compression         |

---

## FAQ: Prediction Errors & Progressive Disclosure

**Q: Is a failed patch apply really an *objective* “Prediction Error” signal?**
**A:** Yes—within ApplyDiff’s scope. The AI “predicts” that the `From:` block exists in the current file. When `apply_patch_impl` can’t find it (e.g., `ErrorCode::NoMatch`), that’s a concrete mismatch between expectation and on-disk reality. We count that automatically as `total_errors`.

**Q: What about compiler errors or syntax issues that get fixed over a few iterations?**
**A:** Those can still reflect convergence via *progressive disclosure* (you only see the next errors after fixing the current ones). ApplyDiff focuses first on the most direct, mechanical signal—**patch application failure**—as a high-precision drift indicator. (Compiler/test failures can exist in addition, but the automatic PE signal comes from apply-time failures.)

---

## Inspiration: Resource-Rationality

ApplyDiff borrows from **bounded biological systems**: limited working memory (~3–4 items), **prediction errors** to trigger updates, and **saliency maps** to allocate attention. Principles like **Rate-Distortion Theory** and a flexible **~40% constraints / 60% flexibility** balance inform the design—yielding a robust, predictable system that manages entropy in complex AI conversations.

---

*Notes:*

* This README reflects the GUI-first workflow and objective drift tracking via failed patch application.
