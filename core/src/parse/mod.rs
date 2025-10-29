use crate::error::{ErrorCode, PatchError, Result};
use std::path::PathBuf;

pub mod parse_classic;

const MAX_BLOCKS: usize = 1000;

#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub file: PathBuf,
    pub from: String,
    pub to: String,
    pub fuzz: f64,
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses an input string for "classic" style patch blocks.
    pub fn parse(&self, input: &str) -> Result<Vec<PatchBlock>> {
        assert!(
            input.len() < 100_000_000,
            "Input size exceeds safety limit"
        );
        let mut blocks: Vec<PatchBlock> = Vec::new();
        let mut lines = input.lines().peekable();

        while lines.peek().is_some() {
            if let Some(line) = lines.peek() {
                if line.trim_start().starts_with(">>>") {
                    if blocks.len() >= MAX_BLOCKS {
                        return Err(PatchError::Validation {
                            code: ErrorCode::BoundsExceeded,
                            message: format!("Exceeded MAX_BLOCKS limit of {}", MAX_BLOCKS),
                            context: "parser".to_string(),
                        });
                    }
                    let block = parse_classic::parse_classic_block(&mut lines)?;
                    blocks.push(block);
                } else {
                    lines.next(); // Skip non-header lines
                }
            }
        }

        if blocks.is_empty() {
            return Err(PatchError::Parse {
                code: ErrorCode::NoBlocksFound,
                message: "No patch blocks found in the input".to_string(),
                context: "parser".to_string(),
            });
        }

        Ok(blocks)
    }
}