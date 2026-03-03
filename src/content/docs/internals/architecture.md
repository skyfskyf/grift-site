---
title: Architecture
description: System architecture of the Grift interpreter — arena design, evaluation, GC, and TCO.
---

Grift is a vau-calculus Lisp interpreter that runs entirely on a fixed-size
arena allocator with no heap allocation, no standard library, and no unsafe code.

## Crate Structure

```
grift_arena     Fixed-size arena allocator with free-list, mark-and-sweep GC
grift_macros    Proc macros for grift (compile-time code generation)
grift           Parser, evaluator, value types, Lisp API — depends on grift_arena
```

`grift_arena` is a general-purpose arena with no knowledge of Lisp. `grift`
stores `Value` enums in the arena and implements the language on top of it.

## Arena Allocator

### Memory Layout

The arena is a flat array of `N` slots, where `N` is a const generic:

```
Arena<Value, N>
┌────────────────────────────────────────────────────────┐
│ slots: [Cell<Slot<Value>>; N]                          │
│ free_head: Cell<usize>       ← head of free list       │
│ len: Cell<usize>             ← count of occupied slots  │
│ gc_enabled: Cell<bool>                                  │
└────────────────────────────────────────────────────────┘
```

Each slot is a tagged union:

```rust
enum Slot<T: Copy> {
    Free { next_free: usize },     // linked-list pointer to next free slot
    Occupied { value: T },         // the stored value
}
```

Free slots form an intrusive singly-linked list. Allocation pops the head;
deallocation pushes onto it. Both are O(1).

### Interior Mutability

The arena uses `Cell<Slot<T>>` rather than `RefCell`. Since `T: Copy`, the arena
copies values in and out rather than lending references. This eliminates runtime
borrow checking overhead and the possibility of borrow panics.

### Index Design

`ArenaIndex` wraps a `usize` and directly indexes the slot array. Slot 0 is
permanently allocated as `Value::Nil`, and `ArenaIndex::NIL` is a constant
pointing to slot 0. Nil checks are a comparison against zero with no arena access.

## Value Representation

Every Lisp value is a `Value` enum stored in one arena slot. Each variant
inlines at most two `ArenaIndex`-sized fields:

```rust
enum Value {
    Nil,
    Boolean(bool),
    Number(isize),
    Char { ch: char, cdr: ArenaIndex },
    Symbol(ArenaIndex),
    Cons { car: ArenaIndex, cdr: ArenaIndex },
    String { data: ArenaIndex },
    Operative { params_envparam: ArenaIndex, body_env: ArenaIndex },
    Applicative(ArenaIndex),
    Builtin(BuiltinId),
    Environment { bindings: ArenaIndex, parents: ArenaIndex },
    Inert,
    Ignore,
}
```

### Pre-allocated Singletons

| Slot | Value | Constant |
|------|-------|----------|
| 0 | `Nil` | `ArenaIndex::NIL` |
| 1 | `Boolean(true)` | `ArenaIndex::TRUE` |
| 2 | `Boolean(false)` | `ArenaIndex::FALSE` |
| 3 | `Inert` | `ArenaIndex::INERT` |
| 4 | `Ignore` | `ArenaIndex::IGNORE` |

## Evaluation

### Trampoline TCO

The evaluator is a trampoline loop. Tail-position combiners update `expr`
and `env` and re-enter the loop rather than recursing:

```rust
loop {
    self.maybe_collect(expr, env);
    match self.lisp.arena.get(expr)? {
        Value::Symbol(_) => { /* env lookup */ }
        Value::Cons { car, cdr } => {
            // Evaluate operator, dispatch by combiner type
            // For tail calls: update expr/env and `continue`
            // For non-tail calls: `return` the result
        }
        _ => return Ok(expr), // self-evaluating
    }
}
```

Operative builtins signal their intent via `TailAction`:

```rust
enum TailAction {
    Return(ArenaResult<ArenaIndex>),  // non-tail: return immediately
    Continue,                          // tail: re-enter the eval loop
}
```

### Environment Lookup

`env_lookup` has a fast path for the common case (single-parent chain):

```rust
while !cur.is_nil() {
    // Search bindings alist
    // If single parent: cur = first_parent; continue
    // If multi-parent: fall back to DFS
}
```

## Garbage Collection

### Mark-and-Sweep

The algorithm:

1. **Root initialization**: Mark all root indices and push them onto a mark stack.
2. **Mark phase**: Iteratively pop the mark stack, trace each object's children,
   mark/push any unmarked children.
3. **Sweep phase**: Linear scan of all slots; free any occupied but unmarked.

The mark bitmap and mark stack are `[_; N]` arrays allocated on the Rust
stack, so GC requires no arena allocation.

### GC Roots

The evaluator maintains a shadow stack of GC roots as a cons-list. Any
`ArenaIndex` live across an `eval` call must be pushed onto this root stack:

```rust
self.push_root(cdr);    // protect before eval
self.push_root(env);
let result = self.eval(car, env)?;
self.pop_roots(2);      // restore after eval
```

### GC Trigger

GC triggers when arena occupancy exceeds 75%.

## Data Flow

```
"(+ 1 2)"
    │
    ▼
┌─────────┐   S-expression tokenization
│ Parser   │   Recursive descent on byte slice
│          │   Produces arena-allocated cons tree
└────┬─────┘
     │  ArenaIndex (root of AST)
     ▼
┌──────────┐
│ Evaluator │   Trampoline loop with TCO
│           │   Symbol lookup → env chain walk
│           │   Combination → combiner dispatch
│           │   GC triggered at 75% occupancy
└────┬──────┘
     │  ArenaIndex (result)
     ▼
┌──────────┐
│ arena.get │   Copy Value out of arena
└────┬──────┘
     │  Value
     ▼
  Value::Number(3)
```
