<!-- PROJECT SHIELDS -->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][mit-license-url]
[![Apache-2.0 License][license-shield]][apache-2.0-license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/MordragT/hua">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Hua - Package Manager</h3>

  <p align="center">
    A simple package manager mixing traditional and next-gen concepts
    <br />
    <a href="https://mordragt.github.io/hua/hua-core/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/MordragT/hua/issues">Report Bug</a>
    ·
    <a href="https://github.com/MordragT/hua/issues">Request Feature</a>
  </p>
</div>


<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

![Screenshot][product-screenshot]

There are already many great package managers out there; however, most of them install all packages into the global directories, which makes it hard to roll back to a previous state if something went wrong.
Therefor usually you have to manually chroot into youre installation and resolve the issues or let youre filesystem take snapshots so that you can rollback to a previous state.
In my opinion this is not the task of your filesystem but of your package manager. Next generation package managers like nix prevent those failures by the concept of generations.
Whenever you install, remove or modify a package, a new generations is created, so that you can switch to another generation at any moment. Hua borrows this feature, but retains FHS compatible, so that you
do not have to patch elf files and downloaded applications run just out of the box if you have all the necessary dependencies installed.

<p align="right">(<a href="#top">back to top</a>)</p>


### Built With

- [Rust](https://www.rust-lang.org/)
- [Roc](https://www.roc-lang.org/)
- And many other great dependencies you can find in the `Cargo.toml`

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Development

We use the Nix package manager to create a reproducible development environment.
So to start developing, make sure you installed the Nix package manager and
that the nix flakes feature is enabled.

Then just run:

```sh
nix develop
```

### Installation

Currently there is no way to install hua, but there will be a installation for Nix users soon.

<p align="right">(<a href="#top">back to top</a>)</p>


<!-- USAGE EXAMPLES -->
## Usage

### Add a package

```sh
hua add <name>
```

### Remove a package

```sh
hua remove <name>
```

### Generations

#### List

```sh
hua generations list
```

#### Remove

```sh
hua generations remove <id>
```

### Store

#### Collect unsued packages and delete them

```sh
hua store collect-garbage
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

- [] Improve test coverage
- [] Improve error messages
- [] Create a package format with Roc
- [] File permissions

See the [open issues](https://github.com/MordragT/hua/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#top">back to top</a>)</p>


<!-- LICENSE -->
## License

Dual licensed under either MIT or Apache-2.0 at your choice

<p align="right">(<a href="#top">back to top</a>)</p>


<!-- CONTACT -->
## Contact

Thomas Wehmöller - [@MordragT](https://twitter.com/MordragT)

Project Link: [https://github.com/MordragT/hua](https://github.com/MordragT/hua)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

These are some great projects that helped me create hua

- [NixOS](https://nixos.org/)
- [Img Shields](https://shields.io)
- [GitHub Pages](https://pages.github.com)
- [Readme template](https://github.com/othneildrew/Best-README-Template)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/MordragT/hua.svg?style=for-the-badge
[contributors-url]: https://github.com/MordragT/hua/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/MordragT/hua.svg?style=for-the-badge
[forks-url]: https://github.com/MordragT/hua/network/members
[stars-shield]: https://img.shields.io/github/stars/MordragT/hua.svg?style=for-the-badge
[stars-url]: https://github.com/MordragT/hua/stargazers
[issues-shield]: https://img.shields.io/github/issues/MordragT/hua.svg?style=for-the-badge
[issues-url]: https://github.com/MordragT/hua/issues
[license-shield]: https://img.shields.io/github/license/MordragT/hua.svg?style=for-the-badge
[mit-license-url]: http://opensource.org/licenses/MIT
[apache-2.0-license-url]: http://www.apache.org/licenses/LICENSE-2.0
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/wehmoeller
[product-screenshot]: images/screenshot.png