[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<br />
<p align="center">
  <a href="https://github.com/95jonpet/pjsh">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">pjsh</h3>

  <p align="center">
    A small, not yet POSIX, shell.
    <br />
    <a href="https://github.com/95jonpet/pjsh"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/95jonpet/pjsh">View Demo</a>
    ·
    <a href="https://github.com/95jonpet/pjsh/issues">Report Bug</a>
    ·
    <a href="https://github.com/95jonpet/pjsh/issues">Request Feature</a>
  </p>
</p>

## About The Project

[![Screen Shot][product-screenshot]](https://peterjonsson.se)

The shell _pjsh_ is designed with the following goals:

- The shell command language should follow the [POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html).
- Errors should be clearly indicated to the user.
- Performance should not be a hindrance for the user.

There are two modes of operation:
- _Interactive_: Input is read line by line from stdin.
- _Non-interactive_: Input is passed as an argument or read from a file.

Input size is assumed to be relatively short (i.e. at most a few thousand lines).

```
         +---------------- Shell ----------------+
Input -> | Cursor -> Lexer -> Parser -> Executor | -> Output
         +---------------------------------------+
```

- **Input**: Typically stdin, a string argument, or a file.
- **Cursor**: Provides a `char` stream and keeps track of the current position.
- **Lexer**: Converts a `char` stream to a `Token` stream.
- **Parser**: Converts a `Token` stream to an `AST` (abstract syntax tree).
- **Executor**: Executes an `AST`.
- **Output**: Typically stdout.

### Built With

This project is bult using:

* [Rust](https://www.rust-lang.org/)

## Getting Started

To get a local copy up and running follow these simple example steps.

### Prerequisites

Download and install the latest version of the [Rust programming language](https://www.rust-lang.org/).

### Installation

* Compile the project from scratch:
  ```bash
  cargo build --release
  ```
* Install the compiled binary `target/release/pjsh`:
  ```bash
  cp target/release/pjsh /usr/local/bin
  chmod +x /usr/local/bin/pjsh
  ```

## Usage

See the [POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html).
Note that not all features are implemented yet.

## Roadmap

See the [open issues](https://github.com/95jonpet/pjsh/issues) for a list of proposed features (and known issues).

## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/feature-name`)
3. Commit your Changes (`git commit -m 'Add feature-name'`)
4. Push to the Branch (`git push origin feature/feature-name`)
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE.md` for more information.

## Contact

Project Link: [https://github.com/95jonpet/pjsh](https://github.com/95jonpet/pjsh)

[contributors-shield]: https://img.shields.io/github/contributors/95jonpet/pjsh.svg?style=for-the-badge
[contributors-url]: https://github.com/95jonpet/pjsh/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/95jonpet/pjsh.svg?style=for-the-badge
[forks-url]: https://github.com/95jonpet/pjsh/network/members
[stars-shield]: https://img.shields.io/github/stars/95jonpet/pjsh.svg?style=for-the-badge
[stars-url]: https://github.com/95jonpet/pjsh/stargazers
[issues-shield]: https://img.shields.io/github/issues/95jonpet/pjsh.svg?style=for-the-badge
[issues-url]: https://github.com/95jonpet/pjsh/issues
[license-shield]: https://img.shields.io/github/license/95jonpet/pjsh.svg?style=for-the-badge
[license-url]: https://github.com/95jonpet/pjsh/blob/main/LICENSE.md
[product-screenshot]: images/screenshot.png
