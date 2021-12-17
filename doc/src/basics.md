# The Basics
This chapter describes the basic features of `pjsh`.

## Variables
Variables can be defined using the `:=` operator.
```pjsh
my_var := my_value
```

Variables can later be referenced using the dollar, `$`, operator.
```pjsh
echo $my_var
```

Variables can also be used in interpolated strings.
```pjsh
echo $"Value: ${my_var}"
```

## Aliases
Aliases are simple substitutions that can make a shell easier to use.
For example, certain arguments may be supplied automatically for some common commands.
This is useful for printing colored output in interactive shells (among other things).

Use the `alias` command to print a list of currently defined aliases.

New aliases can be set using the following notation:
```pjsh
# Syntax:
alias command = substitution

# Example:
alias ls = "ls --color=auto"
```

Now, if the shell sees the command `ls`, it will instead execute `ls --color=auto`.

The `unalias` command can be used to remove aliases:
```pjsh
# Syntax:
unalias command

# Example:
unalias ls
```

## Pipelines
POSIX-style pipelines are supported:
```pjsh
# Single line:
ls | grep 'abc' | sort

# Multiple lines:
ls \
  | grep 'abc' \
  | sort
```

## Smart pipelines
A _smart pipeline_ is intruduced to increase readability across multiple lines.
A start symbol `->|` is used to denote the start of the pipeline, and a
semicolon, `;`, symbol is used to denote its end.
```pjsh
# Multiple lines:
->| ls
  | grep 'abc'
  | sort
  ;
```
