# Apple Music to Navidrome

A tool which reads library information from an XML file generated by Apple Music and helps transfer some of that information to a Navidrome database.

## Features

### Metadata transfer

An updated version a given Navidrome database can be made with the following information written for a given user:

- Track playcount
- Album playcount
- Artist playcount
- Starred
- Track play date

A copy of the navidrome database is made and updated, so to see these updates you'll need to replace the existing database with the updated version.

Matching tracks between Apple Music and Navidrome is done by matching artist, album, title, track number, and disc number fields.
Any field which is not present in either database is ignored.
If a track in Apple Music has no match, or has multiple matches, information about the track is written to a file.

#### Notes

- Album playcount is inferred as the minimum playcount of tracks in the album (i.e. it assumes an album has been played only if every track has been listened to).
- Artist playcount is inferred as the sum of all the playcounts of all tracks assocaited with the artist.
- Starred is inferred from whether the track was loven or favourted in Apple Music.


### Playlist export

Playlists saved in Apple Music can be exported as m3u playlists.

The location of a track in the m3u playlist is taken from the Apple Music XML file, and so can be used to import the playlist to Navidrome, so long as both Apple Music and Navidrome use the same files.

### Apple Music XML to JSON

A JSON version of the Apple Music XML can be saved with some common metadata from the XML file (title, playcount, last played, etc.).

## How to use

- Build the `apple_navidrome` target.

- Copy `apple_navidrome` to a folder.

- Copy `an_config.toml` to the same folder or run the `apple_navidrome` target from the folder.
  On first run of `apple_navidrome` a default config file will be made with the same contents, but without the comments.

- Update `an_config.toml` file to match your setup.
  In particular, you'll likely want to update `navidrome_user` to match the username you want to update metadata for.

- Run `apple_navidrome` a second time.

## Caveats

As with most things there are some caveats.

- The Apple Music XML parser has only been tested on a single library (created by Apple Music version 1.4.6.32).
  The parser has been written to be flexible, and if your XML file differs you should (I hope) be able to extend the parser with some effort.

- The Navidrome import has only been tested on a single database (created by Navidrome version 0.53.3).
  As with the XML parser, it should be ok to extend this with some effort.

- Navidrome may fail to import tracks on some playlists.
  - There's not too much that can reasonably be done about this outside of Navidrome.
    The way `apple_navidrome` creates playlists is by:
    - Reading metadata stored in the Apple Music XML file.
    -  Each track has an internal id, and a playlist is a name and list of those internal ids.
    -  `apple_navidrome` then matches each internal id to the track in the Apple Music XML file and then writes out some metadata takes from the Apple Music XML to an m3u file.
    Notably, title and location are written to the m3u and other m3u readers, such as VLC, read the location data fine.
    So, my guess is there's some conflict how Apple Music stores location and how Navidrome reads location.
    (One option is to lookup the track in the Navidrome database and write the m3u location based on what Navidrome stores, but that's outside my definition of 'reasonable', for now…)

- There is no documentation, though some attempt has been made to report errors in a helpful way.
