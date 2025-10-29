/// Builds the static, reliable prompt for the AI.
pub fn build_ai_prompt() -> String {
    let mut prompt = String::new();

    prompt.push_str("You are producing APPLYDIFF patches. Output ONLY the classic format below.\n");
    prompt.push_str("Do NOT include explanations, markdown code fences, or extra commentary.\n\n");
    prompt.push_str("Format (repeat per changed file):\n\n");
    prompt.push_str(">>> file: <relative/path/from/project/root> | fuzz=0.85\n");
    prompt.push_str("--- from\n");
    prompt.push_str("<exact old text; may be empty to create/append>\n");
    prompt.push_str("--- to\n");
    prompt.push_str("<new text>\n");
    prompt.push_str("<<<\n\n");
    prompt.push_str("Rules:\n");
    prompt.push_str("- Include 3+ lines of surrounding context in 'from' and 'to' to ensure a unique match.\n");
    prompt.push_str("- Preserve exact indentation and whitespace.\n");

    prompt
}