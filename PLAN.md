# Implementation Plan

## 1. Add documentation comments to public APIs

### Problem
Public functions and structs lack documentation comments explaining their purpose and usage.

### Solution
Add comprehensive documentation using Rust doc comments:

```rust
/// Resolve plainlist tracks against library and generate M3U playlist
/// 
/// This function scans the music library, matches tracks from the plainlist,
/// and creates an M3U playlist with resolved file paths.
///
/// # Arguments
/// * `args` - Resolution arguments including input/output paths and library location
///
/// # Returns
/// * `anyhow::Result<()>` - Success or detailed error information
pub fn process_resolve(args: ResolveArgs) -> anyhow::Result<()> {
    // implementation
}
```

### Steps
1. Add documentation to all public structs in `src/resolve.rs`
2. Add documentation to all public functions in `src/extract.rs`  
3. Add documentation to all public functions in `src/append.rs`  
4. Add documentation to all public structs and functions in `src/library.rs`
5. Add documentation to all public structs and functions in `src/track.rs`
6. Add documentation to all public structs and functions in `src/utils.rs`
7. Add module-level documentation where missing

## 2. Add comprehensive unit tests for core logic

### Problem
While there are some tests in `src/track.rs`, core library matching logic lacks comprehensive testing.

### Solution
Add extensive unit tests for:
- Library query logic with various artist combinations
- Normalization functions with edge cases  
- Error handling paths in track parsing and resolution

### Steps
1. Add tests to `src/library.rs` for different query scenarios (found, not found, ambiguous, etc.)
2. Add tests to `src/normalize.rs` for edge cases in normalization
3. Add integration tests for full resolve flow
4. Add test coverage for error paths in track parsing

## 3. Improve error handling consistency

### Problem
Inconsistent use of error handling patterns throughout the codebase.

### Solution
Standardize on using `anyhow::Context` for meaningful error messages and `?` operator for propagation.

### Steps
1. Review all functions that return `anyhow::Result` and ensure consistent context usage
2. Update error messages to be more descriptive where appropriate
3. Ensure all file I/O operations have proper error handling with context
