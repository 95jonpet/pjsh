# Aliasing

Aliases are simple substitutions that can make a shell easier to use.
For example, certain arguments may be supplied automatically for some common commands.
This is useful for printing colored output in interactive shells (among other things).

Use the `alias` command to print a list of currently defined aliases.

## Assigning Aliases

New aliases can be set using the following notation:
```pjsh
# Syntax:
alias command = substitution

# Example:
alias ls = "ls --color=auto"
```

Now, if the shell sees the command `ls`, it will instead execute the command `ls --color=auto`.

Aliases are resolved recursively until the first word is no longer an alias or until the first word has already been used in the alias resolution. Thus, the above example will not result in an infinite loop.

## Removing aliases

The `unalias` command can be used to remove aliases:
```pjsh
# Syntax:
unalias command

# Example:
unalias ls
```
