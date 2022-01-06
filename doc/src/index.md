# Introduction

_PJSH_ is a shell that aims to make the shell easier and more predictable to use. The syntax is mostly kept from the [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html), but legacy implementation details are removed in order to increase the readability and usability of shell scripts.

PJSH does not require heavy quoting around arguments as splitting is never done implicitly. If it looks like a word in code, it is also a word after expanding variables.

There is also support for multiline strings in which leading whitespace is trimmed in a sensible manner. This means that text indentation is not ruined in the same way as with the [heredoc](https://en.wikipedia.org/wiki/Here_document) seen in POSIX shells.

PJSH is designed to satisfy the following requirements:

- Command execution is predictable.
- Shell scripts are readable.
- The shell is not locked to a specific operating system.
