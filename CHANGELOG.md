# Changelog

## 0.x.x:
- Added a section for the top 5 songs on the artist page.
- Added more information to page titles.
- Added error pages for 400, 404, and 500.
- Added Cache-Control headers to static resources.
- Added a description meta tag to the home page.
- Added logging for internal server errors.
- Fixed font scaling on smaller devices.
- Fixed browsers not invalidating static assets between Intellectual versions.
- Fixed multiple panics from invalid requests/responses.
- Fixed the Intellectual logo being hard to see in light mode.
- Fixed the lyric parser sometimes returning empty lines.
- Changed default address to `0.0.0.0`.


## 0.2.0:
- Added paginated searches.
- Added light/dark themes.
- Added an artist page to view information about an artist.
- Added a 'Search on Genius' button.
- Added more hover effects.
- Added gzip compression for responses.
- Added a flag to set worker count.
- Added security response headers.
- Added a web app manifest.
- Improved responsiveness on the lyrics page ([#6] by [@SeniorCluckers]).
- Updated fallback font to not be as jarring from Inter.
- Updated dependencies.
- Fixed static files being served from disk instead of being bundled.
- Fixed page view counts being posted in a separate batch.
- Fixed genius response being relayed if an image couldn't be fetched.


## 0.1.0:
- Initial release.


<!-- Users -->
[@SeniorCluckers]: https://github.com/SeniorCluckers

<!-- Pull Requests -->
[#6]: https://github.com/Insprill/intellectual/pull/6
