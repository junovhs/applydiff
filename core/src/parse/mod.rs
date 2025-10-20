use crate::error::{ErrorCode, PatchError, Result};
use std::path::PathBuf;

mod parse_classic;
mod parse_armored;
mod parse_base64;

pub use parse_classic::parse_classic_block;
pub use parse_armored::parse_armored_block;
pub use parse_base64::decode_base64_lossy;

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
    pub fn new() -> Self { Self::default() }

    pub fn parse(&self, input: &str) -> Result<Vec<PatchBlock>> {
        let mut out: Vec<PatchBlock> = Vec::new();
        let mut lines = input.lines().enumerate().peekable();
        let mut block_count = 0usize;

        while let Some((_, line)) = lines.peek().cloned() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("-----BEGIN APPLYDIFF AFB-1-----") {
                block_count += 1;
                if block_count > MAX_BLOCKS {
                    return Err(PatchError::Validation {
                        code: ErrorCode::BoundsExceeded,
                        message: format!("Exceeded MAX_BLOCKS limit of {}", MAX_BLOCKS),
                        context: "parser".to_string(),
                    });
                }
                let blk = parse_armored_block(&mut lines)?;
                out.push(blk);
                continue;
            }

            if trimmed.starts_with(">>>") {
                block_count += 1;
                if block_count > MAX_BLOCKS {
                    return Err(PatchError::Validation {
                        code: ErrorCode::BoundsExceeded,
                        message: format!("Exceeded MAX_BLOCKS limit of {}", MAX_BLOCKS),
                        context: "parser".to_string(),
                    });
                }
                let blk = parse_classic_block(&mut lines)?;
                out.push(blk);
                continue;
            }

            lines.next();
        }

        if out.is_empty() {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "No patch blocks found".to_string(),
                context: "".to_string(),
            });
        }

        Ok(out)
    }
}