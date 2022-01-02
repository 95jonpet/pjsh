# Command Resolution

All words are expanded prior to resolving command names:

1. Words are interpolated.
2. Aliases are expanded.
3. Globs are expanded.

Command names are then resolved in the following order using the first word from the expanded input:

1. Attempt to use a built-in command with the requested name.
2. Attempt to use the name as a path to a program.
3. Search for a program with the requested name in the path using:
   - The paths in the `$PATH` variable.
   - The file extensions in the `$PATHEXT` variable.

An execution error is returned if the command name cannot be resolved to a program.
