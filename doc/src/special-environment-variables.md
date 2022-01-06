# Special Environment Variables
This chapter describes all special environment variables that are used by the shell.

### $?

The value of `$?` contains the exit code of the last command.

### $HOME
Absolute path to the user's home directory. This value is automatically set when creating a new shell.

### $OLDPWD
Absolute path to the previous working directory. Managed by the `cd` builtin.

### $PATH
Contains paths to search for programs in.

Values are colon-separated on most systems, with the exception of Windows using semicolon-separated values.

### $PATHEXT
Extensions that are allowed when searching for programs using `$PATH`.

If unset, or empty, only programs without a file extension are matched.

Values are colon-separated on most systems, with the exception of Windows using semicolon-separated values.

### $PS1
Prompt to use when requesting a new line of input.

This value is interpolated by the shell and printed to stderr.

### $PS2
Prompt to use when requesting an additional line of input while processing an incomplete logical line of input.

This value is interpolated by the shell and printed to stderr.

### $PWD
Absolute path to the current working directory. Managed by the `cd` builtin.
