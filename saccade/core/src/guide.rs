use crate::error::Result;
use std::path::Path;

const GUIDE_CONTENT: &str = r#"========================================
SACCADE PACK GUIDE (Single-File)
========================================
Saccade now generates ONE file: PACK.txt
with clearly labeled sections.
...
(Full guide content omitted for brevity)
...
========================================
"#;

pub struct GuideGenerator;

impl GuideGenerator {
    pub fn new() -> Self { Self }
    pub fn generate_guide(&self) -> Result<String> { Ok(GUIDE_CONTENT.to_string()) }
    pub fn print_guide(&self, pack_dir: &Path, _has_deps: bool) -> Result<()> {
        let absolute_pack_dir = dunce::canonicalize(pack_dir)?;
        eprintln!("✅ Success! Generated pack");
        eprintln!("   In: {}\n", absolute_pack_dir.display());
        Ok(())
    }
}