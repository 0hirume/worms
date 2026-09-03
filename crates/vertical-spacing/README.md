# vertical-spacing

A larvae worm that applies consistent blank-line spacing to Luau.

## Spacing

The formatter keeps related declarations together and inserts one blank line:

- between services, require aliases, type groups, classes, functions, and the module return;
- around expanded calls, tables, and types;
- around `if`, `for`, `while`, `repeat`, and `do` blocks;
- around function and class declarations;
- before a noninitial `return`.

A declaration and the indexed assignments that populate it form one block. Consecutive blank lines are collapsed.

## Install

```text
larvae worm add 0hirume/worms --name vertical-spacing
larvae worm install
```
