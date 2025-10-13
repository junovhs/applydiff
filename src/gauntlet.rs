use crate::apply::Applier;
use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use crate::parser::Parser;

use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Run a battery of end-to-end tests in a temporary directory:
/// - exact replace
/// - fuzzy replace (typos/whitespace)
/// - append-only (empty from)
/// - multi-line with CRLF vs LF normalization
/// - multi-file batch
/// - prompt example must parse (keeps clipboard prompt honest)
///
/// Returns a markdown-ish log string. Cleans up the temp directory at the end.
pub fn run() -> String {
    let rid = (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64);
    let logger = Logger::new(rid);

    let mut log = String::new();
    logln(&mut log, "🧪 **Self-Test Gauntlet** starting…");

    // 1) Create sandbox
    let base = match make_sandbox() {
        Ok(p) => p,
        Err(e) => return format!("❌ Failed to create sandbox: {e}"),
    };
    logln(&mut log, format!("📦 Sandbox: `{}`", base.display()));

    // Build initial files
    let hello = base.join("hello.txt");
    let js    = base.join("web/app.js");
    let md    = base.join("docs/readme.md");

    if let Err(e) = write_tree(&[
        (&hello, "Hello world\n"),
        (&js,    "function greet(){\r\n  console.log('Helo wrld');\r\n}\r\n"),
        (&md,    "# Title\r\n\r\n- item A\r\n- item B\r\n"),
    ]) {
        logln(&mut log, format!("❌ Failed to write initial files: {e}"));
        log_cleanup(&mut log, &base);
        return log;
    }

    // 2) Build one big patch
    let patch = r#"
>>> file: hello.txt | fuzz=1.0
--- from
Hello world
--- to
Hello brave new world
<<<

>>> file: web/app.js | fuzz=0.80
--- from
  console.log('Hello world');
--- to
  console.log('Hello brave new world');
<<<

>>> file: docs/readme.md | fuzz=1.0
--- from

--- to
## Changelog
- Added greeting
<<<
"#;

    // 3) Preview (dry run)
    logln(&mut log, "\n👁 **Preview** (dry-run):");
    let parser = Parser::new();
    let blocks = match parser.parse(patch) {
        Ok(b) => b,
        Err(e) => {
            logln(&mut log, format!("❌ Parse failed: {e}"));
            log_cleanup(&mut log, &base);
            return log;
        }
    };
    logln(&mut log, format!("✓ Parsed {} block(s)", blocks.len()));

    let applier_preview = Applier::new(&logger, base.clone(), true);
    let mut preview_ok = 0usize;
    for (i, b) in blocks.iter().enumerate() {
        logln(&mut log, format!("- Block {}: `{}`", i + 1, b.file.display()));
        match applier_preview.apply_block(b) {
            Ok(res) => {
                preview_ok += 1;
                logln(&mut log, format!("  ✓ match at {} (score {:.2})", res.matched_at, res.score));
            }
            Err(e) => logln(&mut log, format!("  ❌ {e}")),
        }
    }
    if preview_ok != blocks.len() {
        logln(&mut log, "❌ Preview had failures; aborting apply.");
        log_cleanup(&mut log, &base);
        return log;
    }
    logln(&mut log, "✓ Preview complete.");

    // 4) Apply (real)
    logln(&mut log, "\n⚙️ **Apply** (real changes):");
    let applier = Applier::new(&logger, base.clone(), false);
    let mut applied = 0usize;
    for (i, b) in blocks.iter().enumerate() {
        logln(&mut log, format!("- Block {}: `{}`", i + 1, b.file.display()));
        match applier.apply_block(b) {
            Ok(res) => {
                applied += 1;
                logln(&mut log, format!("  ✓ applied at {} (score {:.2})", res.matched_at, res.score));
            }
            Err(e) => logln(&mut log, format!("  ❌ {e}")),
        }
    }
    logln(&mut log, format!("✓ Applied {} / {}", applied, blocks.len()));

    // 5) Verify expectations
    let mut verify_pass = 0usize;
    let mut verify_fail = 0usize;

    // hello.txt expected
    verify_eq(
        &mut log,
        &hello,
        "Hello brave new world\n",
        &mut verify_pass,
        &mut verify_fail,
        "hello.txt exact replacement",
    );

    // web/app.js expected (CRLF preserved except changed line content)
    let js_expected = "function greet(){\r\n  console.log('Hello brave new world');\r\n}\r\n";
    verify_eq(
        &mut log,
        &js,
        js_expected,
        &mut verify_pass,
        &mut verify_fail,
        "web/app.js fuzzy replacement (CRLF)",
    );

    // docs/readme.md expected (append-only) – normalized comparison
    let md_expected = "# Title\r\n\r\n- item A\r\n- item B\r\n## Changelog\n- Added greeting";
    verify_eq_normalized(
        &mut log,
        &md,
        md_expected,
        &mut verify_pass,
        &mut verify_fail,
        "docs/readme.md append-only",
    );

    logln(&mut log, format!("\n🧾 **Verification**: {} passed, {} failed", verify_pass, verify_fail));

    // 5b) Prompt example must parse (sanity for the clipboard prompt)
    {
        use crate::prompts::example_patch;
        let parser2 = Parser::new();
        match parser2.parse(&example_patch()) {
            Ok(v) if !v.is_empty() => logln(&mut log, "🧩 Prompt example: ✓ parser accepted"),
            Ok(_) => logln(&mut log, "🧩 Prompt example: ❌ no blocks found"),
            Err(e) => logln(&mut log, format!("🧩 Prompt example: ❌ parse failed: {}", e)),
        }
    }

    // 6) Cleanup
    log_cleanup(&mut log, &base);

    if verify_fail == 0 {
        logln(&mut log, "\n✅ **Self-Test PASSED**");
    } else {
        logln(&mut log, "\n❌ **Self-Test FAILED** — see details above");
    }

    log
}

