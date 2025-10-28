
## ApplyDiff Biomimetic Patch Format Specification v3.0

*(Incorporating Layered Application Resilience and Dual-Model Architecture)*

═══════════════════════════════════════════════════════════════════

### CORE ARCHITECTURAL RATIONALE (Dual-Model Pattern)

To achieve maximum accuracy (85% SOTA) and resilience, the workflow operates on an **Architect/Editor Separation Pattern**:

1.  **The Architect/Planner Model (Reasoning):** Interprets the complex user request, generates the overall plan (Guidance), and selects the files to be edited.
2.  **The Editor/Coder Model (Precision):** Receives the planning data and the `[SESSION CONTEXT]`. This model's *only task* is the mechanical generation of the precise patch payload in the defined format.

This specification governs the output of the **Editor/Coder Model** and the requirements for the **Application Engine**.

═══════════════════════════════════════════════════════════════════

### 1. SESSION CONTEXT BLOCK (Mandatory Prefix for Editor Model)

This block serves as the **Proactive Guidance** signal and **Prediction Error (PE) feedback** mechanism for the Editor Model.

```
[SESSION CONTEXT]
Modified: ( patches, % changed), ( patches, % changed)
Guidance: [Natural language plan/goals from Architect/Planner]
Health: <total errors>, <ambiguous matches>, exchange <current exchange>/<max exchanges>
[/SESSION CONTEXT]
```

**Rules:**

*   This context is generated and updated by the `ApplyDiff` Application Engine after each step.
*   **Ambiguity Tracking:** The `ambiguous matches` count tracks instances where the fuzzy match confidence score is too close between two potential locations (`best_score - second_score < 0.02`).
*   **Actionable Signal:** High ambiguity (e.g., 3+ ambiguous matches) explicitly signals the Editor/Coder that it needs to include **MORE context lines** (e.g., 5-7 lines instead of 3), rather than changing the patch content itself.

═══════════════════════════════════════════════════════════════════

### 2. PATCH BLOCK (Default Format - Classic Style)

This format uses a **modified unified diff style** proven to provide **3X accuracy improvement** over search/replace blocks for application tasks.

```
>>> file: <path/to/file.ext> [| mode=patch] [| fuzz=0.85]
--- from
<context lines, plus lines to remove (if any)>
--- to
<context lines, plus lines to add (if any)>
<
```

**Rules:**

*   **Format Mandate:** **Plain text only** (NO Base64, NO JSON wrapping, NO special encoding). This is the **ONLY format AI should generate for patches**.
*   **Context:** Must include **3+ surrounding context lines** for fuzzy matching. This enables the fuzzy logic required for **9X error reduction**.
*   **Line Numbers:** Explicitly **remove line numbers** from headers or hunks; the application relies purely on context matching.
*   **Whitespace:** Preserve exact indentation and whitespace.
*   **Multi-file:** Multiple blocks are allowed (one immediately following the other). This aligns with unified diff's excellent multi-file capability.

═══════════════════════════════════════════════════════════════════

### 3. WHOLE FILE BLOCK (Replacement Strategy)

This block implements the **Apoptosis/Refresh** strategy, used when the cost of incremental repair exceeds the replacement threshold.

```
>>> file: <path/to/file.ext> | mode=replace
--- from
--- to
<entire new file contents>
<
```

**Rules:**

*   **Trigger Conditions:** Used when the file change **>70%**, **8+ patches applied**, or the file is explicitly designated a **KEYSTONE** file.
*   **Content:** The `--- to` section must contain the *entire updated content* of the file.

#### THRESHOLD DECISION TABLE (Quantitative Precision)

| File State | Format Choice | Rationale |
| :--- | :--- | :--- |
| <30% changed, <6 patches | **PATCH BLOCK** | Incremental repair optimal |
| 30-50% changed, 6-8 patches | **PATCH BLOCK (caution)** | Approaching threshold, monitor |
| 50-70% changed, 8+ patches | **AI decides (warning)** | Transition zone, context-dependent |
| **>70% changed OR KEYSTONE change** | **WHOLE FILE mandatory** | Full refresh more reliable |
| Multiple ambiguous matches (3+) | **PATCH with MORE context (5+ lines)** | Increase context size to resolve positional ambiguity |

═══════════════════════════════════════════════════════════════════

### 4. APPLICATION ENGINE REQUIREMENTS (Internal Mandate)

The Application Engine (ApplyDiff Core) must implement a highly robust, layered matching logic based on best-in-class open-source systems:

1.  **Progressive Fallback:** Apply matches sequentially, stopping at the first successful match above the confidence threshold:
    *   **Tier 1:** Exact Substring Match (Fast Path).
    *   **Tier 2:** Whitespace-Normalized Equality (Ignoring cosmetic diffs).
    *   **Tier 3:** Relative-Indentation-Preserving Equality (Crucial for syntactic correctness in languages like Python).
    *   **Tier 4:** Damerau-Levenshtein Fuzzy Search with Confidence Scoring (Minimizes editing errors).
2.  **Ambiguity Guard:** Before accepting a fuzzy match, the engine must compare the best score (`best_score`) against the second-best score (`second_score`). If the difference is too small (`< 0.02`), the result is rejected as an **Ambiguous Match**.

═══════════════════════════════════════════════════════════════════

### 5. ERROR FEEDBACK (Specific PE Signaling)

If the Application Engine fails to apply the patch, it must return one of these specific, actionable messages to the LLM (Editor) to enable self-correction.

| Status | Error Code/Message | Required AI Action |
| :--- | :--- | :--- |
| **❌ Ambiguous match detected** | Your "from" block matched multiple locations in the file with near-equal confidence. The application cannot proceed safely. | **Action:** Submit the same patch content but use **MORE surrounding context lines (5+)** to uniquely define the target location. |
| **❌ No match found** | Your "from" block did not match any location in the file. Possible causes: File changed, whitespace differs, or code moved. | **Action:** Request current state of the relevant function/section. |
| **✅ Patch Applied** | Apply succeeded. Health updated in [SESSION CONTEXT]. | **Action:** Continue to next task step or end. |
| **❌ Patch Format Invalid** | The output did not conform to the required Classic Style (`>>> file:`, `--- from`, `--- to`, `<`). | **Action:** Regenerate output strictly adhering to the mandated format. |