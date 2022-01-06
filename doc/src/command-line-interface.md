# Command Line Interface

PJSH comes with a small command line interface.

Available commands can be listed using the `--help` argument:

```pjsh
pjsh --help
```

### Start An Interactive Shell

An interactive shell can be started by calling `pjsh` without any arguments:


```pjsh
pjsh
```

### Execute A Script

Optionally, a script file can be passed as an argument. The script will be executed in a new non-interactive shell.

```pjsh
pjsh path/to/script.pjsh
```

Alternatively, commands can be piped to the `pjsh` binary:

```pjsh
echo "ls -lah" | pjsh
```

### Execute A Command

A command can be passed using the `-c` or `--command` option:

```pjsh
pjsh -c "ls -lah"
```

The command is executed in a new non-interactive shell.
