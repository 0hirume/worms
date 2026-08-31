# vertical-spacing

A larvae worm that applies consistent blank-line spacing to Luau.

## Spacing

The formatter keeps related declarations together and inserts one blank line:

- between services, require aliases, type groups, constants, classes, functions, and the module return;
- around multiline declarations;
- around `if`, `for`, `while`, `repeat`, and `do` blocks;
- around function and class declarations;
- before a noninitial `return`.

Declarations remain adjacent to indexed assignments that populate them. Consecutive blank lines are collapsed.

## Install

```text
larvae worm add 0hirume/worms --name vertical-spacing
larvae worm install
```
