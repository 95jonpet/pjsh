# Redirection

When executing a command, its input and output may redirected to or from file handles. Thus, the contents of a file may be used as input for a command. Likewise, file can be created from the output of a command.

| Syntax     | Description                                           |
| :--------- | :---------------------------------------------------- |
| `> file`   | Write standard output to `file` (truncated).          |
| `>> file`  | Append standard output to `file`.                     |
| `< file`   | Read standard input from `file`.                      |
| `n> file`  | Write from file descriptor `n` to `file` (truncated). |
| `n>> file` | Append file descriptor `n` to `file`.                 |
| `n< file`  | Read file descriptor `n` from `file`.                 |
| `x>&y`     | Redirect file descriptor `x` to file descriptor `y`.  |

## Process Substitution

Another type of redirection is _process substitution_, which redirects output from a command to a file, substituting the expression to that file's path.

```pjsh
cat <(ls)  # Prints the contents of ls from a file.
```

Of course, the path itself can be printed using `echo`:

```pjsh
# Prints /tmp/pjsh_IDENTIFIER_stdout.
echo <(ls)
```
