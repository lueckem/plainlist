# plainlist Agent Instructions

This repository is a Rust CLI tool for managing music playlists in plain text format.

## Build & Run

- Build with: `cargo build`
- Run with: `cargo run`

## Commands

### Resolve
Parses plainlists, resolves tracks against a music library, and generates `.m3u` playlists.

Example:
```
plainlist resolve --library=~/Music -o ~/Music/playlists ~/plainlists
```

Use `--portable` flag for relative paths in m3u playlists.

### Extract
Extracts plainlists from other formats.

Examples:
```
plainlist extract --from m3u -o favorite_songs.plainlist favorite_songs.m3u
plainlist extract --from spotify -o ~/Music/playlists ~/Downloads/spotify_export.json
```

## Format Specification

- Each line contains one track (except comment lines starting with `#` and empty lines)
- Track format: `artist -- title` (including spaces around `--`)
- Multiple artists separated by `; `, ` feat. `, or ` / ` (including spaces)
- Extension is `.plainlist`

## Project Structure

- Main CLI entrypoint: `src/main.rs`
- Commands: 
  - `resolve` in `src/resolve.rs`
  - `extract` in `src/extract.rs`
- Modules:
  - `library.rs`: Library scanning and track matching
  - `normalize.rs`: Track and artist string normalization for matching
  - `track.rs`: Track parsing and representation
  - `utils.rs`: Utility functions for file operations
- Dependencies: clap (for CLI), lofty (for audio metadata), rayon (for parallel processing)

## Key Files

- `README.md`: Project documentation
- `Cargo.toml`: Dependencies and project metadata
- `flake.nix`: Nix build configuration

## Testing

- Run tests with: `cargo test`
- There are unit tests
- No dedicated test files or integration tests (yet)

## Environment

- Uses Nix for development environment setup (see `flake.nix`)
- Requires Rust toolchain
