use crate::apply::Applier;
use crate::backup;
use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use crate::parser::Parser;

use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> String {
    let rid = (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64);
    let logger = Logger::new(rid);

    let mut log = String::new();
    logln(&mut log, "🧪 **Self-Test Gauntlet** starting…");

    // Create sandbox
    let base = match make_sandbox() {
        Ok(p) => p,
        Err(e) => return format!("❌ Failed to create sandbox: {e}"),
    };
    logln(&mut log, format!("📦 Sandbox: `{}`", base.display()));

    // Track totals
    let mut total_cases = 0usize;
    let mut cases_passed = 0usize;

    // ========== T01: exact single-line replace ==========
    total_cases += 1;
    case_header(&mut log, "T01 exact single-line replace");
    let t = base.join("t01");
    write_tree(&[(&t.join("hello.txt"), "Hello world\n")]).ok();
    let patch = blocks(&[Block {
        file: "t01/hello.txt",
        fuzz: 1.0,
        from: "Hello world",
        to: "Hello brave new world",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact("hello.txt", "Hello brave new world\n")],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T02: fuzzy replace (CRLF preserved) ==========
    total_cases += 1;
    case_header(&mut log, "T02 fuzzy replace + CRLF preserved");
    let t = base.join("t02");
    write_tree(&[(
        &t.join("web/app.js"),
        "function greet(){\r\n  console.log('Hello world');\r\n}\r\n",
    )])
    .ok();
    let patch = blocks(&[Block {
        file: "t02/web/app.js",
        fuzz: 0.85,
        from: "  console.log('Hello world');\n",
        to: "  console.log('Hello brave new world');\n",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact(
            "web/app.js",
            "function greet(){\r\n  console.log('Hello brave new world');\r\n}\r\n",
        )],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T03: append-only (empty from) ==========
    total_cases += 1;
    case_header(&mut log, "T03 append-only");
    let t = base.join("t03");
    write_tree(&[(
        &t.join("docs/readme.md"),
        "# Title\r\n\r\n- item A\r\n- item B\r\n",
    )])
    .ok();
    let patch = blocks(&[Block {
        file: "t03/docs/readme.md",
        fuzz: 1.0,
        from: "",
        to: "## Changelog\n- Added greeting",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Normalized(
            "docs/readme.md",
            "# Title\r\n\r\n- item A\r\n- item B\r\n## Changelog\n- Added greeting",
        )],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T04: newline preservation (CRLF) ==========
    total_cases += 1;
    case_header(&mut log, "T04 newline preservation (CRLF)");
    let t = base.join("t04");
    write_tree(&[(&t.join("file.txt"), "AAA\r\nBBB\r\nCCC\r\n")]).ok();
    let patch = blocks(&[Block {
        file: "t04/file.txt",
        fuzz: 1.0,
        from: "BBB\r\n",
        to: "BXX",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact("file.txt", "AAA\r\nBXX\r\nCCC\r\n")],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T05: no-match above fuzz threshold ==========
    total_cases += 1;
    case_header(&mut log, "T05 no-match (fuzz too strict)");
    let t = base.join("t05");
    write_tree(&[(&t.join("foo.txt"), "aaaa bbbb cccc\n")]).ok();
    let original = fs::read_to_string(t.join("foo.txt")).unwrap_or_default();
    let patch = blocks(&[Block {
        file: "t05/foo.txt",
        fuzz: 0.99,
        from: "axaa bxb cccc",
        to: "changed text",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact("foo.txt", &original)],
        expect_counts(0, 1),
    ) {
        cases_passed += 1;
    }

    // ========== T06: multiple blocks same file (non-overlapping) ==========
    total_cases += 1;
    case_header(&mut log, "T06 multiple blocks in same file");
    let t = base.join("t06");
    write_tree(&[(
        &t.join("config.ini"),
        "[core]\ncolor = auto\neditor = nano\n",
    )])
    .ok();
    let patch = blocks(&[
        Block {
            file: "t06/config.ini",
            fuzz: 1.0,
            from: "editor = nano",
            to: "editor = vim",
        },
        Block {
            file: "t06/config.ini",
            fuzz: 1.0,
            from: "color = auto",
            to: "color = always",
        },
    ]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact(
            "config.ini",
            "[core]\ncolor = always\neditor = vim\n",
        )],
        expect_counts(2, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T07: Unicode text ==========
    total_cases += 1;
    case_header(&mut log, "T07 unicode content");
    let t = base.join("t07");
    write_tree(&[(&t.join("unicode.txt"), "café naïve 😊\n")]).ok();
    let patch = blocks(&[Block {
        file: "t07/unicode.txt",
        fuzz: 1.0,
        from: "naïve",
        to: "savvy",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact("unicode.txt", "café savvy 😊\n")],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T08: missing file (should error; not created) ==========
    total_cases += 1;
    case_header(&mut log, "T08 missing file");
    let t = base.join("t08");
    fs::create_dir_all(&t).ok();
    let patch = blocks(&[Block {
        file: "t08/missing.txt",
        fuzz: 1.0,
        from: "",
        to: "hello",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Missing("missing.txt")],
        expect_counts(0, 1),
    ) {
        cases_passed += 1;
    }

    // ========== T09: large file (50k lines) exact replace near end ==========
    total_cases += 1;
    case_header(&mut log, "T09 large file 50k lines");
    let t = base.join("t09");
    fs::create_dir_all(&t).ok();
    let big_path = t.join("big.txt");
    {
        let mut data = String::new();
        for i in 0..50_000 {
            data.push_str(&format!("line {i}\n"));
        }
        fs::write(&big_path, data).ok();
    }
    let patch = blocks(&[Block {
        file: "t09/big.txt",
        fuzz: 1.0,
        from: "line 49999\n",
        to: "LINE 49999\n",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Contains("big.txt", "LINE 49999\n")],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T10: file without trailing newline ==========
    total_cases += 1;
    case_header(&mut log, "T10 file without trailing newline");
    let t = base.join("t10");
    write_tree(&[(&t.join("noeol.txt"), "last line no newline")]).ok();
    let patch = blocks(&[Block {
        file: "t10/noeol.txt",
        fuzz: 1.0,
        from: "last line no newline",
        to: "last line with newline",
    }]);
    if run_case(
        &logger,
        &mut log,
        &t,
        &patch,
        &[Expect::Exact("noeol.txt", "last line with newline")],
        expect_counts(1, 0),
    ) {
        cases_passed += 1;
    }

    // ========== T11: backup restore ==========
    total_cases += 1;
    case_header(&mut log, "T11 backup restore");
    let t = base.join("t11");
    write_tree(&[(&t.join("restore.txt"), "ORIGINAL\n")]).ok();
    let original = fs::read_to_string(t.join("restore.txt")).unwrap_or_default();
    let rels = vec![PathBuf::from("restore.txt")];

    let t11_passed = match backup::create_backup(&t, &rels) {
        Ok(_) => {
            let _ = fs::write(t.join("restore.txt"), "MUTATED\n");
            match backup::latest_backup(&t) {
                Some(bk) => match backup::restore_backup(&t, &bk) {
                    Ok(()) => {
                        let mut vpass = 0usize;
                        let mut vfail = 0usize;
                        verify_eq(
                            &mut log,
                            &t.join("restore.txt"),
                            &original,
                            &mut vpass,
                            &mut vfail,
                            "expect restored content",
                        );
                        vfail == 0
                    }
                    Err(e) => {
                        logln(&mut log, format!("  ❌ restore failed: {}", e));
                        false
                    }
                },
                None => {
                    logln(&mut log, "  ❌ no backup found to restore");
                    false
                }
            }
        }
        Err(e) => {
            logln(&mut log, format!("  ❌ create_backup failed: {}", e));
            false
        }
    };
    if t11_passed {
        logln(&mut log, "  ✅ case passed");
        cases_passed += 1;
    } else {
        logln(&mut log, "  ❌ case failed");
    }

    // Prompt example must parse (clipboard contract)
    {
        use crate::prompts::example_patch;
        let parser2 = Parser::new();
        match parser2.parse(&example_patch()) {
            Ok(v) if !v.is_empty() => logln(&mut log, "🧩 Prompt example: ✓ parser accepted"),
            Ok(_) => logln(&mut log, "🧩 Prompt example: ❌ no blocks found"),
            Err(e) => logln(&mut log, format!("🧩 Prompt example: ❌ parse failed: {}", e)),
        }
    }

    // Cleanup
    log_cleanup(&mut log, &base);

    // Summary
    logln(&mut log, format!("\n🧾 **Cases Passed**: {}/{}", cases_passed, total_cases));
    if cases_passed == total_cases {
        logln(&mut log, "\n✅ **Self-Test PASSED**");
    } else {
        logln(&mut log, "\n❌ **Self-Test FAILED** — see failed cases above");
    }

    log
}

/* ---------------- case runner & helpers ---------------- */

#[derive(Clone, Copy)]
struct Counts {
    ok: usize,
    fail: usize,
}
fn expect_counts(ok: usize, fail: usize) -> Counts {
    Counts { ok, fail }
}

#[derive(Clone, Copy)]
struct Block<'a> {
    file: &'a str,
    fuzz: f32,
    from: &'a str,
    to: &'a str,
}

fn blocks(specs: &[Block]) -> String {
    let mut s = String::new();
    for b in specs {
        s.push_str(&format!(
            ">>> file: {} | fuzz={}\n--- from\n{}\n--- to\n{}\n<<<\n\n",
            b.file, b.fuzz, b.from, b.to
        ));
    }
    s
}

enum Expect<'a> {
    Exact(&'a str, &'a str),
    Normalized(&'a str, &'a str),
    Contains(&'a str, &'a str),
    Missing(&'a str),
}

fn run_case(
    logger: &Logger,
    log: &mut String,
    dir: &Path,
    patch: &str,
    expects: &[Expect],
    counts: Counts,
) -> bool {
    // parse
    let parser = Parser::new();
    let blocks = match parser.parse(patch) {
        Ok(b) => b,
        Err(e) => {
            logln(log, format!("  ❌ parse failed: {}", e));
            return false;
        }
    };
    logln(log, format!("  • parsed {} block(s)", blocks.len()));

    // preview (dry-run)
    let previewer = Applier::new(logger, dir.parent().unwrap_or(dir).to_path_buf(), true);
    for (i, b) in blocks.iter().enumerate() {
        match previewer.apply_block(b) {
            Ok(res) => logln(
                log,
                format!(
                    "    ✓ preview block {} at {} (score {:.2})",
                    i + 1,
                    res.matched_at,
                    res.score
                ),
            ),
            Err(e) => logln(log, format!("    ❌ preview block {}: {}", i + 1, e)),
        }
    }

    // apply
    let applier = Applier::new(logger, dir.parent().unwrap_or(dir).to_path_buf(), false);
    let mut oka = 0usize;
    let mut faila = 0usize;
    for (i, b) in blocks.iter().enumerate() {
        match applier.apply_block(b) {
            Ok(res) => {
                oka += 1;
                logln(
                    log,
                    format!(
                        "    ✓ apply block {} at {} (score {:.2})",
                        i + 1,
                        res.matched_at,
                        res.score
                    ),
                );
            }
            Err(e) => {
                faila += 1;
                logln(log, format!("    ❌ apply block {}: {}", i + 1, e));
            }
        }
    }

    let counts_ok = oka == counts.ok && faila == counts.fail;
    if !counts_ok {
        logln(
            log,
            format!(
                "  ❌ expected apply counts ok={} fail={}, got ok={} fail={}",
                counts.ok, counts.fail, oka, faila
            ),
        );
    }

    // verify
    let mut vfail = 0usize;
    for e in expects {
        match e {
            Expect::Exact(rel, want) => {
                let mut vpass = 0usize;
                verify_eq(log, &dir.join(rel), want, &mut vpass, &mut vfail, &format!("expect exact {}", rel));
            }
            Expect::Normalized(rel, want) => {
                let mut vpass = 0usize;
                verify_eq_normalized(
                    log,
                    &dir.join(rel),
                    want,
                    &mut vpass,
                    &mut vfail,
                    &format!("expect normalized {}", rel),
                );
            }
            Expect::Contains(rel, needle) => {
                let mut vpass = 0usize;
                verify_contains(
                    log,
                    &dir.join(rel),
                    needle,
                    &mut vpass,
                    &mut vfail,
                    &format!("expect contains {}", rel),
                );
            }
            Expect::Missing(rel) => {
                let mut vpass = 0usize;
                verify_missing(
                    log,
                    &dir.join(rel),
                    &mut vpass,
                    &mut vfail,
                    &format!("expect missing {}", rel),
                );
            }
        }
    }

    let passed = counts_ok && vfail == 0;
    if passed {
        logln(log, "  ✅ case passed");
    } else {
        logln(log, "  ❌ case failed");
    }
    passed
}

fn make_sandbox() -> Result<PathBuf> {
    let root = std::env::temp_dir();
    let dir = root.join(format!(
        "applydiff_selftest_{}",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
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
            logln(log, format!("    ✓ {}: OK", label));
        }
        Ok(got) => {
            *bad += 1;
            logln(
                log,
                format!(
                    "    ❌ {}: mismatch\n      expected:\n----\n{}\n----\n      got:\n----\n{}\n----",
                    label, expected, got
                ),
            );
        }
        Err(e) => {
            *bad += 1;
            logln(log, format!("    ❌ {}: read failed: {}", label, e));
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
            logln(log, format!("    ✓ {}: OK (normalized)", label));
        }
        Ok(got) => {
            *bad += 1;
            logln(
                log,
                format!(
                    "    ❌ {}: mismatch (normalized)\n      expected:\n----\n{}\n----\n      got:\n----\n{}\n----",
                    label, expected, got
                ),
            );
        }
        Err(e) => {
            *bad += 1;
            logln(log, format!("    ❌ {}: read failed: {}", label, e));
        }
    }
}

fn verify_contains(
    log: &mut String,
    path: &Path,
    needle: &str,
    ok: &mut usize,
    bad: &mut usize,
    label: &str,
) {
    match fs::read_to_string(path) {
        Ok(got) if got.contains(needle) => {
            *ok += 1;
            logln(log, format!("    ✓ {}: contains {:?}", label, needle));
        }
        Ok(got) => {
            *bad += 1;
            let snippet = &got[..got.len().min(200)];
            logln(
                log,
                format!(
                    "    ❌ {}: does not contain {:?}\n      got snippet:\n----\n{}\n----",
                    label, needle, snippet
                ),
            );
        }
        Err(e) => {
            *bad += 1;
            logln(log, format!("    ❌ {}: read failed: {}", label, e));
        }
    }
}

fn verify_missing(
    log: &mut String,
    path: &Path,
    ok: &mut usize,
    bad: &mut usize,
    label: &str,
) {
    if !path.exists() {
        *ok += 1;
        logln(log, format!("    ✓ {}: file not present (as expected)", label));
    } else {
        *bad += 1;
        logln(log, format!("    ❌ {}: file exists but should be missing", label));
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

fn case_header(log: &mut String, name: &str) {
    logln(log, format!("\n— {} —", name));
}

fn logln<S: Into<String>>(buf: &mut String, s: S) {
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf.push_str(&s.into());
}
