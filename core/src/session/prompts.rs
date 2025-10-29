use super::SessionState;
use std::fmt::Write;

/// Builds the full proactive guidance prompt for the AI based on current session state.
pub fn build_ai_prompt(state: &SessionState) -> String {
    let mut prompt = String::new();

    // --- Header ---
    writeln!(prompt, "[SESSION CONTEXT]").unwrap();

    // --- Health Metrics ---
    let health_status = format!(
        "Health: {}/3 errors, exchange {}/10",
        state.total_errors, state.exchange_count
    );
    writeln!(prompt, "{}", health_status).unwrap();

    // --- File Change Summary ---
    let mut modified_files = state
        .files
        .iter()
        .filter(|(_, metrics)| metrics.patch_count > 0)
        .collect::<Vec<_>>();
    modified_files.sort_by_key(|(path, _)| path.as_path());

    if !modified_files.is_empty() {
        let mut modified_summary = "Modified: ".to_string();
        for (path, metrics) in modified_files.iter().take(5) {
            // Take up to 5 most relevant to keep it concise
            write!(
                modified_summary,
                "{} ({} patches, {:.0}% changed); ",
                path.display(),
                metrics.patch_count,
                metrics.percent_changed
            )
            .unwrap();
        }
        modified_summary.pop(); // Remove trailing space
        modified_summary.pop(); // Remove trailing semicolon
        writeln!(prompt, "{}", modified_summary).unwrap();
    }

    // --- Guidance and Threshold Warnings ---
    let mut guidance = String::from("Guidance: ");
    let mut has_guidance = false;

    if state.total_errors >= 3 {
        guidance.push_str("HIGH ERROR COUNT - Session drift likely. Reset or proceed with caution. ");
        has_guidance = true;
    }
    if state.exchange_count >= 10 {
        guidance.push_str("SESSION LIMIT REACHED - A refresh is required to continue. ");
        has_guidance = true;
    }

    for (path, metrics) in &state.files {
        if metrics.patch_count >= 8 {
            write!(guidance, "File '{}' has a high patch count. Consider a full-file replacement. ", path.display()).unwrap();
            has_guidance = true;
        }
        if metrics.percent_changed > 70.0 {
            write!(guidance, "File '{}' has changed significantly. A full-file replacement is recommended. ", path.display()).unwrap();
            has_guidance = true;
        }
    }

    if !has_guidance {
        guidance.push_str("System is healthy. Proceed with patch generation.");
    }

    writeln!(prompt, "{}", guidance).unwrap();
    writeln!(prompt, "[/SESSION CONTEXT]\n").unwrap();

    // --- Static Instructions ---
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
    prompt.push_str("- If you are unsure, ask for more information instead of generating a faulty patch.\n");

    prompt
}