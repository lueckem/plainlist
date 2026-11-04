# plainlist

Manage music playlists in a plain text format!

## Format specification

- Each line contains one track (except comment lines starting with `#` and empty lines)
- Track format: `artist -- title` (including spaces around the separator `--`)
- Multiple artists separated by `; `, ` feat. `, or ` / ` (including spaces), for example
  - `artist1 feat. artist2; artist 3 -- track`
  - `AC/DC / artist2 -- track`
- The file extension is `.plainlist`

## Usage

### Resolve

Parses plainlists, reads your music library, resolves tracks against the library, and generates `.m3u` playlists.

**Examples:**
- Process single plainlist: `plainlist resolve --library=~/Music -o ~/Music/favorites.m3u favorites.plainlist`
- Process all plainlists in directory: `plainlist resolve --library=~/Music -o ~/Music/playlists ~/plainlists`

**Behavior:**
- Missing or ambiguous tracks are reported and omitted from output playlists
- Use `--portable` flag for relative paths in playlists

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

## Why use plainlists?
- simplicity: text files, simple format, hand-editable, git-trackable
- independent of music library
    - add tracks you don't own yet
    - resolve again when files change
- independent of apps/players
