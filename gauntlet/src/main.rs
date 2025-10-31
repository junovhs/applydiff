use anyhow::{anyhow, Result};
use applydiff_backend::commands::{self, AppState};
use applydiff_core::session::state::SessionState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::TempDir;

struct TestContext {
    _temp_dir: TempDir,
    project_root: PathBuf,
    app_state: AppState,
}

struct Test {
    name: &'static str,
    run: fn() -> Result<()>,
}

macro_rules! tests {
    ($($test_name:ident),*) => { [ $(Test { name: stringify!($test_name), run: $test_name }),* ] };
}

fn main() {
    let tests = tests![
        a1_whole_file_parser, a2_regex_parser, a3_session_initialization, a4_initial_state_check,
        b1_pe_tracking_no_match, b2_pe_tracking_ambiguous_match, b3_successful_patch_metrics,
        c1_automated_file_request_path, c2_automated_file_request_range, c3_automated_file_request_symbol,
        c4_dynamic_briefing_content, c5_threshold_enforcement_error_block, c6_threshold_enforcement_exchange_block,
        c7_session_refresh
    ];
    println!("Running Saccade Integration Test Suite...");
    println!("========================================");
    let (passed, total) = tests.iter().fold((0, 0), |(mut passed, total), test| {
        print!("  - Running Test [{}]: ", test.name);
        match (test.run)() {
            Ok(()) => { println!("\x1B[32mPASS\x1B[0m"); passed += 1; }
            Err(e) => { println!("\x1B[31mFAIL\x1B[0m"); eprintln!("    Error: {e:?}"); }
        }
        (passed, total + 1)
    });
    println!("========================================");
    println!("Gauntlet Summary:");
    println!("  Total: {total}");
    println!("  \x1B[32mPass : {passed}\x1B[0m");
    println!("  \x1B[31mFail : {}\x1B[0m", total - passed);
    println!("========================================");
    if passed != total { std::process::exit(1); }
}

fn a1_whole_file_parser() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "Hello World")])?;
    let patch = ">>> file: file.txt | mode=replace\nThis is the new content.\n<<<";
    commands::apply_patch_logic(patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    let content = fs::read_to_string(ctx.project_root.join("file.txt"))?;
    assert_eq!(content, "This is the new content.");
    Ok(())
}

fn a2_regex_parser() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "hello world, hello universe")])?;
    let patch = ">>> file: file.txt | mode=regex\n--- from\nhello\n--- to\ngoodbye\n<<<";
    commands::apply_patch_logic(patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    let content = fs::read_to_string(ctx.project_root.join("file.txt"))?;
    assert_eq!(content, "goodbye world, goodbye universe");
    Ok(())
}

fn a3_session_initialization() -> Result<()> {
    let ctx = setup_project(&[("src/main.rs", "fn main() {}")])?;
    assert!(ctx.project_root.join(".applydiff/session.json").exists());
    Ok(())
}

fn a4_initial_state_check() -> Result<()> {
    let ctx = setup_project(&[("README.md", "Test")])?;
    let state = read_session_state(&ctx.project_root)?;
    assert_eq!(state.total_errors, 0);
    assert_eq!(state.exchange_count, 0);
    Ok(())
}

fn b1_pe_tracking_no_match() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "original")])?;
    let patch = ">>> file: file.txt\n--- from\nnon-existent\n--- to\nnew\n<<<";
    commands::apply_patch_logic(patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    assert_eq!(read_session_state(&ctx.project_root)?.total_errors, 1);
    Ok(())
}

fn b2_pe_tracking_ambiguous_match() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "one\ntwo\none")])?;
    let patch = ">>> file: file.txt\n--- from\none\n--- to\nthree\n<<<";
    commands::apply_patch_logic(patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    assert_eq!(read_session_state(&ctx.project_root)?.total_errors, 1);
    Ok(())
}

fn b3_successful_patch_metrics() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "line one")])?;
    let patch = ">>> file: file.txt\n--- from\nline one\n--- to\nline 1\n<<<";
    commands::apply_patch_logic(patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    let state = read_session_state(&ctx.project_root)?;
    assert_eq!(state.exchange_count, 1);
    assert_eq!(state.file_metrics.get(Path::new("file.txt")).unwrap().patch_count, 1);
    Ok(())
}

