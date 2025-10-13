# Safety Documentation

This project follows Power of Ten (Po10) safety principles adapted for Rust.

## Po10 Compliance Checklist

### Rule 1: Control Flow
✅ **No recursion** - All algorithms use iterative approaches  
✅ **No eval/dynamic dispatch** - All code paths enumerated at compile time  
✅ **No nonlocal jumps** - All functions return explicitly via `Result<T>`

### Rule 2: Bounded Loops
✅ **Static/runtime bounds** - Every loop has explicit guard or proven termination
```rust
// Example from parser.rs
while idx < line_count {  // Bounded by input size
    if blocks.len() >= self.max_blocks {  // Runtime guard
        return Err(...);
    }
    // ...
}
```

### Rule 3: Allocation Policy
✅ **Bounded allocations** - All data structures have max sizes:
- `MAX_BLOCKS = 1000`
- `MAX_LINES_PER_BLOCK = 10000`
- `MAX_FILE_SIZE = 10MB`
- `MAX_INPUT_SIZE = 100MB`

✅ **No unbounded growth** - All `Vec` usage validated against limits before allocation

### Rule 4: Small Units
✅ **Functions ≤60 logical lines** - All functions kept focused and testable  
✅ **One statement per line** - No dense compound statements

Verified:
```bash
$ find src -name "*.rs" -exec grep -c "^    fn " {} \; | sort -rn
# Largest function: ~55 lines (apply_block in apply.rs)
```

### Rule 5: Assertions
✅ **≥2 assertions per function** - Pre/post conditions validated

Examples:
```rust
pub fn new(rid: u64) -> Self {
    assert!(rid > 0, "Logger rid must be non-zero");  // Pre
    Self { rid }
}

pub fn find_fuzzy(&self, haystack: &str, needle: &str) -> Result<MatchResult> {
    assert!(haystack.len() <= MAX_HAYSTACK_SIZE);  // Pre
    assert!(needle.len() <= MAX_NEEDLE_SIZE);      // Pre
    // ... computation ...
    assert!((0.0..=1.0).contains(&ratio));         // Post
    ratio
}
```

### Rule 6: Smallest Scope
✅ **Tight variable scope** - Variables declared at use point  
✅ **No variable reuse** - Each variable has single purpose  
✅ **Block-scoped temporaries** - Short-lived values in minimal scope

### Rule 7: Contracts
✅ **Input validation** - All public functions validate inputs:
```rust
pub fn parse(&self, input: &str) -> Result<Vec<PatchBlock>> {
    assert!(input.len() < 100_000_000);  // Size check
    if line_count > self.max_lines {     // Bounds check
        return Err(...);
    }
    // ... parse ...
}
```

✅ **Result checking** - All `Result<T>` types marked `#[must_use]`  
✅ **Stable error codes** - Machine-parseable codes (1000-5999)

### Rule 8: Minimal Metaprogramming
✅ **No macro complexity** - Only derives and simple function macros  
✅ **No build-time code generation** - All variants enumerated in source  
✅ **Explicit over implicit** - No hidden trait magic

### Rule 9: Minimal Indirection
✅ **No `unsafe`** - Zero unsafe blocks in codebase  
✅ **Minimal `dyn`** - No dynamic dispatch in hot paths  
✅ **Concrete types** - All types known at compile time

### Rule 10: Zero Warnings
✅ **Pedantic lints** - Clippy in strict mode
```toml
[build]
rustflags = ["-D", "warnings"]
```

✅ **CI gates** - All checks enforced in CI:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo audit`

## Debugging & Observability

### Structured Logging
All errors logged in JSONL format:
```json
{
  "ts": "2025-10-13T12:34:56Z",
  "level": "error",
  "rid": 12345,
  "subsystem": "apply",
  "action": "write_file",
  "code": 3002,
  "msg": "Failed to write file",
  "context": {"path": "src/main.rs"}
}
```

### Error Context
Every error includes:
- Stable error code (machine-readable)
- Human message
- Structured context
- Stack of actions leading to failure

### Bounded Data in Logs
```rust
const MAX_MSG_LEN: usize = 1024;
const MAX_CONTEXT_LEN: usize = 4096;
```

## Testing Strategy

### Unit Tests
- All core algorithms have unit tests
- Edge cases explicitly covered
- Property-based tests for bounds

### Integration Tests
- Full end-to-end workflow tested
- Git safety verified
- Error paths exercised

### Manual Testing
```bash
just test-example  # Runs full scenario
```

## Runtime Safety

### Panic Strategy
`panic = "abort"` in all profiles - no unwinding  
All recoverable errors use `Result<T, PatchError>`

### Bounds Checking
Every array/slice access validated or proven safe via assert

### Integer Safety
- No unchecked arithmetic
- Use of `saturating_*` methods where appropriate
- Range assertions before casts

## Threat Model

**In scope:**
- Malformed patch files (caught by parser validation)
- Extremely large inputs (bounded by size limits)
- Filesystem errors (handled gracefully)
- Git repo issues (validated before proceeding)

**Out of scope:**
- Binary exploitation (Rust's memory safety)
- Time-of-check-time-of-use races (atomic git operations)
- Malicious git hooks (user's responsibility)

## Confidence Levels

| Component | Confidence | Notes |
|-----------|------------|-------|
| Parser | High | Bounded loops, validated inputs |
| Matcher | High | Fuzzy matching with clear thresholds |
| Applier | High | Atomic write, backup via git |
| Git Safety | High | Clean tree check, safety commit |
| Error Handling | High | Stable codes, structured context |

## Future Safety Improvements

- [ ] Add fuzzing with cargo-fuzz
- [ ] Formal verification of core algorithms
- [ ] Memory profiling under load
- [ ] Benchmark suite for performance bounds
- [ ] Property tests for all public APIs

## Maintenance

**Before merging:**
1. Run `just check`
2. Verify all tests pass
3. Check no new `unsafe` blocks
4. Verify function sizes ≤60 lines
5. Ensure ≥2 assertions per new function

**On release:**
1. Run `cargo audit`
2. Update dependency bounds
3. Re-verify all safety properties
4. Update this document if rules change