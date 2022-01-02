# Installation

_PJSH_ is built for _Linux_ and _Windows_. There is currently no build for _macOS_.

| Description          | Download link |
| :------------------- | :------------ |
| Windows installer    | TBA           |
| Linux (.deb) package | TBA           |
| Linux (.rpm) package | TBA           |
| Linux (.tar)         | TBA           |

## Installing From Source Code

Additionally, PJSH can be compiled from its source code. This is advanced and not recommended for most users.

Prerequisites:

- [Git](https://git-scm.com) must be installed.
- [Rust](https://www.rust-lang.org) must be installed.

Perform the following steps to compile and install the application from its source code:

1. Clone the source code.
    ```bash
    git clone https://github.com/95jonpet/pjsh.git
    ```
2. Compile the source code and install the resulting binary using `cargo`:
    ```bash
    cd pjsh
    cargo install
    ```

## Known Issues

The following issues are known to exist:

- Windows: Redirecting output from Git Bash/MSYS utilities to a file in append mode may truncate it. This is most likely an issue with the Git Bash/MSYS utilities rather than the shell.
