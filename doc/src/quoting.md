# Quoting

The most basic building block for all text input is a _word_. Every command name or argument alias is comprised of a single word.

Words can be combined to form commands.

Note that some utilities interpolate arguments after they are passed by the shell. This means that some strings could possibly be interpolated twice. The `$PS1` and `$PS2` prompts are some examples of such cases.

| Delimiter | Form         | Globbed? | Unindented? |
| :-------: | :----------- | :------: | :---------: |
|  _None_   | Literal      |   Yes    |     No      |
|    `'`    | Quoted       |    No    |     No      |
|    `"`    | Quoted       |    No    |     No      |
|   `'''`   | Multiline    |    No    |     Yes     |
|   `"""`   | Multiline    |    No    |     Yes     |
|  `` ` ``  | Interpolated |   Yes    |     No      |
|  ` ``` `  | Interpolated |   Yes    |     Yes     |
## Literals

Any normal, unquoted, text is parsed into whitespace-separated words. The amount of whitespace does not matter; multiple spaces will not result in empty words.

Some examples of literals:

- `word`
- `my_word`
- `1234`
- `/usr/local/bin`
- `~/.pjsh`

Literals are subject to _globbing_.

## Quoted Words

Words containing whitespace can be quoted using either single quotes `'...'` or double quotes `"..."`. There is no functional difference between the two variants, but it is recommended to use single quotes to surround content with double quotes and vice versa.

Some examples of quoted words:

- `"This is a word"`
- `'This is also a word'`
- `"can't be unquoted"`
- `'"everything" is a word'`

A backslash can be used to escape a character that would otherwise be considered a word-terminator:

- `"Illegal value \"arg\" found!"`
- `'Only escape \'terminators\' when required.'`

Quoted words are not subject to _globbing_.

## Multiline Words

In order to allow neat indentation to be used throughout script files, special multiline words surrounded by triple quotes (`'''` or `"""`) can be used.

Such words must span multiple lines, and cannot contain any non-whitespace characters on the first or last line.

Whitespace characters are removed to the same level as the first non-whitespace character within the multiline word.

Some examples of multiline words:

- ```
  echo """
    This is a message.
  """
  ```
  The same as `echo "This is a message."`
- ```
  echo '''
    Single quotes are functionally the same.
  '''
  ```
  The same as `echo 'Single quotes are functionally the same.'`

Multiline words are not subject to _globbing_.

## Interpolated Words

Special words containing whitespace and/or other values can be created using backticks (`` ` ``).

Interpolated words come in both the normal `` ` `` single line version, and the ` ``` ` multiline version.

The following values are interpolated within interpolated words:

- Environment variables: `$var`, `${var}`
- Subshells: `$(...)`

Interpolated words are subject to _globbing_.

## Special Sequences

The following sequences have a special meaning in quoted words:

| Sequence   | Description                                       |
| :--------- | :------------------------------------------------ |
| `\u{NNNN}` | Unicode char `NNNN` (hex).                        |
| `\e`       | ANSI escape. Equivalent to `\u{001b}`.            |
| `\\`       | Equivalent to `\` if required to avoid confusion. |
