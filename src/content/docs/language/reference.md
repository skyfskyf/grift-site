---
title: Language Reference
description: Complete reference for Grift types, primitives, and evaluation rules.
---

Grift implements a strict subset of the Kernel programming language (Shutt 2010),
a Lisp dialect based on the vau calculus. This document specifies every type,
every primitive, and the evaluation rules.

## Types

| Type | Written as | Self-evaluating | Mutable | `eq?` semantics |
|------|-----------|-----------------|---------|-----------------|
| Nil | `()` | yes | no | value |
| Boolean | `#t`, `#f`, `#true`, `#false` | yes | no | value |
| Number | `42`, `-7`, `+3` | yes | no | value |
| Symbol | `foo`, `define!`, `+` | no (triggers lookup) | no | value |
| Pair | `(1 . 2)`, `(a b c)` | no (triggers combination) | no | identity |
| String | `"hello"` | yes | no | identity |
| Operative | `<operative>` | yes | no | identity |
| Applicative | `<applicative>` | yes | no | identity |
| Builtin | `<builtin>` | yes | no | identity |
| Environment | `<environment>` | yes | yes | identity |
| Inert | `#inert` | yes | no | value |
| Ignore | `#ignore` | yes | no | value |

## Evaluation Rules

### Self-Evaluating Forms

Everything except symbols and pairs evaluates to itself. Numbers, booleans,
strings, nil, inert, ignore, operatives, applicatives, builtins,
and environments all return themselves when evaluated.

### Symbol Lookup

When a symbol is evaluated, the evaluator walks the environment chain:

1. Search the current environment's bindings alist for a matching symbol.
2. If not found and there is a single parent, repeat with the parent.
3. If there are multiple parents, perform depth-first search with cycle detection.
4. If no binding is found in any ancestor, signal `UnboundVariable`.

### Combination (Function Application)

When a pair `(operator . operands)` is evaluated:

1. Evaluate `operator` to obtain a combiner.
2. Dispatch based on the combiner type:

**Operative** (compound fexpr): Pass `operands` unevaluated and the caller's
environment to the operative. Bind parameters per the parameter tree, optionally
bind the environment parameter, then evaluate the body in the operative's
closed environment extended with those bindings.

**Builtin** (Rust-native operative): Pass `operands` unevaluated and the
caller's environment to the Rust dispatch function.

**Applicative** (wrapper): Evaluate all operands left-to-right in the caller's
environment, then pass the evaluated argument list to the underlying combiner.

### Truthiness

Grift follows strict boolean semantics. The `if`, `and`, `or`, `cond`, and
`not` operatives require their test expressions to evaluate to actual boolean
values (`#t` or `#f`). Non-boolean values in boolean context signal `TypeError`.

### Formal Parameter Trees

Parameter binding uses recursive tree matching (Kernel §4.9.1):

- A **symbol** binds to the corresponding argument value.
- `#ignore` discards the corresponding argument.
- `()` (nil) requires the corresponding argument to also be nil.
- A **pair** requires the argument to be a pair and recursively matches car/cdr.

## Primitive Operatives

Operatives receive their operands unevaluated.

### `quote`

```lisp
(quote expr) ; → expr
```

Return `expr` without evaluating it.

```lisp
(quote (+ 1 2))   ; → (+ 1 2)
(quote hello)      ; → hello
```

### `if`

```lisp
(if test consequent)
(if test consequent alternative)
```

Evaluate `test`. If `#t`, evaluate `consequent` in tail position. If `#f`,
evaluate `alternative` in tail position (or return `()` if omitted).

```lisp
(if #t 1 2)        ; → 1
(if #f 1 2)        ; → 2
(if (< 3 5) 10 20) ; → 10
```

### `define!`

```lisp
(define! definiend expression)
```

Evaluate `expression`, then match `definiend` (a formal parameter tree)
against the result, binding symbols in the current environment. Returns `#inert`.

```lisp
(define! x 42)
(define! (a b) (list 1 2))  ; a → 1, b → 2
```

### `set!`

```lisp
(set! env symbol value)
```

Update an existing binding in the given environment. The environment must not
be the ground environment.

```lisp
(define! x 1)
(define! e (current-environment))
(set! e x 2)
x                  ; → 2
```

### `vau`

```lisp
(vau formals env-param . body)
```

Create an operative. `formals` is a parameter tree for the unevaluated operands.
`env-param` is bound to the caller's dynamic environment (or `#ignore` to discard it).

```lisp
(define! my-quote (vau (x) #ignore x))
(my-quote (+ 1 2))   ; → (+ 1 2)
```

### `lambda` / `fn!`

```lisp
(lambda formals . body)
(fn! name formals . body)
```

`lambda` creates an applicative (arguments are evaluated). `fn!` is shorthand for
`(define! name (lambda formals . body))`.

```lisp
(define! double (lambda (x) (* x 2)))
(fn! triple (x) (* x 3))
```

### `begin`

```lisp
(begin . body)
```

Evaluate each expression in `body` sequentially. Return the value of the last one.

```lisp
(begin
  (define! x 1)
  (define! y 2)
  (+ x y))          ; → 3
```

### `cond`

```lisp
(cond (test1 . body1) (test2 . body2) ...)
```

Evaluate tests in order. For the first `#t` test, evaluate the corresponding body.

