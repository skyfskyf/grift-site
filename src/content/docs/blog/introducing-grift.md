---
title: "Introducing Grift: Vau Calculus for Embedded Systems"
description: Announcing the first public release of Grift, a no_std Lisp interpreter implementing vau calculus.
---

Grift is a Lisp interpreter built from the ground up for embedded and
resource-constrained systems. It implements the **vau calculus** — a
foundational model where first-class operatives (fexprs) subsume both
traditional functions and macros.

## Why Vau Calculus?

In most Lisps, there's a fundamental split between functions (which evaluate
their arguments) and macros (which receive unevaluated syntax). This split
creates complexity: macros are second-class, can't be passed as values, and
require a separate expansion phase.

Vau calculus eliminates this split. An **operative** created with `vau`
receives its operands *unevaluated* along with the caller's environment.
The operative can then choose what to evaluate and when:

```lisp
;; An operative that selectively evaluates
(define! my-if (vau (test then else) e
  (if (eval test e)
    (eval then e)
    (eval else e))))
```

Traditional applicative functions are derived by wrapping an operative
with `wrap`, which pre-evaluates all arguments:

```lisp
;; lambda = wrap + vau
(define! double (lambda (x) (* x 2)))
;; equivalent to:
;; (define! double (wrap (vau (x) #ignore (* x 2))))
```

## Design Constraints

Grift operates under severe constraints by design:

- **`no_std`**: No standard library. Only `core::` types.
- **`no_alloc`**: No heap allocation. All data lives in a fixed-size arena.
- **`#![forbid(unsafe_code)]`**: Zero unsafe blocks across all crates.

These constraints make Grift suitable for bare-metal microcontrollers,
bootloaders, and other environments where a traditional runtime isn't
available.

## The Arena

All values live in a flat `[Cell<Slot<Value>>; N]` array where `N` is a
compile-time const generic:

```rust
let lisp: Lisp<20000> = Lisp::new();  // 20,000 slot arena
```

The arena uses a free-list for O(1) allocation and deallocation, with
mark-and-sweep garbage collection triggered at 75% occupancy. The GC's
mark bitmap and stack live on the Rust stack — not in the arena — so
garbage collection itself requires no arena allocation.

## What's Next

- **WASM REPL**: A browser-based REPL using [bevy_ratatui](https://github.com/ratatui/bevy_ratatui)
  compiled to WebAssembly.
- **More standard library**: Additional list processing, string operations,
  and pattern matching.
- **Documentation**: Comprehensive language reference and tutorials.

Check out the [documentation](/grift-site/language/getting-started/) to get started,
or [try the REPL](/grift-site/demo/) to experiment with Grift in your browser.
