[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Docker Pulls][docker-pulls-shield]][docker-pulls-url]
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

### Clearnet
| URL                                        | Country | Cloudflare |
|--------------------------------------------|---------|------------|
| https://intellectual.insprill.net/         | 🇺🇸 US   | ✔️         |
| https://in.bloat.cat/                      | 🇷🇴 RO   |            |
| https://intellectual.catsarch.com/         | 🇺🇸 US   |            |
| https://intellectual.privacyfucking.rocks/ | 🇩🇪 DE   |            |

If there is a checkmark under "Cloudflare", that means the site
is proxied behind [Cloudflare](https://www.cloudflare.com/).  
This means they have the ability to monitor traffic between you and the server.

### Tor
| URL                                                                                | Country | Onion Of                  |
|------------------------------------------------------------------------------------|---------|---------------------------|
| http://intellectual.catsarchywsyuss6jdxlypsw5dc7owd5u5tr6bujxb7o6xw2hipqehyd.onion | 🇺🇸 US   | intellectual.catsarch.com |




<!-- DEPLOYMENT -->

## Deployment

### Deploying

#### Docker

The easiest way to host intellectual is via Docker, and the included Docker Compose file.
To create a new directory, download the `docker-compose.yml`, and cd into the directory, run the following command (Requires cURL 7.10.3 or newer)
```bash
curl https://raw.githubusercontent.com/Insprill/intellectual/master/docker-compose.yml --create-dirs -o intellectual/docker-compose.yml && cd intellectual
```
By default, it'll bind to `127.0.0.1:8080`.
Once you're satisfied with the container, you can start it with
```bash
docker compose up -d
```

#### Native

If you don't want to use Docker, you can download the latest [stable](https://github.com/Insprill/intellectual/releases) or [nightly](https://nightly.link/Insprill/intellectual/workflows/rust/master) build from GitHub actions. Make sure to choose the version for your target operating system.

Append the `-h` flag when running to see all available arguments.

### TLS

Intellectual supports TLS connections natively using [rustls][rustls-repo].
To enable TLS, provide the `--tls` flag, followed by `--tls-key-file` and `--tls-cert-file` pointing to their respective files on disk.




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
- [x] Show artists' work on their page
- [x] Improve responsiveness
- [x] View Albums
- [x] Theme support
- [ ] Annotation support
- [ ] More search types
  - By lyrics
  - For artists
  - For albums
- [ ] Better accessibility
- [ ] Support for more lyric sources

Contributions are what make the open-source community such an amazing place to learn, inspire, and create.  
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
[docker-pulls-shield]: https://img.shields.io/docker/pulls/insprill/intellectual?style=for-the-badge
[docker-pulls-url]: https://hub.docker.com/r/insprill/intellectual
[rustls-repo]: https://github.com/rustls/rustls
