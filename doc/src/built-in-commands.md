# Built-in Commands

PJSH comes with a few utilities that are designed to make the shell easier to use. These utilities are tightly coupled with the shell and can provide functionality that alters the shell itself.

| Built-in    | Description                                             |
| :---------- | :------------------------------------------------------ |
| alias       | Define shell aliases.                                   |
| cd          | Change working directory.                               |
| echo        | Print output to stdout.                                 |
| exit        | Exit the shell with a specific status code.             |
| false       | Always false in logic (exits with status `1`).          |
| interpolate | Interpolate arguments outside the current shell.        |
| pwd         | Print the current working directory to stdout.          |
| sleep       | Wait for a configurable amount of time.                 |
| source      | Execute a script in the current environment.            |
| true        | Always true in logic (exits with status `0`).           |
| type        | Print the type of a command (i.e. built-in or program). |
| unalias     | Remove an alias from the shell.                         |
| unset       | Remove variables from the shell's environment.          |
| which       | Find a program in `$PATH`.                              |
