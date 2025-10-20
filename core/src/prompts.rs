pub fn build_ai_prompt() -> String {
    // Short, explicit instructions for LLMs to emit AFB-1 armored blocks.
    // We keep this as a single string to avoid formatting surprises.
    let prompt = r#"You are producing APPLYDIFF patches for a human user. Output ONLY the armored format below.
Do NOT include explanations, markdown code fences, or extra commentary.

Format (repeat per changed file):

-----BEGIN APPLYDIFF AFB-1-----
Path: <relative/path/from/project/root>
Fuzz: 0.85
Encoding: base64
From:
<base64 of EXACT old text; may be empty to create/append>
To:
<base64 of new text>
-----END APPLYDIFF AFB-1-----

Rules:
- Base64 may be wrapped arbitrarily; whitespace will be ignored.
- If you cannot find the exact old text, lower Fuzz (e.g., 0.80) but keep intent.
- Emit multiple blocks back-to-back for multiple files.
"#;
    prompt.to_string()
}
