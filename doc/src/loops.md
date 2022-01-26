# Loops

The shell can execute a series of instructions for as long as a condition is considered `true`.

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
