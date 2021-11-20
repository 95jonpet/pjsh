# The Basics
This chapter describes the basic features of `pjsh`.

## Variables
Variables can be defined using the `:=` operator.
```pjsh
my_var := my_value
```

Variables can later be referenced using the dollar, `$`, operator.
```pjsh
echo $my_var
```

Variables can also be used in interpolated strings.
```pjsh
echo $"Value: ${my_var}"
```

## Aliases
TODO: alias  
TODO: unalias

## Pipelines
POSIX-style pipelines are supported:
```pjsh
# Single line:
ls | grep 'abc' | sort

# Multiple lines:
ls \
  | grep 'abc' \
  | sort
```

## Smart pipelines
A _smart pipeline_ is intruduced to increase readability across multiple lines.
A start symbol `->|` is used to denote the start of the pipeline, and a
semicolon, `;`, symbol is used to denote its end.
```pjsh
# Multiple lines:
->| ls
  | grep 'abc'
  | sort
  ;
```
