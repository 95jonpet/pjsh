# Standard environment variables
This chapter describes all environment variables that are used by `pjsh`.

### HOME
Absolute path to the user's home directory. This value is automaticaly set when creating a new shell.

### OLDPWD
Absolute path to the previous working directory. Managed by the `cd` builtin.

### PATH
Contains paths to search for programs in.

Values are colon-separated on most systems, with the exception of Windows using semicolon-separated values.

### PATHEXT
Extensions that are allowed when searching for programs using `$PATH`.

If unset, or empty, only programs without a file extension are matched.

Values are colon-separated on most systems, with the exception of Windows using semicolon-separated values.

### PS1
Prompt to use when requesting a new line of input. Printed to stderr by `pjsh`.

### PS2
Prompt to use when requesting an additional line of input while processing an incompleted logical line of input. Printed to stderr by `pjsh`.

### PWD
Absolute path to the current working directory. Managed by the `cd` builtin.
