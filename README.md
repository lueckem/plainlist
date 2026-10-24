# plainlist

Manage music playlists in a plain format!

## Format specification

- each line contains one track
    - except comment lines which start with `#`
    - except empty lines
- a track has the format `artist -- title`
- the separator between artist and title is ` -- `, including the spaces
- multiple artists are separated with `; ` or ` feat. `  or  ` / `,  again including the spaces
    - example: artist1 feat. artist2; artist 3 -- track
    - example: AC/DC / artist2 -- track
- the extension is `.plainlist`

## Usage

### Resolve

Parses one or multiple plainlists, reads your music library, resolves the tracks in the plainlist against the music you have, and finally generates a `.m3u` playlist that can be read by any music player.

Example: `plainlist resolve --library=~/Music -o ~/Music/playlists ~/plainlists` reads all plainlists in the folder `~/plainlists` and stores the created playlists in the folder `~/Music/playlists`.

Use the `--portable` flag to use relative paths in the m3u playlists instead of absolute paths.

### Extract

Extract a plainlist from a different format.

Example: `plainlist extract --from m3u -o favorites.plainlist favorites.m3u` reads the files in the m3u playlist and creates the matching plainlist.

Supported source formats:
- `--from m3u`
- `--from spotify` (spotify export JSON format)


### Append

Append tracks from the library to an existing plainlist.

Example: `plainlist append --library ~/Music --artist "The Beatles" --album "Help!" favorites.plainlist` finds all tracks with matching artist and album from the library and appends them to the plainlist.


## Why use plainlists?
- simplicity
    - just text files
    - simple format
    - edit them by hand
    - track them via git
- independent of your music library
    - add tracks you do not own yet
    - plainlists do not break when your music files change; simply resolve again
    - send your plainlist to a friend; it will resolve against his library and tell him which tracks are missing
- independent of any apps, music players, etc.
    - you own your plainlists
