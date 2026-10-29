# Implementation Plan

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

Describe the different CLI commands in more detail in the `--help`.

## 4. Add `stats` command

Print statistics about a plainlist. Example: `plainlist stats songs.plainlist`.

The stats should include:
- number of songs
- number of artists
- top 10 artists sorted by number of tracks in the plainlist
- if and how many duplicate tracks (if `--verbose`, then print all duplicate tracks)

Optionally, a local music library can also be supplied via `--library`, which unlocks further stats:
- number of missing tracks (if `--verbose`, then print all missing tracks)
- total running time

## 5. Add `strip` command

Strips all songs from a plainlist that are not in the local library and store as a new plainlist.

Example: `plainlist strip --library=~/Music -o songs.plainlist stripped_songs.plainlist`
