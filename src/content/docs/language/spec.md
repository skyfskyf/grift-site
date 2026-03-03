---
title: Language Specification
description: Formal reference specification for the Grift dialect, based on Kernel R-1.
---

*Version 1.5 — Reference specification for the Grift dialect, building on the
Kernel Programming Language Revised-1 Report (R-1).*

## 1. Overview

Grift is a strict subset of the Kernel programming language (Shutt, R-1).
It implements first-class operatives (vau calculus / fexprs) in a `no_std`,
`no_alloc`, `no_unsafe` environment suitable for embedded and
resource-constrained systems.

All values live in a fixed-size arena with const-generic capacity.
There is no heap allocation; all dynamic data structures are arena-allocated.

### 1.1 Deviations from Kernel R-1

Grift intentionally omits several Kernel features to remain minimal and
embedded-friendly:

- **No continuations** — `call/cc`, `guard-continuation`, and related forms
  are not provided.
- **No ports / I/O** — Standard Kernel port operations are replaced by
  string-based I/O primitives (`raw-read-string`, `raw-display-to-string`).
- **Fixed-precision integers only** — Numbers are machine-width signed
  integers (`isize`), not arbitrary-precision.
- **No mutation of pairs** — `set-car!` and `set-cdr!` are not provided;
  pairs are immutable once constructed.
- **No `$sequence`** — Use `begin` (equivalent semantics).

Everything that Grift *does* provide follows the Kernel R-1 semantics.

## 2. Types

| Type | Written as | Self-evaluating? |
|------|-----------|------------------|
| Nil | `()` | yes |
| Boolean | `#t`, `#f` | yes |
| Number | `42`, `-7`, `+3` | yes |
| Symbol | `foo`, `+`, `define!` | no (lookup) |
| Pair | `(1 . 2)`, `(a b c)` | no (combination) |
| String | `"hello\n"` | yes |
| Operative | `(vau ...)` | yes |
| Applicative | `(wrap ...)` | yes |
| Builtin | *(not writable)* | yes |
| Environment | *(not writable)* | yes |
| Inert | `#inert` | yes |
| Ignore | `#ignore` | yes |

### 2.1 Nil

The empty list. Written `()`. Used as the list terminator in proper lists.

### 2.2 Booleans

Two values: `#t` (true) and `#f` (false). Aliases `#true` and `#false`
are accepted by the reader. Boolean contexts require strict booleans —
passing a non-boolean signals `TypeError`.

### 2.3 Numbers

Machine-width signed integers (`isize`). Arithmetic uses checked operations;
overflow signals `ArithmeticOverflow` and division by zero signals
`DivisionByZero`.

### 2.4 Symbols

Interned identifiers. Two symbols with the same name share the same arena
index, so symbol equality is pointer equality. Symbols are case-sensitive.

### 2.5 Pairs

Immutable cons cells with `car` and `cdr`. A proper list is a chain of
pairs terminated by nil: `(a b c)` ≡ `(a . (b . (c . ())))`.

### 2.6 Strings

Linked lists of character nodes in the arena. Each node stores one Unicode
character and a link to the next node (or nil). Escape sequences: `\n`,
`\t`, `\r`, `\\`, `\"`.

### 2.7 Operatives

First-class fexprs created by `vau`. Receives operands **unevaluated**
together with the caller's dynamic environment.

### 2.8 Applicatives

Wrappers created by `wrap`. Evaluates all operands in the caller's
environment before passing them to the underlying combiner.

### 2.9 Environments

First-class mutable mappings from symbols to values, with a parent chain
for lexical scoping. The ground environment is immutable.

### 2.10 Inert and Ignore

`#inert` is returned by side-effecting forms. `#ignore` is used in formal
parameter trees to discard an operand.

## 3. Evaluation Rules

### 3.1 Self-Evaluation

All values except symbols and pairs evaluate to themselves.

### 3.2 Symbol Lookup

A symbol evaluates by searching the current environment's alist. If not
found, the search continues in parent environments (depth-first when
multiple parents exist). Failure signals `UnboundVariable`.

### 3.3 Combination

When a pair `(operator . operands)` is evaluated:

1. Evaluate `operator` in the current environment.
2. Dispatch based on combiner type:
   - **Operative**: Pass operands unevaluated + caller's environment
   - **Builtin**: Pass operands unevaluated + caller's environment to Rust
   - **Applicative**: Evaluate operands left-to-right, then pass to inner combiner

### 3.4 Tail Position

Expressions in tail position are optimized via trampoline. Tail positions
include: the last expression in `begin`, both branches of `if`, the body
of `let`/`let*`/`letrec`, and the body of `vau`/`lambda`.

## 4. Reader Syntax

### 4.1 Atoms

- **Booleans**: `#t`, `#f`, `#true`, `#false`
- **Special values**: `#inert`, `#ignore`
- **Numbers**: Optional sign followed by digits: `42`, `-7`, `+3`
- **Symbols**: Any sequence of non-delimiter characters that is not a
  number or special value
- **Delimiters**: space, tab, newline, CR, `(`, `)`, `"`, `;`

### 4.2 Lists

- Proper list: `(a b c)` ≡ `(a . (b . (c . ())))`
- Dotted pair: `(a . b)`
- Empty list: `()`

### 4.3 Strings

Delimited by `"`. Supported escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`.

### 4.4 Quote Shorthand

`'expr` is expanded by the reader to `(quote expr)`.

### 4.5 Comments

Line comments begin with `;` and extend to end of line.

## 5. Error Conditions

| Error | Description |
|-------|-------------|
| `OutOfMemory` | Arena full, even after GC |
| `TypeError` | Wrong type for operation |
| `ParseError` | Malformed S-expression (includes line/column) |
| `ArithmeticOverflow` | Checked arithmetic overflow |
| `DivisionByZero` | Division or modulo by zero |
| `UnboundVariable` | Symbol not found in environment chain |
| `NotCallable` | Attempted to call a non-combiner |
| `ImmutableEnvironment` | Mutating the ground environment |
| `InvalidArgument` | Invalid argument to an operation |
| `Cyclic` | Cycle detected during traversal |

## 6. Implementation Constraints

- **Arena capacity**: Fixed at compile time via const generic `N`.
- **No unsafe code**: `#![forbid(unsafe_code)]` enforced crate-wide.
- **No heap allocation**: `no_std`, `no_alloc`. Only `core::` types.
- **MSRV**: Rust 1.85 (edition 2024).
- **Tail-call optimization**: Via trampoline; unbounded tail recursion.
- **GC**: Mark-and-sweep, triggered at 75% occupancy.
