# Filtering

Variable-based values may be manipulated in _value pipelines_ using _filters_.

The syntax for value pipelines is as follows:

```pjsh
items = [2 1 4 3]
echo ${items | sort | join ", "}
```

## Word filters

The following filters can be applied to words:

| Filter    | Return type | Description                                          |
| :-------- | :---------- | :--------------------------------------------------- |
| `lower`   | Word        | Converts the input word to all lowercase characters. |
| `split c` | List        | Splits the input on all `c` characters.              |
| `upper`   | Word        | Converts the input word to all uppercase characters. |

## List filters

The following filters can be applied to lists:

| Filter    | Return type | Description                               |
| :-------- | :---------- | :---------------------------------------- |
| `index i` | Word        | Returns the `i`-th item in the input.     |
| `len`     | Word        | Returns the number of items in the input. |
| `reverse` | List        | Reverses the input.                       |
| `sort`    | List        | Sorts the input.                          |
| `unique`  | List        | Removes duplicate values from the input.  |
