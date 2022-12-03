[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![AGPLv3 License][license-shield]][license-url]




<!-- PROJECT LOGO -->
<br />
<div align="center">
  <h1>Intellectual</h1>
  <p>
    Alternate frontend for <a href="https://genius.com/">Genius</a> focused on privacy and simplicity 
    <br />
    <br />
    <a href="https://github.com/Insprill/intellectual/issues">Report Bug</a>
    ·
    <a href="https://github.com/Insprill/intellectual/issues">Request Feature</a>
  </p>
</div>




<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about-the-project">About The Project</a></li>
    <li><a href="#instances">Instances</a></li>
    <li><a href="#deployment">Deployment</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>




<!-- ABOUT THE PROJECT -->

## About The Project

Intellectual is an alternative frontend for [Genius](https://genius.com/) focused on privacy and simplicity.  
Written in Rust, Intellectual is incredibly lightweight.
Not requiring JavaScript and proxying all requests through the server, including images.

Intellectual is still very early in development and is lacking many features.  
Check out the [roadmap](#roadmap) for what features will be coming next!




<!-- Instances -->

## Instances

Want your own instance listed here? Open an issue for it!  
Not sure how to host your own instance? View the [deployment](#deployment) instructions.

| URL                               | Country | Cloudflare |
|-----------------------------------|---------|------------|
| https://intellectual.insprill.net | 🇺🇸 US | ✔️         |

If there is a checkmark under "Cloudflare", that means the site
is proxied behind [Cloudflare](https://www.cloudflare.com/).  
This means they have the ability to monitor traffic between you and the server.




<!-- DEPLOYMENT -->

## Deployment

### Getting an API key

To host intellectual, you'll need an API key from Genius.
Firstly, create a [new API client on Genius](https://genius.com/api-clients/new). You will need to have an account in order to do this.
The Icon URL and Redirect URL do not need to be set.  
Under "Client Access Token", click "Generate Access Token", and copy the token provided.
This will need to be passed in via the `GENIUS_AUTH_TOKEN` environment variable.

### Docker

The easiest way to host intellectual is via Docker.  
First, clone this repo and cd into the directory.
```
git clone https://github.com/Insprill/intellectual
cd intellectual
```
Next, open the `docker-compose.yml`, find `GENIUS_AUTH_TOKEN: "token"` and replace `token` with the token you copied above.  
Then you can start the container.
```
docker compose up -d
```

### Native

If you don't want to use Docker, you can download the latest [stable](https://github.com/Insprill/intellectual/releases) or [nightly](https://nightly.link/Insprill/intellectual/workflows/rust/master) build from GitHub actions. Make sure to choose the version for your target operating system.

Before starting, make sure to set the `GENIUS_AUTH_TOKEN` environment variable to the token you copied above.
Now you can start Intellectual. Append `-h` when running it to see all available arguments.




<!-- ROADMAP -->

## Roadmap

- [x] Search for songs
- [x] View lyrics
- [x] More song info on the lryics page
  - Song name
  - Song/album image
  - Album name
  - Artist
  - Release date
- [x] View artist info
- [x] Paginated searches
- [x] More robust error handling
- [ ] Theme support
- [ ] Improve responsiveness
- [ ] Annotation support
- [ ] More search types
  - By lyrics
  - For artists
  - For albums
- [ ] Show artists' work on their page
- [ ] Better accessibility
- [ ] Support for more lyric sources

Contributions are what make the open source community such an amazing place to learn, inspire, and create.  
Any contributions you make are **greatly appreciated**!  
If you're new to contributing to open-source projects,
you can follow [this](https://docs.github.com/en/get-started/quickstart/contributing-to-projects) guide to get up-to-speed.




<!-- LICENSE -->

## License

Distributed under the GNU Affero General Public License v3.0.  
See [LICENSE][license-url] for more information.




<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/Insprill/intellectual.svg?style=for-the-badge
[contributors-url]: https://github.com/Insprill/intellectual/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Insprill/intellectual.svg?style=for-the-badge
[forks-url]: https://github.com/Insprill/intellectual/network/members
[stars-shield]: https://img.shields.io/github/stars/Insprill/intellectual.svg?style=for-the-badge
[stars-url]: https://github.com/Insprill/intellectual/stargazers
[issues-shield]: https://img.shields.io/github/issues/Insprill/intellectual.svg?style=for-the-badge
[issues-url]: https://github.com/Insprill/intellectual/issues
[license-shield]: https://img.shields.io/github/license/Insprill/intellectual.svg?style=for-the-badge
[license-url]: https://github.com/Insprill/intellectual/blob/master/LICENSE
