# Changelog

# 0.5.0:
- Added the ability to go to artists/albums via path.
- Removed the ability to go to artists/albums via ID.


# 0.4.0:
- Added support for viewing albums.
- Added support for TLS connections.
- Added flag for the desired Keep-Alive timeout. The default has been increased to 15 seconds from 5.
- Added hover animations to pagination buttons.
- Fixed the lyric page's "View on Genius" button taking the user to an invalid URL.
- Removed the requirement for a Genius API token.


# 0.3.1:
- Fixed the artist image taking up the entire background on small Safari viewports.
- Fixed the image URL on the artist page not being URL encoded.


## 0.3.0:
- Added a section for the top 5 songs on the artist page.
- Added more information to page titles.
- Added error pages for 400, 404, and 500.
- Added Cache-Control headers to static resources.
- Added a description meta tag to the home page.
- Added logging for internal server errors.
- Fixed font scaling on smaller devices.
- Fixed browsers not invalidating static assets between versions.
- Fixed multiple panics from invalid requests/responses.
- Fixed the logo being hard to see in light mode.
- Fixed the lyric parser sometimes creating empty lines.
- Fixed the lyric parser creating new lines where annotations start/end.
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
- Fixed the genius response being relayed if an image couldn't be fetched.


## 0.1.0:
- Initial release.


<!-- Users -->
[@SeniorCluckers]: https://github.com/SeniorCluckers

<!-- Pull Requests -->
[#6]: https://github.com/Insprill/intellectual/pull/6
