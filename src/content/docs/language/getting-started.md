---
title: Getting Started
description: Quick start guide for the Grift programming language.
---

Grift is a vau-calculus Lisp interpreter suitable for bare metal devices and
embedded use. It runs in `no_std`, `no_alloc` environments with zero unsafe code.

## Installation

Add Grift to your Rust project:

```bash
cargo add grift
```

## Quick Start

```rust
use grift::{Lisp, Value};

// Create a Lisp instance with a 20,000-slot arena
let lisp: Lisp<20000> = Lisp::new();

// Evaluate expressions
assert_eq!(lisp.eval("(+ 1 2)"), Ok(Value::Number(3)));
assert_eq!(lisp.eval("(car (cons 1 2))"), Ok(Value::Number(1)));
```

The const generic parameter (`20000`) sets the arena capacity in slots.

## Running the REPL

Grift includes a REPL for interactive use:

```bash
cargo run -p grift --features repl
```

Or try the [browser-based REPL](/grift-site/demo/) powered by bevy_ratatui and WebAssembly.

## Your First Program

### Basic Arithmetic

```lisp
(+ 1 2 3)           ; → 6
(* 2 (+ 3 4))       ; → 14
(- 10 3)            ; → 7
(/ 15 3)            ; → 5
(% 17 5)            ; → 2
```

### Defining Variables

```lisp
(define! x 42)
(define! greeting "hello world")
x                    ; → 42
greeting             ; → "hello world"
```

### Defining Functions

Grift uses `lambda` for regular functions (arguments are evaluated before
the function body runs):

```lisp
(define! double (lambda (x) (* x 2)))
(double 5)           ; → 10

(define! add (lambda (a b) (+ a b)))
(add 3 4)            ; → 7
```

### Operatives (Fexprs)

The distinguishing feature of Grift is the `vau` operative. Unlike `lambda`,
`vau` receives its operands **unevaluated** along with the caller's
environment:

```lisp
;; Create a custom quote — operands are NOT evaluated
(define! my-quote (vau (x) #ignore x))
(my-quote (+ 1 2))   ; → (+ 1 2), not 3

;; An operative that selectively evaluates
(define! my-if (vau (test then else) e
  (if (eval test e)
    (eval then e)
    (eval else e))))
```

### Lists

```lisp
;; Build lists with cons
(cons 1 (cons 2 (cons 3 ())))  ; → (1 2 3)

;; Or use the list function
(list 1 2 3)                    ; → (1 2 3)

;; Access elements
(car '(a b c))                  ; → a
(cdr '(a b c))                  ; → (b c)

;; Quote prevents evaluation
'(+ 1 2)                        ; → (+ 1 2)
(quote (hello world))           ; → (hello world)
```

### Conditionals

```lisp
(if #t "yes" "no")              ; → "yes"
(if (< 3 5) "less" "greater")  ; → "less"

;; cond for multiple branches
(cond
  ((= x 1) "one")
  ((= x 2) "two")
  (#t "other"))
```

### Recursion

Grift supports tail-call optimization, so recursive functions run in
constant stack space:

```lisp
;; Tail-recursive fibonacci
(define! fib (lambda (n a b)
  (if (= n 0) a
    (fib (- n 1) b (+ a b)))))
(fib 50 0 1)          ; → 12586269025
```

## Next Steps

- [Language Reference](/grift-site/language/reference/) — Complete type system and primitive reference
- [Specification](/grift-site/language/spec/) — Formal language specification
- [Architecture](/grift-site/internals/architecture/) — How Grift works internally
- [Try the REPL](/grift-site/demo/) — Interactive browser-based interpreter
