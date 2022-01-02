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
