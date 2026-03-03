---
title: "Building a WASM REPL with Bevy and Ratatui"
description: A technical walkthrough of compiling the Grift REPL to WebAssembly using bevy_ratatui.
---

One of the goals for Grift has been to provide an interactive REPL that
runs directly in the browser. This post walks through the architecture
of compiling a bevy_ratatui terminal application to WebAssembly.

## The Stack

The browser REPL combines three Rust crates:

1. **[Grift](https://github.com/skyfskyf/grift)** — The Lisp interpreter
   with its arena allocator and evaluator.
2. **[Ratatui](https://ratatui.rs)** — Terminal UI framework for rendering
   the REPL interface.
3. **[bevy_ratatui](https://github.com/ratatui/bevy_ratatui)** — Integration
   layer that uses Bevy's ECS and update loop to drive the Ratatui UI.

## Why bevy_ratatui?

bevy_ratatui provides a `windowed` feature that renders the Ratatui terminal
into a graphical window instead of an actual terminal. When compiled to WASM,
this window becomes a canvas element in the browser.

The key benefit is that the same Ratatui code works in both the terminal and
the browser — no separate web frontend needed.

## Architecture

```
┌─────────────────────────────────┐
│          Bevy App               │
│                                 │
│  ┌──────────┐  ┌─────────────┐  │
│  │  Input   │  │   REPL      │  │
│  │  System  │  │   System    │  │
│  └────┬─────┘  └──────┬──────┘  │
│       │               │         │
│  ┌────▼────────────────▼─────┐  │
│  │     bevy_ratatui          │  │
│  │  (windowed backend)       │  │
│  └───────────┬───────────────┘  │
│              │                  │
│         ┌────▼────┐             │
│         │ Canvas  │             │
│         └─────────┘             │
└─────────────────────────────────┘
```

The Bevy app runs at 60fps. Each frame:

1. **Input system** reads keyboard events from the browser
2. **REPL system** processes any pending input through the Grift evaluator
3. **Draw system** renders the terminal UI via Ratatui
4. **bevy_ratatui** composites the result to the HTML canvas

## WASM Considerations

### Arena Size

When compiling to WASM, the arena size affects the initial WASM memory
allocation. A `Lisp<20000>` arena works well for interactive use without
excessive memory consumption.

### No File I/O

Grift already operates without file I/O (`no_std`), making it naturally
suited for the WASM environment. Input and output are handled through
the REPL's string-based interface.

### Event Loop

Bevy's `ScheduleRunnerPlugin` handles the WASM event loop, integrating
with the browser's `requestAnimationFrame` for smooth rendering.

## Try It Out

Visit the [REPL demo](/grift-site/demo/) to try Grift in your browser,
or check the [source code](https://github.com/skyfskyf/grift) to build
it yourself.
