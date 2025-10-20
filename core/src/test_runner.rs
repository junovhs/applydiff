use crate::apply::Applier;
use crate::logger::Logger;
use crate::parse::Parser;
use crate::test_helpers::*;
use chrono::Local;
use serde::Deserialize;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Deserialize, Debug)]
struct TestMeta {
    description: String,
    expect_ok: usize,
    expect_fail: usize,
    expected_log_contains: Option<String>,
}

pub fn run() -> String {
    let rid = (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64);
    
    let mut log = String::new();
    logln(&mut log, "🧪 **Self-Test Gauntlet** starting…");

    let tests_root = match find_tests_dir() {
        Some(path) => path,
        None => return "❌ Could not find 'tests' directory in project root.".to_string(),
    };
    logln(&mut log, format!("📂 Found test suite at: {}", tests_root.display()));

    let mut test_cases = 0;
    let mut cases_passed = 0;

    let entries = match fs::read_dir(&tests_root) {
        Ok(iter) => iter.collect::<std::io::Result<Vec<_>>>().unwrap_or_default(),
        Err(e) => return format!("❌ Failed to read 'tests' directory: {}", e),
    };
    
    for entry in entries {
        if entry.path().is_dir() {
            test_cases += 1;
            let case_name = entry.file_name().to_string_lossy().to_string();
            case_header(&mut log, &case_name);

            if run_test_case(rid, &mut log, &entry.path()) {
                cases_passed += 1;
                logln(&mut log, "  ✅ case passed");
            } else {
                logln(&mut log, "  ❌ case failed");
            }
        }
    }

    logln(&mut log, format!("\n🧾 **Cases Passed**: {}/{}", cases_passed, test_cases));
    if cases_passed == test_cases && test_cases > 0 {
        logln(&mut log, "\n✅ **Self-Test PASSED**");
    } else {
        logln(&mut log, "\n❌ **Self-Test FAILED** – see failed cases above");
    }

    log
}

fn run_test_case(rid: u64, log: &mut String, case_path: &Path) -> bool {
    let sandbox = match make_sandbox() {
        Ok(p) => p,
        Err(e) => {
            logln(log, format!("  ❌ Sandbox creation failed: {}", e));
            return false;
        }
    };
    
    let meta_path = case_path.join("meta.json");
    let meta: TestMeta = match fs::read_to_string(&meta_path) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                logln(log, format!("  ❌ Failed to parse meta.json: {}", e));
                cleanup(&sandbox).ok();
                return false;
            }
        },
        Err(e) => {
            logln(log, format!("  ❌ Failed to read meta.json: {}", e));
            cleanup(&sandbox).ok();
            return false;
        }
    };
    logln(log, format!("  • {}", meta.description));

    let before_dir = case_path.join("before");
    if let Err(e) = copy_dir_all(&before_dir, &sandbox) {
        logln(log, format!("  ❌ Failed to copy 'before' state: {}", e));
        cleanup(&sandbox).ok();
        return false;
    }

    let patch_path = case_path.join("patch.txt");
    let patch_content = match fs::read_to_string(&patch_path) {
        Ok(p) => p,
        Err(e) => {
            logln(log, format!("  ❌ Failed to read patch.txt: {}", e));
            cleanup(&sandbox).ok();
            return false;
        }
    };
    
    let log_buffer = Rc::new(RefCell::new(String::new()));
    let logger = Logger::new_for_test(rid, Some(log_buffer.clone()));

    let parser = Parser::new();
    let blocks = match parser.parse(&patch_content) {
        Ok(b) => b,
        Err(e) => {
            logln(log, format!("  ❌ Patch parsing failed: {}", e));
            cleanup(&sandbox).ok();
            return false;
        }
    };

    let applier = Applier::new(&logger, sandbox.clone(), false);
    let mut ok_count = 0;
    let mut fail_count = 0;
    for block in &blocks {
        match applier.apply_block(block) {
            Ok(_) => ok_count += 1,
            Err(_) => fail_count += 1,
        }
    }

    let mut checks_passed = true;

    if ok_count != meta.expect_ok || fail_count != meta.expect_fail {
        logln(log, format!(
            "    ❌ Mismatch in apply counts. Expected ok={}, fail={}. Got ok={}, fail={}.",
            meta.expect_ok, meta.expect_fail, ok_count, fail_count
        ));
        checks_passed = false;
    } else {
        logln(log, format!("    ✓ Apply counts match (ok={}, fail={})", ok_count, fail_count));
    }
    
    if let Some(expected_str) = meta.expected_log_contains {
        if !log_buffer.borrow().contains(&expected_str) {
            logln(log, format!("    ❌ Log verification failed. Did not find '{}'.", expected_str));
            checks_passed = false;
        } else {
            logln(log, format!("    ✓ Log verification passed. Found '{}'.", expected_str));
        }
    }

    let after_dir = case_path.join("after");
    if let Err(e) = verify_dirs_match(log, &sandbox, &after_dir) {
        logln(log, format!("    ❌ File verification failed: {}", e));
        checks_passed = false;
    }

    cleanup(&sandbox).ok();
    checks_passed
}

fn find_tests_dir() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    loop {
        let tests_path = current.join("tests");
        if tests_path.is_dir() {
            return Some(tests_path);
        }
        if !current.pop() { break; }
    }
    
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let manifest_path = PathBuf::from(manifest_dir);
    let parent = manifest_path.parent()?;
    let tests = parent.join("tests");
    if tests.is_dir() {
        return Some(tests);
    }
    
    None
}