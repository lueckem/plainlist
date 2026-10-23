# Implementation Plan

## 1. Add comprehensive unit tests for core logic

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

## 2. Improve error handling consistency

### Problem
Inconsistent use of error handling patterns throughout the codebase.

### Solution
Standardize on using `anyhow::Context` for meaningful error messages and `?` operator for propagation.

### Steps
1. Review all functions that return `anyhow::Result` and ensure consistent context usage
2. Update error messages to be more descriptive where appropriate
3. Ensure all file I/O operations have proper error handling with context

## 3. Improve CLI help

Describe the different CLI in more detail in the `--help`.
