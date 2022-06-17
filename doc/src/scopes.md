# Scopes

The shell uses the following _scope_ structure:

```plain
environment {
  pjsh {
    global {
      exported {}
      function {}
      subshell {}
    }
  }
}
```

This roughly translates into the following rules:

1. Environment variables are loaded into the topmost scope: _environment_.
2. A _pjsh_ scope is created which holds the shell's default variables.
3. A _global_ scope is created for each script or interactive session.
4. Functions and subshells are evaluated in their own, potentially nested, scopes.
5. Commands are also supplied with _exported_ variables.
6. Inner scopes can read variables defined in outer scopes.

Scopes are managed within a _context_.

Variables cannot be exported from a child scope to a parent scope.
