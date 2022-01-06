# Configuration

All of the configuration for PJSH is stored in the configuration directory located at `~/.pjsh` (or `%userprofile%/.pjsh` on Windows).

The configuration is driven by scripts that are run upon starting the shell.

## Initialization Scripts
Pjsh automatically executes the following scripts:

| Order | Script path                     | Description                                  |
| ----: | :------------------------------ | :------------------------------------------- |
|     1 | `~/.pjsh/init-always.pjsh`      | Executed when starting a new shell.          |
|     2 | `~/.pjsh/init-interactive.pjsh` | Executed when starting an interactive shell. |

Init scripts can be used to prepare the shell by:
  - defining common functions.
  - defining environment variables such as `PS1` and `PS2`.
  - setting common aliases.

Note that there is no need to call `~/.pjsh/init-interactive.pjsh` from `~/.pjsh/init-always.pjsh`.
It should, in fact, be avoided.

### An example

The following `~/.pjsh/init-interactive.pjsh` defines two aliases and sets up a custom `PS1` prompt.

```pjsh
#!/bin/pjsh

alias ll = "ls -l"
alias ls = "ls -F --color=auto --show-control-chars"

# PS1 is interpolated on use.
PS1 := """
    ┌──($USERNAME)-[$PWD]
    └─\$\u{0020}
"""
```

A colored `PS1` prompt can be specified as follows:

```pjsh
PS1 := """
    \e[30;0m┌──(\e[1;36m$USERNAME\e[30;0m)-[\e[1;32m$PWD\e[30;0m]\e[0m
    \e[30;0m└─\$\u{0020}\e[0m
"""
```
