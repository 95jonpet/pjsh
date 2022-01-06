# Globbing

The shell performs filename expansion of literal and interpolated words that contain special wildcard patterns. This is known as _globbing_.

- A single word of input is replaced by zero or more words when globbing.
- Files starting with a dot (`.`) are typically considered hidden.

## Match-all Wildcard

The asterisk (`*`) symbol matches any number of characters including none. Only matches paths to existing files and directories.

- Matches are expanded in alphabetic order (A-Z).
- Files starting with a dot are not matched by `*`, but by `.*`.

| Example | Matches                   | Does not match       |
| :------ | :------------------------ | :------------------- |
| `*at`   | `at`, `flat`, `hat`       | `.at`, `atom`        |
| `at*`   | `at`, `atlas`, `atom.log` | `cat`                |
| `*at*`  | `at`, `sator`             | `a`, `t`             |
| `*.tmp` | `.tmp`, `log.tmp`         | `file.tmp.test`      |
| `*`     | `a`, `b`, `cat`           | `.app.log`, `.vimrc` |
| `.*`    | `.app.log`, `.vimrc`      | `tmp.log`            |


## Tilde

Any tilde (`~`) character at the start of a globbed word is replaced by the path to the current user's home directory. This is equivalent to the value of `$HOME`.

For example, the following command can be used to list all files under the `.pjsh` directory in the user's home directory.

```pjsh
ls ~/.pjsh
```

## Globbing Consequences

Note that globbing is not always desired. Consider the following example:

```pjsh
# Unwanted globbing in this command:
find / -name *.log

# Could be expanded to any of the following (or similar):
find / -name
find / -name a.log b.log
```

The shell will expand `*.log` prior to invoking the `find` command, meaning that it will never receive `*.log` as input, but rather all expanded file names from the current directory.

The `*.log` argument must thus be quoted in order to allow the `find` command to interpret the `*` symbol:

```pjsh
find / -name "*.log"
```
