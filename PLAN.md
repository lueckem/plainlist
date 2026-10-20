# Implementation Plan

## 1. Replace unwrap() calls with proper error handling in resolve.rs

### Problem
In `src/resolve.rs`, lines 87-88 use `unwrap()` on `o.parent()` which can cause panic if path has no parent.

### Solution
Replace with proper error handling:

```rust
// Before:
let o_dir = o.parent().unwrap();

// After:
let o_dir = o.parent()
    .context("Failed to get output directory")?;
```

### Steps
1. Locate lines 87-88 in `src/resolve.rs`
2. Replace `o.parent().unwrap()` with proper error handling using `?` operator
3. Update function signature to return `anyhow::Result<()>`
4. Test that the change doesn't break existing functionality

## 2. Add documentation comments to public APIs

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
3. Add documentation to all public structs and functions in `src/library.rs`
4. Add documentation to all public structs and functions in `src/track.rs`
5. Add documentation to all public structs and functions in `src/utils.rs`
6. Add module-level documentation where missing

## 3. Implement or remove TODO comments for append command

### Problem
TODO comment in `src/main.rs` references an unimplemented append command.

### Solution
Either implement the feature or remove the TODO if it's not planned.

### Steps
1. Remove the TODO comment from `src/main.rs` since we're keeping focus on current functionality
2. Alternatively, if implementing, create the append command structure

## 4. Add comprehensive unit tests for core logic

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

## 5. Improve error handling consistency

### Problem
Inconsistent use of error handling patterns throughout the codebase.

### Solution
Standardize on using `anyhow::Context` for meaningful error messages and `?` operator for propagation.

### Steps
1. Review all functions that return `anyhow::Result` and ensure consistent context usage
2. Update error messages to be more descriptive where appropriate
3. Ensure all file I/O operations have proper error handling with context
