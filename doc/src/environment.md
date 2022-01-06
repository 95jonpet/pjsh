# Environment

The shell maintains a set of environment variables that can be used by the shell. Additionally, environment variables are inherited by the commands that the shell spawns.

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
echo `Value: ${my_var}`
```

## Scopes

By default, variables are only visible to the shell itself. In order to use variables outside the shell, they must be _exported_.

```pjsh
# Make $user known to the shell.
user := "Shell User"

# The following works as the shell evaluates $user.
echo $user

# New commands are not aware of $user.
my_command

# Export $user to the environment.
export user

# New commands are now aware of $user.
# Previously spawned commands remain unaffected.
my_command
```

## Subshells

Commands can be run in a new context by creating a _subshell_ using parentheses (i.e. `(command arg1 arg2)` or `$(command arg1 arg2)` in its interpolated form).

Subshells inherit a copy of the parent shell's environment at the time of creation. Further changes to the parent shell will not affect existing subshells.

Interpolating a subshell will result in a single word consisting of the output from the subshell's standard output file descriptor. The parent shell waits for the subshell to complete when interpolating it.
