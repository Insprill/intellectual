# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.7.0] - 2024-02-16

### Added
- Support for aarch64.
- Support for Brotli compression.
- A new docker tag for development versions (`develop`).

### Fixed
- Album cards in artist descriptions not rendering correctly ([#23] by [@ButteredCats]).
- Images in artist descriptions not being proxied, causing them not to load.
- Artist/album/lyric links in artist descriptions pointing to Genius.
- Artist link in albums directing to an invalid page.
- Proxied images not sending a Content-Type header.
- Static assets not being stored in shared caches (e.g. Cloudflare).
- Pages not being stored in local (browser) caches.
- Version link in navbar not working.

### Changed
- Images are now optimized before being proxied (70-80% smaller).
- The Git hash is now included in versions that aren't built against a release tag.


## [0.6.0] - 2023-10-25

### Added
- A settings menu to change theme.
- Pre-built binary for musl libc.

### Fixed
- Long titles covering other UI elements ([#15] by [@Ftonans]).

### Changed
- Reduced binary size by 20%.

### Removed
- Pre-built macOS binary. 


## [0.5.1] - 2023-07-07

### Fixed
- Lyrics not showing for songs with no verse headers.


## [0.5.0] - 2023-06-01

### Added
- The ability to go to artists/albums via path.

### Removed
- The ability to go to artists/albums via ID.


## [0.4.0] - 2023-05-18

### Added
- Support for viewing albums.
- Support for TLS connections.
- Flag for the desired Keep-Alive timeout. The default has been increased to 15 seconds from 5.
- Hover animations to pagination buttons.

### Fixed
- The lyric page's "View on Genius" button taking the user to an invalid URL.

### Removed
- The requirement for a Genius API token.


## [0.3.1] - 2023-05-09

### Fixed
- The artist image taking up the entire background on small Safari viewports.
- The image URL on the artist page not being URL encoded.


## [0.3.0] - 2023-05-09

### Added
- A section for the top 5 songs on the artist page.
- More information to page titles.
- Error pages for 400, 404, and 500.
- Cache-Control headers to static resources.
- A description meta tag to the home page.
- Logging for internal server errors.

### Fixed
- Font scaling on smaller devices.
- Browsers not invalidating static assets between versions.
- Multiple panics from invalid requests/responses.
- The logo being hard to see in light mode.
- The lyric parser sometimes creating empty lines.
- The lyric parser creating new lines where annotations start/end.

### Changed
- The default address to `0.0.0.0`.


## [0.2.0] - 2022-11-20

### Added
- Paginated searches.
- Light/dark themes.
- An artist page to view information about an artist.
- A 'Search on Genius' button.
- More hover effects.
- Gzip compression for responses.
- A flag to set worker count.
- Security response headers.
- A web app manifest.

### Fixed
- Static files being served from disk instead of being bundled.
- Page view counts being posted in a separate batch.
- The genius response being relayed if an image couldn't be fetched.

### Changed
- Improved responsiveness on the lyrics page ([#6] by [@SeniorCluckers]).
- Updated fallback font to not be as jarring from Inter.
- Updated dependencies.


## [0.1.0] - 2022-10-13

- Initial release.


<!-- Users -->
[@ButteredCats]: https://github.com/ButteredCats
[@Ftonans]: https://github.com/Ftonans
[@SeniorCluckers]: https://github.com/SeniorCluckers

<!-- Pull Requests -->
[#23]: https://github.com/Insprill/intellectual/pull/23
[#15]: https://github.com/Insprill/intellectual/pull/15
[#6]: https://github.com/Insprill/intellectual/pull/6

<!-- Diffs -->
[Unreleased]: https://github.com/Insprill/intellectual/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/Insprill/intellectual/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Insprill/intellectual/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/Insprill/intellectual/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Insprill/intellectual/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Insprill/intellectual/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Insprill/intellectual/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Insprill/intellectual/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Insprill/intellectual/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Insprill/intellectual/releases/tag/v0.1.0