```lisp
(cond
  ((= x 1) "one")
  ((= x 2) "two")
  (#t "other"))
```

### `and` / `or`

```lisp
(and . exprs)
(or . exprs)
```

Short-circuit boolean operators. All arguments must evaluate to booleans.

```lisp
(and #t #t #f)     ; → #f
(or #f #f #t)      ; → #t
```

### `let` / `let*` / `letrec`

```lisp
(let ((var1 val1) (var2 val2)) . body)
(let* ((var1 val1) (var2 val2)) . body)
(letrec ((var1 val1) (var2 val2)) . body)
```

`let` binds in parallel, `let*` binds sequentially, `letrec` allows mutual recursion.

```lisp
(let ((x 1) (y 2)) (+ x y))        ; → 3
(let* ((x 1) (y (+ x 1))) (+ x y)) ; → 3
```

### `when` / `unless`

```lisp
(when test . body)
(unless test . body)
```

Conditional execution. `when` executes body if test is `#t`; `unless` if `#f`.

## Applicative Primitives

Applicatives evaluate their arguments before the body runs.

### Arithmetic

| Name | Signature | Behavior |
|------|-----------|----------|
| `+` | `(+ . nums)` | Sum. `(+)` → 0 |
| `-` | `(- n . nums)` | Subtraction. `(- n)` → negation |
| `*` | `(* . nums)` | Product. `(*)` → 1 |
| `/` | `(/ n . nums)` | Integer division (truncated) |
| `%` | `(% a b)` | Remainder |

### Comparison

| Name | Signature | Behavior |
|------|-----------|----------|
| `=` | `(= . nums)` | All equal |
| `<` | `(< . nums)` | Strictly increasing |
| `>` | `(> . nums)` | Strictly decreasing |
| `<=` | `(<= . nums)` | Non-decreasing |
| `>=` | `(>= . nums)` | Non-increasing |

### Pair Operations

| Name | Signature | Behavior |
|------|-----------|----------|
| `cons` | `(cons a b)` | Construct a pair |
| `car` | `(car pair)` | First element |
| `cdr` | `(cdr pair)` | Rest element |
| `list` | `(list . args)` | Build a proper list |
| `list*` | `(list* . args)` | Build a dotted list |

### Type Predicates

| Name | Returns `#t` when |
|------|-------------------|
| `null?` | All arguments are `()` |
| `pair?` | All arguments are pairs |
| `number?` | All arguments are numbers |
| `symbol?` | All arguments are symbols |
| `boolean?` | All arguments are booleans |
| `inert?` | All arguments are `#inert` |
| `ignore?` | All arguments are `#ignore` |
| `operative?` | All arguments are operatives or builtins |
| `applicative?` | All arguments are applicatives |
| `environment?` | All arguments are environments |

### Equality

| Name | Behavior |
|------|----------|
| `eq?` | Identity equality |
| `equal?` | Structural equality (recursive on pairs, character-by-character for strings) |

### Combiner Operations

| Name | Behavior |
|------|----------|
| `eval` | Evaluate expression in given environment |
| `wrap` | Wrap a combiner as an applicative |
| `unwrap` | Extract underlying combiner from an applicative |
| `apply` | Apply applicative to argument list |

### Environment Operations

| Name | Behavior |
|------|----------|
| `make-environment` | Create environment with given parents |
| `make-empty-environment` | Create environment with no parents |
| `current-environment` | Return caller's dynamic environment (operative) |

### String / I/O

| Name | Behavior |
|------|----------|
| `raw-read-string` | Parse a string as an S-expression |
| `raw-display-to-string` | Convert value to display form |
| `raw-write-to-string` | Convert value to written form |

### System

| Name | Behavior |
|------|----------|
| `gc-collect` | Trigger garbage collection |
| `error` | Signal an error with message |

## Standard Library

Defined in the prelude and loaded lazily:

| Function | Description |
|----------|-------------|
| `(map f lst)` | Apply `f` to each element |
| `(filter pred lst)` | Keep elements where `pred` returns `#t` |
| `(length lst)` | Count elements in a proper list |
| `(append a b)` | Concatenate two lists |

## Deriving Lambda from Vau

The relationship between `vau`, `wrap`, and `lambda` is central to Grift:

```lisp
;; lambda is equivalent to:
;; (wrap (vau params #ignore (begin body...)))
```

A `vau` creates an operative: a combiner that receives operands unevaluated.
`wrap` creates an applicative: a combiner that evaluates operands first.
Therefore `lambda` — which evaluates arguments then binds them — is simply
a wrapped `vau` that ignores the caller's environment.

## Error Conditions

| Error | Condition |
|-------|-----------|
| `OutOfMemory` | Arena full, allocation failed |
| `TypeError` | Wrong type for operation |
| `ParseError` | Malformed S-expression syntax |
| `UnboundVariable` | Symbol not found in environment chain |
| `NotCallable` | Attempted to apply a non-combiner |
| `ArithmeticOverflow` | Checked arithmetic overflow |
| `DivisionByZero` | Division or modulo by zero |
| `InvalidArgument` | Bad argument (e.g., duplicate parameter symbol) |
| `ImmutableEnvironment` | Attempted to mutate the ground environment |
| `Cyclic` | Cycle detected in parameter tree or traversal |
