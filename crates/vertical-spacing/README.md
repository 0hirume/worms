# vertical-spacing-worm

A native larvae formatter for plain Luau. It runs larvae's standard formatter with the project's resolved `[fmt]` settings, then enforces the deterministic parts of the vertical-spacing preferences.

It enforces:

- contiguous engine service bindings;
- contiguous requires;
- one blank line between services, requires, reimported types, local types, constants, functions, and the module return;
- contiguous adjacent simple type aliases;
- one blank line between multiline record or union aliases;
- one blank line between top-level functions;
- one blank line after a guard whose branches terminate with `return`, `break`, `continue`, or `error(...)`;
- one blank line before a noninitial `return` in any block;
- at most one blank line through larvae's standard formatter.

It does not reorder statements or guess whether arbitrary assignments are related. A boundary containing a comment is left unchanged so formatting cannot silently change which statement owns the comment.

## Use

Install the worm from a GitHub release of the workspace:

```text
larvae worm add owner/worms@0.1.0 --name vertical-spacing
larvae worm install
```

`larvae fmt` then routes `.luau` and `.lua` files through the worm.
