---
title: Contributor Guide
description: Implementation guide for contributors — module structure, adding builtins, and key invariants.
---

This guide covers the internal module structure, key data types, hot paths,
and how to extend Grift with new builtins and value types.

## Module Structure

### `grift_arena` (no_std, no dependencies)

```
src/
├── lib.rs       Re-exports and module declarations
├── types.rs     ArenaIndex, ArenaError, ArenaResult, Slot (internal)
├── arena.rs     Arena<T, N> — alloc, free, get, set
├── traits.rs    ArenaDelete, ArenaCopy, Trace
├── gc.rs        Mark-and-sweep: initialize_roots, process_mark_stack, sweep_unmarked
├── iter.rs      ArenaIterator
└── stats.rs     ArenaStats, GcStats
```

### `grift` (no_std, depends on grift_arena)

```
src/
├── lib.rs       Crate root — #![no_std], #![forbid(unsafe_code)], re-exports
├── value.rs     Value enum, BuiltinId, Display impl, accessor macros
├── lisp.rs      Lisp<N> — arena wrapper, symbol interning, env operations
├── parse.rs     Parser — recursive descent S-expression parser
└── eval.rs      Evaluator — TCO trampoline, builtin registration, all operatives
```

## Key Data Types

### `ArenaIndex`

A `usize` wrapper. Slot 0 is `NIL`. The `is_nil()` check is `self.0 == 0`.
Comparison is by raw index — two indices are equal iff they point to the same slot.

### `Value`

A `Copy` enum (~24 bytes on 64-bit). Tag byte plus up to two `usize` fields.
The two-field limit is a design constraint.

### `Lisp<N>`

Owns the `Arena<Value, N>`. Provides the userland API: `eval`, `symbol`, `cons`,
`car`, `cdr`, environment operations.

### `Evaluator<'a, N>`

Borrows a `&'a Lisp<N>`. Holds ground environment, standard environment, and
GC root stack. Created fresh for each `Lisp::eval()` call.

### `Parser<'a>`

Cursor over a `&[u8]` byte slice. Directly allocates arena values — no
intermediate token stream.

## How to Add a New Builtin

1. **Choose operative or applicative.** Operatives receive unevaluated args;
   applicatives receive evaluated args.

2. **Add to `define_builtins!`** in `eval.rs`:

   ```rust
   operatives {
       "my-op" => op_my_op => op_my_op,
   }
   // or
   applicatives {
       "my-fn" => bi_my_fn => builtin_my_fn,
   }
   ```

3. **Implement the method** on `Evaluator`:

   For operatives:
   ```rust
   fn op_my_op(&mut self, args: ArenaIndex,
               expr: &mut ArenaIndex, env: &mut ArenaIndex) -> TailAction
   ```

   For applicatives:
   ```rust
   fn builtin_my_fn(&self, args: ArenaIndex) -> ArenaResult<ArenaIndex>
   ```

4. **Use macros** for common patterns: `type_predicate!`, `cmp_builtin!`,
   `pair_builtin!`, `fold_numbers!`.

5. **Root discipline**: If your implementation calls `self.eval()`, push any
   live values onto the root stack first.

## How to Add a New Value Variant

1. Add the variant to `enum Value` in `value.rs` (max two `ArenaIndex` fields).
2. Update `type_name()` to return a name string.
3. Update `is_self_evaluating()` — most new types should return true.
4. Update `is_immutable()` for value-based `eq?` semantics.
5. Update `Display` for printing.
6. Update `Trace` in `lisp.rs` for GC.
7. Add accessor via `value_accessor!` if needed.
8. Add type predicate via `type_predicate!` and register in `define_builtins!`.

## Invariants

These must be maintained for correctness:

1. **Slot 0 is always Nil.** `ArenaIndex::NIL` must resolve to `Value::Nil`.
2. **Symbols are interned.** Same name → same `ArenaIndex`.
3. **Ground environment is immutable.** `define!` and `set!` reject mutation.
4. **GC roots span eval calls.** Any live `ArenaIndex` across a recursive
   `eval` must be rooted.
5. **Parameter trees are validated.** Checked for cycles and duplicate symbols.
6. **All arena types are Copy.** Required by the `Cell`-based arena.
7. **No unsafe code.** `#![forbid(unsafe_code)]` in both crates.
8. **No heap allocation.** `Vec`, `String`, `Box`, `alloc::` are forbidden.

## Building and Testing

```bash
cargo build --workspace
cargo test --workspace
```

Run the REPL:

```bash
cargo run -p grift --features repl
```

Run benchmarks:

```bash
cargo run -p grift --example fib_bench --release
```