/* ---------------- helpers ---------------- */

fn make_sandbox() -> Result<PathBuf> {
    let root = std::env::temp_dir();
    let dir = root.join(format!("applydiff_selftest_{}", Local::now().format("%Y%m%d_%H%M%S")));
    fs::create_dir_all(&dir).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("create sandbox failed: {e}"),
        path: dir.clone(),
    })?;
    Ok(dir)
}

fn write_tree(specs: &[(&PathBuf, &str)]) -> Result<()> {
    for (path, contents) in specs {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("mkdir {:?} failed: {e}", parent),
                path: parent.to_path_buf(),
            })?;
        }
        fs::write(path, contents).map_err(|e| PatchError::File {
            code: ErrorCode::FileWriteFailed,
            message: format!("write {:?} failed: {e}", path),
            path: (*path).clone(),
        })?;
    }
    Ok(())
}

fn verify_eq(
    log: &mut String,
    path: &Path,
    expected: &str,
    ok: &mut usize,
    bad: &mut usize,
    label: &str,
) {
    match fs::read_to_string(path) {
        Ok(got) if got == expected => {
            *ok += 1;
            logln(log, format!("  ✓ {}: OK", label));
        }
        Ok(got) => {
            *bad += 1;
            logln(log, format!("  ❌ {}: mismatch\n    expected:\n----\n{}\n----\n    got:\n----\n{}\n----", label, expected, got));
        }
        Err(e) => {
            *bad += 1;
            logln(log, format!("  ❌ {}: read failed: {}", label, e));
        }
    }
}

fn verify_eq_normalized(
    log: &mut String,
    path: &Path,
    expected: &str,
    ok: &mut usize,
    bad: &mut usize,
    label: &str,
) {
    let norm = |s: &str| s.replace("\r\n", "\n");
    match fs::read_to_string(path) {
        Ok(got) if norm(&got) == norm(expected) => {
            *ok += 1;
            logln(log, format!("  ✓ {}: OK (normalized)", label));
        }
        Ok(got) => {
            *bad += 1;
            logln(log, format!("  ❌ {}: mismatch (normalized compare)\n    expected:\n----\n{}\n----\n    got:\n----\n{}\n----", label, expected, got));
        }
        Err(e) => {
            *bad += 1;
            logln(log, format!("  ❌ {}: read failed: {}", label, e));
        }
    }
}

fn cleanup(dir: &Path) -> Result<()> {
    fs::remove_dir_all(dir).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("remove sandbox failed: {e}"),
        path: dir.to_path_buf(),
    })
}

fn log_cleanup(log: &mut String, dir: &Path) {
    match cleanup(dir) {
        Ok(()) => logln(log, "🧹 Cleanup: removed sandbox."),
        Err(e) => logln(log, format!("⚠️ Cleanup warning: {e}")),
    }
}

fn logln<S: Into<String>>(buf: &mut String, s: S) {
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf.push_str(&s.into());
}
