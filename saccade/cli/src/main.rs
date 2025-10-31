use anyhow::Result;
use clap::Parser;
use saccade_core::config::{Config, GitMode};
use saccade_core::SaccadePack;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "saccade")]
struct Cli {
    #[arg(short, long, default_value = "ai-pack")]
    out: PathBuf,
    #[arg(long, default_value = "3")]
    max_depth: usize,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::new();
    config.pack_dir = cli.out;
    config.max_depth = cli.max_depth;
    config.code_only = cli.code_only;
    config.dry_run = cli.dry_run;
    config.verbose = cli.verbose;
    config.git_mode = if cli.git_only { GitMode::Yes } else if cli.no_git { GitMode::No } else { GitMode::Auto };
    
    let pack = SaccadePack::new(config);
    pack.generate()?;
    Ok(())
}