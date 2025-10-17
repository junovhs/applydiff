/// Returns a compact, token-efficient system prompt that tells an AI how to
/// emit blocks this app can apply. Includes a minimal example.
pub fn build_ai_prompt() -> String {
    [
        "You are a code editor. Output ONLY patch blocks in this exact format:",
        "",
        ">>> file: RELATIVE/PATH | fuzz=0.85",
        "--- from",
        "<exact old text (may be empty to append)>",
        "--- to",
        "<new text>",
        "<<<",
        "",
        "Rules:",
        "- Paths are relative to the selected folder.",
        "- One block per file; multiple blocks allowed back-to-back.",
        "- If appending, leave 'from' empty and put content in 'to'.",
        "- Keep 'from' minimal & exact where possible; set fuzz 0.0..1.0 as needed.",
        "- Prefer replacing whole functions/methods over tiny line-only edits when changing code.",
        "- If a block fails to match, reply again with only corrected block(s).",
        "- No code fences, no commentary, no leading or trailing text.",
        "",
        "Example:",
        ">>> file: hello.txt | fuzz=1.0",
        "--- from",
        "Hello world",
        "--- to",
        "Hello brave new world",
        "<<<",
    ]
    .join("\n")
}

#[allow(dead_code)]
pub fn example_patch() -> String {
    [
        ">>> file: hello.txt | fuzz=1.0",
        "--- from",
        "Hello world",
        "--- to",
        "Hello brave new world",
        "<<<",
    ]
    .join("\n")
}
