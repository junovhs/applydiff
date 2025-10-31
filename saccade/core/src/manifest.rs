use crate::config::Config;
use crate::detection::BuildSystemType;
use crate::error::Result;
use crate::stage0::Stage0Generator;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ManifestGenerator { config: Config }
pub struct ProjectInfoContext<'a> {
    pub raw_count: usize,
    pub filtered_count: usize,
    pub pack_dir: &'a Path,
    pub in_git: bool,
    pub files: &'a [PathBuf],
    pub detected_systems: &'a [BuildSystemType],
}

impl ManifestGenerator {
    pub fn new(config: Config) -> Self { Self { config } }
    pub fn generate_project_info(&self, ctx: &ProjectInfoContext) -> Result<String> {
        let now: DateTime<Local> = Local::now();
        let git_commit = if ctx.in_git { self.get_git_commit() } else { None };
        let stage0 = Stage0Generator::new(self.config.clone());
        let lang_snapshot = stage0.generate_languages(ctx.files)?;
        let detected_str: Vec<String> = ctx.detected_systems.iter().map(|s| s.to_string()).collect();

        let mut out = String::new();
        out.push_str(&format!("Generated: {}\n", now.format("%Y-%m-%d %H:%M:%S")));
        out.push_str(&format!("- files.kept: {}\n", ctx.filtered_count));
        if let Some(commit) = git_commit { out.push_str(&format!("- git.commit: {}\n", commit)); }
        out.push_str(&format!("- detected_build_systems: [{}]\n\n", detected_str.join(", ")));
        out.push_str("LANGUAGE SNAPSHOT\n-------------------\n");
        out.push_str(&lang_snapshot);
        Ok(out)
    }
    fn get_git_commit(&self) -> Option<String> {
        Command::new("git").args(["rev-parse", "--short", "HEAD"]).output().ok()
            .filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}