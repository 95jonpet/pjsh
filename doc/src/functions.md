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

## Named Arguments

Additionally, functions may accept named arguments:

```pjsh
# Create a directory and navigate to it.
fn md(path) {
  mkdir -p $path && cd $path
}

# Usage:
md new-directory
```

Multiple named arguments should be separated by whitespace:

```pjsh
fn my_function(arg1 arg2) {
  # ...
}
```

## Positional Arguments

Unbound arguments are words that cannot be mapped to a named argument in the function definition. Such arguments can be referenced using their positional index (starting with 1).

```pjsh
# Does not accept any named arguments.
# Positional ID can be used to reference arguments.
fn my_function() {
  echo $1 $2
}

my_function positional1 positional2
```