fn c1_automated_file_request_path() -> Result<()> {
    let ctx = setup_project(&[("src/lib.rs", "pub fn hello() {}")])?;
    let request_yaml = "path: src/lib.rs\nreason: test";
    let markdown = commands::resolve_file_request_logic(request_yaml, &ctx.app_state).map_err(|e| anyhow!(e))?;
    assert!(markdown.contains("src/lib.rs") && markdown.contains("pub fn hello() {}"));
    Ok(())
}

fn c2_automated_file_request_range() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "1\n2\n3\n4\n5")])?;
    let request_yaml = "path: file.txt\nrange: lines 2-4";
    let markdown = commands::resolve_file_request_logic(request_yaml, &ctx.app_state).map_err(|e| anyhow!(e))?;
    assert!(markdown.contains("lines 2-4 of 5") && markdown.contains("2\n3\n4") && !markdown.contains('1'));
    Ok(())
}

fn c3_automated_file_request_symbol() -> Result<()> {
    let ctx = setup_project(&[("code.rs", "line1\nline2\nfn my_symbol(){}\nline4\nline5\nline6\nline7\nline8\nline9\nline10")])?;
    let request_yaml = "path: code.rs\nrange: symbol: my_symbol";
    let markdown = commands::resolve_file_request_logic(request_yaml, &ctx.app_state).map_err(|e| anyhow!(e))?;
    assert!(markdown.contains("symbol 'my_symbol' at line 3 (±5 lines context)"));
    Ok(())
}

fn c4_dynamic_briefing_content() -> Result<()> {
    let ctx = setup_project(&[])?;
    {
        let mut g = ctx.app_state.0.lock().unwrap();
        let s = g.as_mut().unwrap();
        s.total_errors = 2;
        s.exchange_count = 5;
    }
    let briefing = commands::get_session_briefing_logic(&ctx.app_state).map_err(|e| anyhow!(e))?;
    assert!(briefing.contains("Exchange Count: 5/10") && briefing.contains("Prediction Errors: 2/3"));
    Ok(())
}

fn c5_threshold_enforcement_error_block() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "a")])?;
    let bad_patch = ">>> file: file.txt\n--- from\nx\n--- to\ny\n<<<";
    for _ in 0..3 {
        commands::apply_patch_logic(bad_patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
    }
    assert_eq!(read_session_state(&ctx.project_root)?.total_errors, 3);
    Ok(())
}

fn c6_threshold_enforcement_exchange_block() -> Result<()> {
    let ctx = setup_project(&[("file.txt", "line 1")])?;
    for i in 0..10 {
        let from = if i == 0 { "line 1".to_string() } else { format!("line {}", i + 1) };
        let to = format!("line {}", i + 2);
        let patch = format!(">>> file: file.txt\n--- from\n{from}\n--- to\n{to}\n<<<");
        commands::apply_patch_logic(&patch, &ctx.app_state).map_err(|e| anyhow!(e))?;
        fs::write(ctx.project_root.join("file.txt"), &to)?;
    }
    assert_eq!(read_session_state(&ctx.project_root)?.exchange_count, 10);
    Ok(())
}

fn c7_session_refresh() -> Result<()> {
    let ctx = setup_project(&[])?;
    {
        let mut g = ctx.app_state.0.lock().unwrap();
        g.as_mut().unwrap().exchange_count = 10;
    }
    commands::refresh_session_logic(&ctx.app_state).map_err(|e| anyhow!(e))?;
    assert_eq!(read_session_state(&ctx.project_root)?.exchange_count, 0);
    Ok(())
}

fn setup_project(files: &[(&str, &str)]) -> Result<TestContext> {
    let temp_dir = TempDir::new()?;
    let project_root = temp_dir.path().to_path_buf(); // Create an owned PathBuf immediately.
    for (name, content) in files {
        let path = project_root.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)?;
    }
    let session_state = commands::init_session_logic(&project_root).map_err(|e| anyhow!(e))?;
    let app_state = AppState(Mutex::new(Some(session_state)));
    Ok(TestContext { _temp_dir: temp_dir, project_root, app_state })
}

fn read_session_state(project_root: &Path) -> Result<SessionState> {
    let content = fs::read_to_string(project_root.join(".applydiff/session.json"))?;
    Ok(serde_json::from_str(&content)?)
}