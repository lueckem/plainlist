# plainlist

[![CI](https://github.com/lueckem/plainlist/actions/workflows/ci.yaml/badge.svg)](https://github.com/lueckem/plainlist/actions/workflows/ci.yaml)
[![Version info](https://img.shields.io/crates/v/plainlist.svg)](https://crates.io/crates/plainlist)

Manage your music playlists in a plain text format!

The `plainlist` cli tool resolves these "plainlists" against your local music library to create `.m3u` playlist files
that can then be read by any music player.
(The music files must have correct title and artist tags.)

### Why use plainlists?
- simplicity: text files, simple format, hand-editable, git-trackable
- independent of the music library
    - add tracks you don't own yet
    - resolve again when music files change
    - share plainlists with others
- independent of apps/players


## Format specification

- Each line contains one track (except comment lines starting with `#` and empty lines)
- Track format: `artist -- title` (including spaces around the separator `--`)
- Multiple artists separated by `; `, ` feat. `, or ` / ` (including spaces), for example
  - `artist1 feat. artist2; artist 3 -- title`
  - `AC/DC / artist2 -- title`
- The file extension is `.plainlist`

## Usage

### Resolve

Parses plainlists, reads your music library, resolves tracks against the library, and generates `.m3u` playlists.

**Examples:**
- Resolve single plainlist: `plainlist resolve --library=~/Music -o ~/Music/favorites.m3u favorites.plainlist`
- Resolve all plainlists in directory: `plainlist resolve --library=~/Music -o ~/Music/playlists ~/plainlists`

**Behavior:**
- Missing or ambiguous tracks are reported and omitted from output playlists
- Use the `--portable` flag for relative paths in playlists

### Extract

Extract plainlists from other formats.

**Examples:**
- M3U: `plainlist extract --from m3u -o favorites.plainlist favorites.m3u`
- Spotify export: `plainlist extract --from spotify -o ~/Music/playlists ~/Downloads/spotify_export.json`

**Supported formats:**
- `--from m3u`: M3U playlist files (.m3u, .m3u8)
    - Invalid entries in M3U files are skipped
- `--from spotify`: Spotify export JSON format
    - creates one plainlist per playlist in the export

### Append

Append tracks from the music library to an existing plainlist.

**Examples:**
- Filter by artist: `plainlist append --library ~/Music --artist "The Beatles" favorites.plainlist`
- Filter by album: `plainlist append --library ~/Music --album "Help!" favorites.plainlist`
