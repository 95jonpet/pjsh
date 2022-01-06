# Commands

The shell interprets each line of input as a command. Every command consists of one or more words: a program and optional arguments.

```pjsh
ls
```
```pjsh
cat file.txt
```
```pjsh
find . -iname "*.config"
```

## Input/Output

Each program is given three file descriptors:

|    # | Name   | Description                         |
| ---: | :----- | :---------------------------------- |
|    0 | stdin  | Standard input to read input from.  |
|    1 | stdout | Standard output to print output to. |
|    2 | stderr | Standard error to print errors to.  |

Standard output and standard error are typically printed to the terminal if no redirection is performed.

## Exit Codes

Every command returns an _exit code_ upon completion.

Success is typically represented with the exit code `0`. Errors are typically represented with exit codes `!= 0`.

Note that exit codes are typically in the `[0, 255]` range (modulo 256). Some operating systems do not have this restriction.

The exit code from the last executed command can be accessed using `$?`.

```pjsh
mkdir my-dir

# Print the exit code of "mkdir my-dir".
echo $?
```
