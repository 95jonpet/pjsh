# Filtering

Variable-based values may be manipulated in _value pipelines_ using _filters_.

The syntax for value pipelines is as follows:

```pjsh
items := [2 1 4 3]
echo ${items | sort | join ", "}
```

## Filters

The following built-in filters are provided:

| Filter            | Input type | Return type   | Description                                                       |
| :---------------- | :--------- | :------------ | :---------------------------------------------------------------- |
| `first`           | List       | Word          | Returns the first item in a list.                                 |
| `join sep`        | List       | Word          | Joins a list using a word separator.                              |
| `last`            | List       | Word          | Returns the last item in a list.                                  |
| `len`             | List       | Word          | Returns the length of a list.                                     |
| `lines`           | Word       | List          | Splits a word into a list of lines (separated by `\n` or `\r\n`). |
| `lowercase`       | Word       | Word          | Converts all characters into lowercase.                           |
| `nth n`           | List       | Word          | Returns the `n`-th item in a list.                                |
| `replace from to` | Word, List | Same as input | Replaces a value in a list or word.                               |
| `reverse`         | List       | List          | Reverses a list.                                                  |
| `sort`            | List       | List          | Sorts a list.                                                     |
| `split sep`       | Word       | List          | Splits a word into a list using a word separator.                 |
| `ucfirst`         | Word       | Word          | Converts the first character into uppercase.                      |
| `unique`          | List       | List          | Removes duplicate items from a list.                              |
| `uppercase`       | Word       | Word          | Converts all characters into uppercase.                           |
| `words`           | Word       | List          | Returns a list of whitespace-separated words.                     |
