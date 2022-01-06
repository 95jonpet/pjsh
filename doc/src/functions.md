# Functions

Reusable functions can be defined using the `fn` keyword.

```pjsh
fn print_greeting() {
  echo "Hello, World!"
}
```

The above function can be executed by using its name as a command:

```pjsh
print_greeting
```

Additionally, functions may accept arguments:

```pjsh
# Create a directory and navigate to it.
fn md(path) {
  mkdir -p $path && cd $path
}

# Usage:
md new-directory
```
