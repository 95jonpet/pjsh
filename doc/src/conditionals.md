# Conditionals

The shell supports basic boolean logic based on the exit codes of executed commands.

- Exit `0` (success) is considered `true`.
- Exit `!=0` (error) is considered `false`.

Commands can be conditionally executed based on the result of previous commands.

### Logical AND Operator (`&&`)

The logical _AND_ operator (`&&`) can be used to run a second command only if a first command is successful.

|    a    |    b    | a && b  |
| :-----: | :-----: | :-----: |
| `true`  | `true`  | `true`  |
| `true`  | `false` | `false` |
| `false` | `true`  | `false` |
| `false` | `false` | `false` |

A small example:

```pjsh
# Print the path of a resolved directory:
cd relative/path && echo pwd
```

### Logical OR Operator (`||`)

The logical _OR_ operator (`||`) can be used to run a second command only if a first command fails.

|    a    |    b    | a \|\| b |
| :-----: | :-----: | :------: |
| `true`  | `true`  |  `true`  |
| `true`  | `false` |  `true`  |
| `false` | `true`  |  `true`  |
| `false` | `false` | `false`  |

A small example:

```pjsh
# Always exit with code 0.
rm output.tmp || true
```

### If-statements

The shell supports more complex conditionals using _if-statements_. Such statements contain a body that is executed if a condition is met.

```pjsh
if <condition> {
  # Conditional code goes here.
}
```

An example:

```pjsh
if true {
  echo "This should be printed"
}
```

Finally, an optional _else_ clause can be executed if the condition is not met.

```pjsh
if false {
  echo "This should not be printed"
} else {
  echo "This should be printed"
}
```

Multiple _if_ and _else if_ clauses can be combined to cover multiple branches.

```pjsh
if false {
  echo "This should not be printed"
} else if false {
  echo "This should not be printed either"
} else {
  echo "This should be printed"
}
```

If-statements always exit with code `0` unless a command fails within the executed branch.

### Switch-statements

The shell supports value-matching conditionals using _switch-statements_.
Such statements contain a body that is executed if an input value is matched.

```pjsh
input := matching

switch $input {
  matching {
    echo "This should be printed"
  }
  multiple values {
    echo "This should not be printed"
  }
  `not ${input}` {
    echo "This should not be printed"
  }
  static {
    echo "This should not be printed"
  }
}
```

Note that all matchable words are interpolated by the shell prior to matching.

### Conditions

Compact conditions can be declared using the `[[ ... ]]` syntax.

| Expression           | Description                                    |
| :------------------- | :--------------------------------------------- |
| `[[ -e path ]]`      | True if `path` exists.                         |
| `[[ is-path path ]]` | True if `path` exists.                         |
| `[[ -f path ]]`      | True if `path` is a file.                      |
| `[[ is-file path ]]` | True if `path` is a file.                      |
| `[[ -d path ]]`      | True if `path` is a directory.                 |
| `[[ is-dir path ]]`  | True if `path` is a directory.                 |
| `[[ a != b ]]`       | True if the strings `a` and `b` are different. |
| `[[ a == b ]]`       | True if the strings `a` and `b` are equal.     |
| `[[ a = b ]]`        | True if the strings `a` and `b` are equal.     |
| `[[ -z string ]]`    | True if the string `string` is empty.          |
| `[[ -n string ]]`    | True if the string `string` is not empty.      |
| `[[ string ]]`       | True if the string `string` is not empty.      |

Furthermore, a condition can be inverted using the `!` symbol:

```pjsh
if [[ ! -e path ]] {
  echo "The path does not exist!"
}
```
