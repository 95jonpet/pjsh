# Loops

The shell can execute a series of instructions for as long as a condition is considered `true`.

## While-loops

A _while-loop_ is the most basic form of loop.

```pjsh
while true {
  echo "This is run forever..."
}
```

```pjsh
# Continuously wait for a lock to be released.
while [[ -f .lock ]] {
  sleep 2
}
```

## For-in-loops

A _for-in-loop_ is a loop over a list of items.

```pjsh
# Iterate over a pre-defined list of items.
for word in [a b c] {
  echo $word
}
```

Lists can also be dynamically evaluated using ranges: 

```pjsh
# Iterate over a range of numeric values.
for i in 1..=10 {
  echo `i=$i`
}
```

## For-in-of-loops

A _for-in-of-loop_ is a more specialized form of the _for-in-loop_.
It allows iteration over well-defined elements of something.

```pjsh
# Iterate over characters.
for char in chars of "characters" {
  echo $char
}
```

```pjsh
# Iterate over words using a specific name.
for color in words of "red green blue" {
  echo $color
}
```

The following iteration constructs are supported:

| Syntax                | Description                                         |
| :-------------------- | :-------------------------------------------------- |
| `for x in chars of y` | Iterate `x` over characters in `y`.                 |
| `for x in lines of y` | Iterate `x` over lines in `y`.                      |
| `for x in words of y` | Iterate `x` over whitespace-separated words in `y`. |
