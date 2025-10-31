use super::state::SessionState;
use std::fmt::Write;

/// Builds the dynamic session briefing for the AI based on current state.
#[must_use]
pub fn build_session_briefing(session: &SessionState) -> String {
    let mut briefing = String::new();

    writeln!(&mut briefing, "[SESSION CONTEXT]").unwrap();
    writeln!(&mut briefing, "- Exchange Count: {}/10", session.exchange_count).unwrap();
    writeln!(&mut briefing, "- Prediction Errors: {}/3", session.total_errors).unwrap();

    if session.total_errors >= 3 {
        briefing.push_str("\n!! DRIFT LIKELY - HIGH ERROR COUNT !!\n");
    } else if session.exchange_count >= 10 {
        briefing.push_str("\n!! EXCHANGE LIMIT REACHED !!\n");
    }

    if !session.keystone_files.is_empty() {
        briefing.push_str("\n[KEYSTONE FILES (CRITICAL)]\n");
        for file in &session.keystone_files {
            writeln!(&mut briefing, "- {}", file.display()).unwrap();
        }
    }

    briefing.push_str("\n[ACTION TEMPLATE]\n");
    briefing.push_str("Goal: <...>\n");
    briefing.push_str("Evidence: <PASTE COMPILER/TEST ERRORS HERE.>\n\n");

    briefing.push_str("[APPLYDIFF PATCH FORMAT]\n");
    briefing.push_str(">>> file: <path> [| mode=replace]\n");
    briefing.push_str("--- from\n<...>\n");
    briefing.push_str("--- to\n<...>\n");
    briefing.push_str("<<<\n");

    briefing
}