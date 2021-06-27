# pjsh

A small, not yet POSIX, shell.

## Goals

_pjsh_ is designed with the following goals:

- The shell command language should follow the [POSIX specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html).
- Errors should be clearly indicated to the user.
- Performance should not be a hindrance for the user.

## Architecture

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
