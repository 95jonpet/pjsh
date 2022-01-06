# Pipelines

Two or more processes can be connected using the pipe (`|`) operator to form pipelines, with each process having its standard output connected to the standard input of the next one.

Pipelines can be used to compose complex commands out of simpler ones. 

The following pipeline can be used to count the number of currently running processes (on Linux).

```pjsh
ps -e --no-headers | wc -l
```

A small explanation:

- `ps -e --no-headers` prints a list of all currently running processes with one process per line to standard output.
- `wc -l` prints the number of lines passed to standard input.

The `|` results in `wc -l` counting the number of processes as there is exactly one line of input for each process.

## Traditional Pipelines

Traditional pipelines end with a newline, meaning that any unwanted newlines must be escaped with a backslash (`\`).

```pjsh
# Single line
ls | grep 'abc' | sort
```
```pjsh
# Multiple lines
ls \
  | grep 'abc' \
  | sort
```

## Terminated Pipelines

An explicitly terminated pipeline variant can be used for complex pipelines that span multiple lines.

A start symbol `->|` is used to denote the pipeline's start, and a semicolon terminator (`;`) is used to denote its end.

```pjsh
# Multiple lines
->| ls
  | grep 'abc'
  | sort
  ;
```
