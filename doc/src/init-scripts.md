# Init Scripts
Pjsh will automatically execute the following scripts:

| Order | Script path                     | Executed when                  |
| ----: | :------------------------------ | :----------------------------- |
|     1 | `~/.pjsh/init-always.pjsh`      | Starting a new shell.          |
|     2 | `~/.pjsh/init-interactive.pjsh` | Starting an interactive shell. |

Init scripts can be used to prepare the shell by:
  - defining common functions.
  - defining environment variables such as `PS1` and `PS2`.
  - setting common aliases.
